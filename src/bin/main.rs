extern crate acrouter;
extern crate http;

use acrouter::{router::Router, tree::Node};
use http::{Request, Response};

fn fake_handle(req: Request<()>) -> Response<()> {
    Response::new(())
}

fn main() {
    let _router = Router::new();
    let node: Node<()> = Node::new();
    println!("{:?}", node);
    node.add_route();
}
