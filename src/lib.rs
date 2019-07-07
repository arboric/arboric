
use futures::future;
use http::header::HeaderMap;
/// The arboric library
use log::{debug, info, trace, warn};
use hyper::client::ResponseFuture;
use hyper::rt::Future;
use hyper::service::{NewService, Service};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};

mod arboric;

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    api_uri: String,
}

#[derive(Debug)]
pub struct ProxyService {
    api_uri: String,
}

impl ProxyService {

    fn forward_headers<T>(request: &Request<T>, header_map: &mut HeaderMap)
    where
        T: std::fmt::Debug,
    {
        trace!("forward_headers({:?}, {:?})", &request, &header_map);
        Self::copy_headers(request.headers(), header_map);
    }

    fn copy_headers(inbound_headers: &HeaderMap, header_map: &mut HeaderMap) {
        debug!("Got {} headers", header_map.iter().count());
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

    fn do_post(&self, inbound: Request<Body>) -> Box<impl Future<Item = Response<Body>, Error = hyper::Error> + Send> {
        trace!("do_post({:?}, {:?})", &self, &inbound);
        let req_uri = inbound.uri();
        debug!("req_uri => {}", req_uri);

        let uri: hyper::Uri = self.api_uri.parse().unwrap();
        debug!("uri => {}", uri);

        let (parts, body) = inbound.into_parts();

        trace!("log_post({:?})", &body);

        use futures::stream::Stream;
        let concat = body.concat2();

        trace!("concat => {:?}", concat);
        let s = concat.map(move |chunk| {
            trace!("chunk => {:?}", &chunk);
            let v = chunk.to_vec();
            let s = String::from_utf8_lossy(&v).to_string();
            debug!("s => {:?}", &s);
            arboric::log_post(&s);
            s
        }).into_stream();
        let mut r = Request::post(&uri).body(Body::wrap_stream(s)).unwrap();
        ProxyService::copy_headers(&parts.headers, r.headers_mut());

        let client = Client::new();
        Box::new(client.request(r))
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

impl NewService for Proxy {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type InitError = hyper::Error;
    type Future = Box<Future<Item = Self::Service, Error = Self::InitError> + Send>;
    type Service = ProxyService;
    fn new_service(&self) -> Self::Future {
        trace!("new_service(&Proxy)");
        Box::new(future::ok(ProxyService {
            api_uri: self.api_uri.clone(),
        }))
    }
}


impl Proxy {
    pub fn new<S>(api_uri: S) -> Proxy
    where
        S: Into<String>,
    {
        Proxy {
            api_uri: api_uri.into(),
        }
    }

    pub fn run(self) {
        // This is our socket address...
        let addr = ([127, 0, 0, 1], 4000).into();

        let bound = Server::bind(&addr);
        info!("Proxy listening on {}", &addr);
        let server = bound
            .serve(self)
            .map_err(|e| eprintln!("server error: {}", e));

        // Run this server for... forever!
        hyper::rt::run(server);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
