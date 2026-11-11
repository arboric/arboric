//! Arboric ProxyService which does the actual work of the Proxy

use crate::Claims;
use crate::arboric::listener::ListenerContext;
use bytes::Bytes;
use frank_jwt::{Algorithm, ValidationOptions, decode};
use http::header::HeaderMap;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::body::Incoming;
use hyper::{Method, Request, Response, StatusCode, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use log::{debug, error, trace, warn};
use simple_error::bail;
use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;

/// The response body type that the proxy returns
pub type ProxyBody = BoxBody<Bytes, hyper::Error>;

/// The HTTP client the proxy uses to call the back-end API
pub type HttpClient = Client<HttpConnector, Full<Bytes>>;

#[derive(Debug, Clone)]
pub struct ProxyService {
    context: Arc<ListenerContext>,
}

impl ProxyService {
    pub fn new(context: Arc<ListenerContext>) -> Self {
        ProxyService { context }
    }

    /// Handles one inbound request. Errors become HTTP status codes, so this
    /// never fails.
    pub async fn call(&self, req: Request<Incoming>) -> Result<Response<ProxyBody>, Infallible> {
        trace!("call({:?}, {:?})", self, req);
        trace!("req.method() => {:?}", req.method());
        let claims: Option<Claims> = match &self.context.secret_key_bytes {
            Some(secret_key_bytes) => {
                match Self::get_authorization_token(req.headers(), secret_key_bytes) {
                    Ok(map) => {
                        trace!("{:?}", map);
                        Some(map)
                    }
                    Err(_) => return Ok(respond(StatusCode::UNAUTHORIZED)),
                }
            }
            None => None,
        };
        let response = match *req.method() {
            Method::GET => {
                trace!("about to call do_get()...");
                self.do_get(claims, req).await
            }
            Method::POST => {
                trace!("about to call do_post()...");
                self.do_post(claims, req).await
            }
            _ => {
                trace!("No match!");
                respond(StatusCode::NOT_FOUND)
            }
        };
        Ok(response)
    }

    fn copy_headers(inbound_headers: &HeaderMap, header_map: &mut HeaderMap) {
        debug!("Got {} headers", inbound_headers.iter().count());
        for (key, value) in inbound_headers.iter() {
            if key != "host" {
                header_map.append(key, value.clone());
                debug!("Forwarding {}: {:?}", key, value);
            } else {
                debug!("Ignoring {}: {:?} header", key, value);
            }
        }
    }

    async fn do_get(&self, _claims: Option<Claims>, req: Request<Incoming>) -> Response<ProxyBody> {
        let req_uri = req.uri();
        debug!("req_uri => {}", req_uri);

        // TODO arboric::log_get(&req);

        let uri = self.compute_get_uri(&req);
        debug!("uri => {}", uri);

        let outbound = Request::get(uri)
            .body(Full::new(Bytes::new()))
            .expect("GET request should be valid");
        let response = self.forward(outbound).await;
        debug!("GET {} => {}", req_uri, response.status());
        response
    }

    fn compute_get_uri(&self, req: &Request<Incoming>) -> Uri {
        let api_uri = &self.context.api_uri;
        let authority = api_uri.authority().unwrap();
        let scheme = api_uri.scheme_str().unwrap();
        let params = req.uri().query().unwrap();
        let pandq = format!("/graphql?{}", params);
        Uri::builder()
            .scheme(scheme)
            .authority(authority.as_str())
            .path_and_query(pandq.as_str())
            .build()
            .unwrap()
    }

    async fn do_post(
        &self,
        claims: Option<Claims>,
        inbound: Request<Incoming>,
    ) -> Response<ProxyBody> {
        trace!("do_post({:?}, {:?})", self, inbound);

        let (parts, body) = inbound.into_parts();
        trace!("do_post({:?})", body);

        let content_type = Self::get_content_type_as_mime_type(&parts.headers);
        trace!("content_type => {:?}", content_type);

        let context = &self.context;

        let auth = context.secret_key_bytes.is_some();
        if auth && claims.is_none() {
            return respond(StatusCode::UNAUTHORIZED);
        }

        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(err) => {
                warn!("Unable to read request body: {}", err);
                return respond(StatusCode::BAD_REQUEST);
            }
        };

        let uri = &context.api_uri;
        debug!("uri => {}", uri);

        let pdp = &context.pdp;
        let body = String::from_utf8_lossy(&bytes).to_string();
        debug!("body => {:?}", body);
        if let Ok(Some((document, counts))) = super::parse_post(content_type, &body) {
            trace!("influx_db_backend => {:?}", context.influx_db_backend);
            if let Some(backend) = &context.influx_db_backend {
                super::log_counts(backend, &counts);
            }
            if auth {
                let request = crate::Request {
                    claims: claims.unwrap(),
                    document,
                };
                if !pdp.allows(&request) {
                    return respond(StatusCode::UNAUTHORIZED);
                }
            }
            let mut outbound = Request::post(uri.clone())
                .body(Full::new(bytes))
                .expect("POST request should be valid");
            Self::copy_headers(&parts.headers, outbound.headers_mut());

            self.forward(outbound).await
        } else {
            respond(StatusCode::BAD_REQUEST)
        }
    }

    /// Sends the request to the back-end API and streams the response back.
    /// If the back-end cannot be reached, returns `502 Bad Gateway`.
    async fn forward(&self, outbound: Request<Full<Bytes>>) -> Response<ProxyBody> {
        match self.context.client.request(outbound).await {
            Ok(response) => response.map(|body| body.boxed()),
            Err(err) => {
                warn!("{}", err);
                respond(StatusCode::BAD_GATEWAY)
            }
        }
    }

    fn get_content_type_as_mime_type(headers: &HeaderMap) -> Option<mime::Mime> {
        trace!("get_content_type_as_mime_type()");
        match headers.get(http::header::CONTENT_TYPE) {
            Some(header_value) => {
                trace!("header_value => {:?}", header_value);
                match header_value.to_str() {
                    Ok(s) => {
                        trace!("s => {}", s);
                        match s.parse::<mime::Mime>() {
                            Ok(mime) => Some(mime),
                            Err(err) => {
                                warn!("{}", err);
                                None
                            }
                        }
                    }
                    Err(err) => {
                        warn!("{}", err);
                        None
                    }
                }
            }
            _ => None,
        }
    }

    fn get_authorization_token(
        headers: &HeaderMap,
        secret_key_bytes: &Vec<u8>,
    ) -> Result<Claims, Box<dyn Error>> {
        if let Some(authorization) = headers.get(http::header::AUTHORIZATION) {
            trace!("{} => {:?}", http::header::AUTHORIZATION, authorization);
            let auth_str = &authorization.to_str()?;
            if let Some(token_str) = auth_str.strip_prefix("Bearer ") {
                trace!("token => {}", token_str);
                match decode(
                    token_str,
                    secret_key_bytes,
                    Algorithm::HS256,
                    &ValidationOptions::dangerous(),
                ) {
                    Ok((_header, payload)) => match payload {
                        serde_json::Value::Object(map) => Ok(map),
                        x => {
                            error!("Expeced JSON Object, got {:?}!", x);
                            bail!("401 Unauthorized")
                        }
                    },
                    Err(e) => {
                        error!("{}", e);
                        bail!("401 Unauthorized")
                    }
                }
            } else {
                bail!("401 Unauthorized")
            }
        } else {
            bail!("401 Unauthorized")
        }
    }
}

fn respond(status_code: StatusCode) -> Response<ProxyBody> {
    let body = Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed();
    let mut response = Response::new(body);
    *response.status_mut() = status_code;
    response
}
