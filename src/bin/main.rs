extern crate acrouter;
// extern crate http;
extern crate hyper;

use acrouter::{router::Router, tree::Node};
// use http::{Request, Response};
use hyper::{Body, Request, Response};

// fn fake_handle(req: Request<Body>) -> Response<Body> {
//     Response::new(Body::from("test"))
// }

// fn fake_handle(req: u32) -> u32 {
//     12
// }
struct Faker {}

impl Faker {
    pub fn call(&self) {
        println!("call");
    }
}

fn main() {
    // let fake_handle = 1;
    let mut router = Router::new();
    router.handle("GET", "/post", Faker{});
    // println!("{:#?}", router.trees);
    let (handle, params, tsr) = router.lookup("GET", "/post");
    match handle {
        Some(h) => h.call(),
        None => println!("None"),
    }
}
