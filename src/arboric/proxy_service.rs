//! Arboric ProxyService which does the actual work of the Proxy

use crate::abac::PDP;
use crate::arboric::listener::ListenerContext;
use crate::Claims;
use frank_jwt::{decode, Algorithm};
use futures::future;
use http::header::HeaderMap;
use hyper::rt::Future;
use hyper::service::Service;
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};
use log::{debug, error, trace, warn};
use simple_error::bail;
use std::error::Error;
use std::sync::Arc;

use super::influxdb;

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug)]
pub struct ProxyService {
    context: Arc<ListenerContext>,
}

impl ProxyService {
    pub fn new(context: Arc<ListenerContext>) -> Self {
        ProxyService {
            context: context.clone(),
        }
    }

    fn copy_headers(inbound_headers: &HeaderMap, header_map: &mut HeaderMap) {
        debug!("Got {} headers", inbound_headers.iter().count());
        for (key, value) in inbound_headers.iter() {
            if key != "host" {
                header_map.append(key, value.into());
                debug!("Forwarding {}: {:?}", key, value);
            } else {
                debug!("Ignoring {}: {:?} header", key, value);
            }
        }
    }

    fn do_get(&self, _claims: Option<Claims>, req: Request<Body>) -> BoxFut {
        let req_uri = req.uri();
        debug!("req_uri => {}", req_uri);

        // TODO arboric::log_get(&req);

        let uri = self.compute_get_uri(&req);
        debug!("uri => {}", uri);

        let client = Client::new();
        let fut = client
            .get(uri)
            .and_then(|res| {
                debug!("GET /localhost:4000 => {}", res.status());
                future::ok(res)
            })
            .map_err(|err| {
                warn!("{}", err);
                err
            });
        Box::new(fut)
    }

    fn compute_get_uri(&self, req: &Request<Body>) -> Uri {
        let api_uri = &self.context.as_ref().api_uri;
        let authority = api_uri.authority_part().unwrap();
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

    fn do_post(
        &self,
        claims: Option<Claims>,
        inbound: Request<Body>,
    ) -> Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        use futures::stream::Stream;

        trace!("do_post({:?}, {:?})", &self, &inbound);
        let req_uri = inbound.uri();
        debug!("req_uri => {}", req_uri);

        let uri: hyper::Uri = self.context.as_ref().api_uri.clone();
        debug!("uri => {}", uri);

        let auth = self.context.as_ref().secret_key_bytes.is_some();
        if auth {
            if claims.is_none() {
                return halt(StatusCode::UNAUTHORIZED);
            }
        };

        let (parts, body) = inbound.into_parts();
        trace!("do_post({:?})", &body);

        let content_type = Self::get_content_type_as_mime_type(&parts.headers);
        trace!("content_type => {:?}", &content_type);

        let influx_db_backend = self.context.as_ref().influx_db_backend.clone();

        // TODO: Figure out the proper lifetime annotations and stop
        // cloning everything
        let pdp = self.context.as_ref().pdp.clone();

        Box::new(body.concat2().from_err().and_then(move |chunk| {
            trace!("chunk => {:?}", &chunk);
            let v = chunk.to_vec();
            let body = String::from_utf8_lossy(&v).to_string();
            debug!("body => {:?}", &body);
            if let Ok(Some((document, counts))) = super::parse_post(content_type, &body) {
                trace!("influx_db_backend => {:?}", &influx_db_backend);
                if let Some(backend) = influx_db_backend {
                    super::log_counts(&backend, &counts);
                }
                if auth {
                    let request = crate::Request {
                        claims: claims.unwrap(),
                        document,
                    };
                    if !pdp.allows(&request) {
                        return halt(StatusCode::UNAUTHORIZED);
                    }
                }
                let mut outbound = Request::post(&uri).body(Body::from(body)).unwrap();
                Self::copy_headers(&parts.headers, outbound.headers_mut());

                let client = Client::new();
                Box::new(client.request(outbound))
            } else {
                halt(StatusCode::BAD_REQUEST)
            }
        }))
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
        req: &Request<Body>,
        secret_key_bytes: &Vec<u8>,
    ) -> Result<Claims, Box<dyn Error>> {
        if let Some(authorization) = req.headers().get(http::header::AUTHORIZATION) {
            trace!("{} => {:?}", http::header::AUTHORIZATION, &authorization);
            let auth_str = &authorization.to_str()?;
            if auth_str.starts_with("Bearer ") {
                let ref token_str = auth_str[7..];
                trace!("token => {}", &token_str);
                match decode(&token_str, secret_key_bytes, Algorithm::HS256) {
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

impl Service for ProxyService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = BoxFut;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        trace!("call({:?}, {:?})", &self, &req);
        trace!("req.method() => {:?}", &req.method());
        let claims: Option<Claims>;
        if let Some(ref secret_key_bytes) = &self.context.as_ref().secret_key_bytes {
            if let Ok(map) = Self::get_authorization_token(&req, secret_key_bytes) {
                trace!("{:?}", map);
                claims = Some(map);
            } else {
                return halt(StatusCode::UNAUTHORIZED);
            }
        } else {
            claims = None;
        }
        match req.method() {
            &Method::GET => {
                trace!("about to call do_get()...");
                self.do_get(claims, req)
            }
            &Method::POST => {
                trace!("about to call do_post()...");
                self.do_post(claims, req)
            }
            _ => {
                trace!("No match!");
                halt(StatusCode::NOT_FOUND)
            }
        }
    }
}

fn respond(status_code: StatusCode) -> Response<Body> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = status_code;
    response
}

fn halt(status_code: StatusCode) -> BoxFut {
    Box::new(future::ok(respond(status_code)))
}
