# Radix-Router
[![Build Status](https://travis-ci.org/SunDoge/radix-router.svg?branch=master)](https://travis-ci.org/SunDoge/radix-router)
[![crates.io](http://meritbadge.herokuapp.com/radix-router)](https://crates.io/crates/radix-router)
[![Released API docs](https://docs.rs/radix-router/badge.svg)](https://docs.rs/radix-router)

Radix-Router is a Rust port of [julienschmidt/httprouter](https://github.com/julienschmidt/httprouter).

## Usage
This is just a quick introduction.

Let's start with a `hello world` example:
```rust
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;
extern crate radix_router;

use futures::future;
use hyper::rt::{self, Future};
use hyper::{Body, Request, Response, Server};
use radix_router::router::{BoxFut, Params, Router, Handler};

fn index(_: Request<Body>, _: Params) -> BoxFut {
    let res = Response::builder().body("welcome!\n".into()).unwrap();
    Box::new(future::ok(res))
}

fn hello(_: Request<Body>, ps: Params) -> BoxFut {
    // let name = ps.by_name("name").unwrap();
    let name = &ps[0];
    let res = Response::builder()
        .body(format!("hello, {}!\n", name).into())
        .unwrap();
    Box::new(future::ok(res))
}

fn main() {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    // new_service is run for each connection, creating a 'service'
    // to handle requests for that specific connection.
    let new_service = move || {
        // This is the `Service` that will handle the connection.
        let mut router: Router<Handler> = Router::new();
        router.get("/", Box::new(index));
        router.get("/hello/:name", Box::new(hello));
        router
    };

    let server = Server::bind(&addr)
        .serve(new_service)
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);

    rt::run(server);
}
```

### Handler
The handler can be anything. You can store a `T` and get an `Option<&T>`. Notice that `&T` is immutable. We offer a default `radix_router::router::Handler` which can be a `fn` or `closure`. When using closure, you are able to capture outside parameters. For example:

```rust 
router.get("/", Box::new(get_echo));
router.post("/echo", Box::new(post_echo));
router.post("/echo/uppercase", Box::new(post_echo_uppercase));
router.post("/echo/reversed", Box::new(post_echo_reversed));
router.get("/some", Box::new(move |_, _| -> BoxFut {
    Box::new(future::ok(
        Response::builder().body(some_str.into()).unwrap(),
    ))
}));
```

### Named parameters
`:name` is a *named parameter*. The values are accessible via `Option<Params>`, which is a wrapped slice of `Param`s. You can get the value of a parameter either by its index in the slice. of by using the `by_name(name)` method.

Named parameters only match a single path segment:
```
Pattern: /user/:user

 /user/gordon              match
 /user/you                 match
 /user/gordon/profile      no match
 /user/                    no match
```

**Note:** Since this router has only explicit matches, you can not register static routes and parameters for the same path segment. For example you can not register the patterns `/user/new` and `/user/:user` for the same request method at the same time. The routing of different request methods is independent from each other.

### Catch-All parameters

The second type are *catch-all* parameters and have the form `*name`. Like the name suggests, they match everything. Therefore they must always be at the **end** of the pattern:

```
Pattern: /src/*filepath

 /src/                     match
 /src/somefile.go          match
 /src/subdir/somefile.go   match
```

### Static files
You can serve static files by using:
```rust
router.serve_files("/examples/*filepath", "examples");
```

## Examples
An echo server example is written. You can test it by running

```bash
$ cargo run --example echo
```

```bash
$ curl http://127.0.0.1:3000/echo
Try POSTing data to /echo

$ curl -d "param1=1&param2=2" -X POST http://127.0.0.1:3000/echo
param1=1&param2=2
```
