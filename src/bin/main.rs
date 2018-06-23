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
    let _router = Router::new();
    let mut node = Node::new();
    node.add_route("/something".as_bytes(), fake_handle);
    println!("{:?}", node);
    // node.add_route();extern crate http;
}
