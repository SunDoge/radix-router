extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate radix_router;

use futures::future;
use hyper::rt::{self, Future};
use hyper::{Body, Request, Response, Server};
use radix_router::router::{BoxFut, Params, Router};

fn index(_: Request<Body>, _: Option<Params>) -> BoxFut {
    let res = Response::builder().body("welcome!\n".into()).unwrap();
    Box::new(future::ok(res))
}

fn hello(_: Request<Body>, ps: Option<Params>) -> BoxFut {
    let params = ps.unwrap();
    let name = params.by_name("name").unwrap();
    let res = Response::builder()
        .body(format!("hello, {}!\n", name).into())
        .unwrap();
    Box::new(future::ok(res))
}

fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    let mut router = Router::new();
    router.get("/", index);
    router.get("/hello/:name", hello);

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        // This is the `Service` that will handle the connection.
        router.clone()
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
