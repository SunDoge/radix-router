extern crate acrouter;
// extern crate http;
extern crate hyper;

use acrouter::{router::Router, tree::Node};
// use http::{Request, Response};
use hyper::{Body, Request, Response};


fn fake_handle(req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("test"))
}

fn main() {
    let mut router = Router::new();
    router.handle("GET", "/post", fake_handle);
}
