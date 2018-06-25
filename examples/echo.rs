extern crate acrouter;
extern crate futures;
extern crate hyper;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::{service_fn, Service};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;

use acrouter::router::Router;

/// We need to return different futures depending on the route matched,
/// and we can do that with an enum, such as `futures::Either`, or with
/// trait objects.
///
/// A boxed Future (trait object) is used as it is easier to understand
/// and extend with more types. Advanced users could switch to `Either`.
type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
type Handler = fn(Request<Body>, Response<Body>) -> BoxFut;

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
fn echo(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => {
            *response.body_mut() = Body::from("Try POSTing data to /echo");
        }

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => {
            *response.body_mut() = req.into_body();
        }

        // Convert to uppercase before sending back to client.
        (&Method::POST, "/echo/uppercase") => {
            let mapping = req.into_body().map(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });

            *response.body_mut() = Body::wrap_stream(mapping);
        }

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive. So, this returns a different
        // future, waiting on concatenating the full body, so that
        // it can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            let reversed = req.into_body().concat2().map(move |chunk| {
                let body = chunk.iter().rev().cloned().collect::<Vec<u8>>();
                *response.body_mut() = Body::from(body);
                response
            });

            return Box::new(reversed);
        }

        // The 404 Not Found route...
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };

    Box::new(future::ok(response))
}

fn get_echo(_: Request<Body>, mut response: Response<Body>) -> BoxFut {
    *response.body_mut() = Body::from("Try POSTing data to /echo");
    Box::new(future::ok(response))
}

fn post_echo(req: Request<Body>, mut response: Response<Body>) -> BoxFut {
    *response.body_mut() = req.into_body();
    Box::new(future::ok(response))
}

fn post_echo_uppercase(req: Request<Body>, mut response: Response<Body>) -> BoxFut {
    let mapping = req.into_body().map(|chunk| {
        chunk
            .iter()
            .map(|byte| byte.to_ascii_uppercase())
            .collect::<Vec<u8>>()
    });

    *response.body_mut() = Body::wrap_stream(mapping);
    Box::new(future::ok(response))
}

fn post_echo_reversed(req: Request<Body>, mut response: Response<Body>) -> BoxFut {
    let reversed = req.into_body().concat2().map(move |chunk| {
        let body = chunk.iter().rev().cloned().collect::<Vec<u8>>();
        *response.body_mut() = Body::from(body);
        response
    });

    Box::new(reversed)
}



fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();
    let mut router: Router<Handler> = Router::new();
    router.handle("GET", "/", get_echo);
    router.handle("POST", "/echo", post_echo);
    router.handle("POST", "/echo/uppercase", post_echo_uppercase);
    router.handle("POST", "/echo/reversed", post_echo_reversed);
    let server = Server::bind(&addr)
        .serve(|| service_fn(echo))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
