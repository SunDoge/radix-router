extern crate acrouter;
extern crate hyper;

use acrouter::{router::Router, tree::Node};
use hyper::{Body, Request, Response};
use std::rc::Rc;

// fn fake_handle(req: Request<Body>) -> Response<Body> {
//     Response::new(Body::from("test"))
// }

fn main() {
    let _router = Router::new();
    let fake_handle = 12;

    let mut root = Node::new("/posts", fake_handle);
    // println!("{:?}", node);
    root.insert_child(1, "/posts/1", "/posts/1", fake_handle);
    root.insert_child(1, "/posts/2", "/posts/1/edit", fake_handle);
    root.insert_child(1, "1", "1", fake_handle);
    root.increment_child_prio(1);
    println!("{:#?}", root);
}
