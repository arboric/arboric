//! Arboric ProxyService which does the actual work of the Proxy

use futures::future;
use http::header::HeaderMap;
use hyper::rt::Future;
use hyper::service::Service;
use hyper::{Body, Client, Method, Request, Response, StatusCode, Uri};
use log::{debug, trace, warn};

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Debug)]
pub struct ProxyService {
    pub api_uri: String,
}

impl ProxyService {
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

    fn do_get(&self, req: Request<Body>) -> BoxFut {
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
        let api_uri: Uri = self.api_uri.parse().unwrap();
        let authority = api_uri.authority_part().unwrap();
        let scheme = api_uri.scheme_str().unwrap();
        let params = req.uri().query().unwrap();
        let pandq = format!("/graphql?{}", params);
        Uri::builder()
            .scheme(scheme)
            .authority(authority.as_str())
            .path_and_query(&pandq[..])
            .build()
            .unwrap()
    }

    fn do_post(
        &self,
        inbound: Request<Body>,
    ) -> Box<impl Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        trace!("do_post({:?}, {:?})", &self, &inbound);
        let req_uri = inbound.uri();
        debug!("req_uri => {}", req_uri);

        let uri: hyper::Uri = self.api_uri.parse().unwrap();
        debug!("uri => {}", uri);

        let (parts, body) = inbound.into_parts();

        trace!("do_post({:?})", &body);

        use futures::stream::Stream;
        let concat = body.concat2();

        let content_type = Self::get_content_type_as_mime_type(&parts.headers);
        trace!("content_type => {:?}", &content_type);
        trace!("concat => {:?}", concat);
        let s = concat
            .map(move |chunk| {
                trace!("chunk => {:?}", &chunk);
                let v = chunk.to_vec();
                let body = String::from_utf8_lossy(&v).to_string();
                debug!("body => {:?}", &body);
                super::log_post(content_type, &body);
                body
            })
            .into_stream();
        let mut r = Request::post(&uri).body(Body::wrap_stream(s)).unwrap();
        Self::copy_headers(&parts.headers, r.headers_mut());

        let client = Client::new();
        Box::new(client.request(r))
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
}

impl Service for ProxyService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = BoxFut;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        trace!("call({:?}, {:?})", &self, &req);
        trace!("req.method() => {:?}", &req.method());
        match req.method() {
            &Method::GET => {
                trace!("about to call do_get()...");
                self.do_get(req)
            }
            &Method::POST => {
                trace!("about to call do_post()...");
                self.do_post(req)
            }
            _ => {
                trace!("No match!");
                let mut response = Response::new(Body::empty());
                *response.status_mut() = StatusCode::NOT_FOUND;
                Box::new(future::ok(response))
            }
        }
    }
}
