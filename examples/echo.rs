extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate radix_router;

use futures::future;
use hyper::rt::{self, Future, Stream};
use hyper::{Body, Request, Response, Server};
use radix_router::router::Params;
use radix_router::router::{BoxFut, Router};

// static PHRASE: &'static [u8] = b"Hello World!";

fn get_echo(_: Request<Body>, _: Params) -> BoxFut {
    // Box::new(future::ok(Response::new(Body::from("Try POSTing data to /echo"))))
    // *response.body_mut() = Body::from("Try POSTing data to /echo");
    let response = Response::builder()
        .body(Body::from("Try POSTing data to /echo"))
        .unwrap();
    Box::new(future::ok(response))
}

fn post_echo(req: Request<Body>, _: Params) -> BoxFut {
    // Box::new(future::ok(Response::new(req.into_body())))
    // *response.body_mut() = req.into_body();
    let response = Response::builder().body(req.into_body()).unwrap();
    Box::new(future::ok(response))
}

fn post_echo_uppercase(req: Request<Body>, _: Params) -> BoxFut {
    let mapping = req.into_body().map(|chunk| {
        chunk
            .iter()
            .map(|byte| byte.to_ascii_uppercase())
            .collect::<Vec<u8>>()
    });

    // *response.body_mut() = Body::wrap_stream(mapping);
    let response = Response::builder()
        .body(Body::wrap_stream(mapping))
        .unwrap();
    Box::new(future::ok(response))
}

fn post_echo_reversed(req: Request<Body>, _: Params) -> BoxFut {
    let reversed = req.into_body().concat2().map(move |chunk| {
        let body = chunk.iter().rev().cloned().collect::<Vec<u8>>();
        // *response.body_mut() = Body::from(body);
        // response
        Response::builder().body(Body::from(body)).unwrap()
    });
    Box::new(reversed)
}

fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();
    let some_str = "Some";

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        // This is the `Service` that will handle the connection.
        // `service_fn_ok` is a helper to convert a function that
        // returns a Response into a `Service`.
        // service_fn_ok(|_| {
        //     Response::new(Body::from(PHRASE))
        // })

        // router.get("/some", |req, ps| {
        //     Box::new(future::ok(Response::new(Body::empty())))
        // });
        let mut router = Router::new();
        router.get("/", get_echo);
        router.post("/echo", post_echo);
        router.post("/echo/uppercase", post_echo_uppercase);
        router.post("/echo/reversed", post_echo_reversed);
        router.get("/some", move |_, _| -> BoxFut {
            Box::new(future::ok(
                Response::builder().body(some_str.into()).unwrap(),
            ))
        });
        router.serve_files("/examples/*filepath", "examples");
        router
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
