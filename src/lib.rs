/// The arboric library
use futures::future;
use http::header::HeaderMap;
use hyper::rt::Future;
use hyper::service::{NewService, Service};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri};
use log::{debug, warn};

// Just a simple type alias
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

/// The main Proxy
#[derive(Debug)]
pub struct Proxy {
    api_uri: String,
}

pub struct ProxyService {
    api_uri: String,
}

impl ProxyService {

    fn forward_headers<T>(request: &Request<T>, header_map: &mut HeaderMap) {
        debug!("Got {} headers", header_map.iter().count());
        for (key, value) in request.headers().iter() {
            if key != "host" {
                header_map.append(key, value.into());
                debug!("Forwarded {} => {:?}", key, value);
            } else {
                debug!("Ignored {} => {:?}", key, value);
            }
        }
    }

    fn do_get(&self, req: Request<Body>) -> BoxFut {
        let req_uri = req.uri();
        debug!("req_uri => {}", req_uri);

        let api_uri: Uri = self.api_uri.parse().unwrap();
        let authority = api_uri.authority_part().unwrap();
        let scheme = api_uri.scheme_str().unwrap();
        let params = req.uri().query().unwrap();
        let pandq = format!("/graphql?{}", params);
        let uri = Uri::builder()
            .scheme(scheme)
            .authority(authority.as_str())
            .path_and_query(&pandq[..])
            .build()
            .unwrap();
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

    fn do_post(&self, req: Request<Body>) -> BoxFut {
        let req_uri = req.uri();
        debug!("req_uri => {}", req_uri);

        let uri: hyper::Uri = self.api_uri.parse().unwrap();
        debug!("uri => {}", uri);

        debug!("{:?}", req.body());

        let mut request = Request::post(uri).body(Body::empty()).unwrap();

        ProxyService::forward_headers(&req, request.headers_mut());
        *request.body_mut() = req.into_body();

        let client = Client::new();
        Box::new(client.request(request))
    }

}


impl Service for ProxyService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = hyper::Error;
    type Future = BoxFut;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        match req.method() {
            &Method::GET => self.do_get(req),
            &Method::POST => self.do_post(req),
            _ => {
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
