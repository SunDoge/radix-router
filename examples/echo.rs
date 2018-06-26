extern crate httprouter;
extern crate hyper;
extern crate pretty_env_logger;

use httprouter::router::Params;
use httprouter::router::Router;
use hyper::rt::{self, Future};
// use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};

// static PHRASE: &'static [u8] = b"Hello World!";
type Handle = fn(Request<Body>, Option<Params>) -> Response<Body>;

fn get_echo(_: Request<Body>, _: Option<Params>) -> Response<Body> {
    Response::new(Body::from("Try POSTing data to /echo"))
}

fn post_echo(req: Request<Body>, _: Option<Params>) -> Response<Body> {
    Response::new(req.into_body())
}

fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = || {
        // This is the `Service` that will handle the connection.
        // `service_fn_ok` is a helper to convert a function that
        // returns a Response into a `Service`.
        // service_fn_ok(|_| {
        //     Response::new(Body::from(PHRASE))
        // })
        let mut router: Router<Handle> = Router::new();
        router.handle("GET", "/echo", get_echo);
        router.handle("POST", "/echo", post_echo);
        router
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
