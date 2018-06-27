// use http::{Request, Response};
use futures::{future, IntoFuture};
use hyper::error::Error;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::collections::BTreeMap;
use tree::Node;

// TODO think more about what a handler looks like
pub type Handle = fn(Request<Body>, Option<Params>) -> Response<Body>;

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub key: String,
    pub value: String,
}

impl Param {
    pub fn new(key: &str, value: &str) -> Param {
        Param {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Params(pub Vec<Param>);

impl Params {
    pub fn by_name(&self, name: &str) -> Option<&str> {
        match self.0.iter().find(|param| param.key == name) {
            Some(param) => Some(&param.value),
            None => None,
        }
    }
}

pub struct Router<T> {
    pub trees: BTreeMap<String, Node<T>>,
}

impl<T> Router<T> {
    pub fn new() -> Router<T> {
        Router {
            trees: BTreeMap::new(),
        }
    }

    pub fn get(&mut self, path: &str, handle: T) {
        self.handle("GET", path, handle);
    }

    pub fn head(&mut self, path: &str, handle: T) {
        self.handle("HEAD", path, handle);
    }

    pub fn options(&mut self, path: &str, handle: T) {
        self.handle("OPTIONS", path, handle);
    }

    pub fn post(&mut self, path: &str, handle: T) {
        self.handle("POST", path, handle);
    }

    pub fn put(&mut self, path: &str, handle: T) {
        self.handle("PUT", path, handle);
    }

    pub fn patch(&mut self, path: &str, handle: T) {
        self.handle("PATCH", path, handle);
    }

    pub fn delete(&mut self, path: &str, handle: T) {
        self.handle("DELETE", path, handle);
    }

    pub fn group() {}

    pub fn serve_files(&mut self, path: &str) {
        if path.as_bytes().len() < 10 || &path[path.len() - 10..] != "/*filepath" {
            panic!("path must end with /*filepath in path '{}'", path);
        }
    }

    pub fn handle(&mut self, method: &str, path: &str, handle: T) {
        if !path.starts_with("/") {
            panic!("path must begin with '/' in path '{}'", path);
        }

        self.trees
            .entry(method.to_string())
            .or_insert(Node::new())
            .add_route(path, handle);
    }

    pub fn lookup(&mut self, method: &str, path: &str) -> (Option<&T>, Option<Params>, bool) {
        self.trees
            .get_mut(method)
            .and_then(|n| Some(n.get_value(path)))
            .unwrap_or((None, None, false))
    }
}

impl<T> Service for Router<T>
where
    T: Fn(Request<Body>, Option<Params>) -> Response<Body>,
{
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = future::FutureResult<Response<Self::ResBody>, Self::Error>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        let (handle, p, _) = self.lookup(req.method().as_str(), req.uri().path());
        match handle {
            Some(h) => future::ok(h(req, p)),
            _ => future::ok(Response::new(Body::from("not found"))),
        }
    }
}

impl<T> IntoFuture for Router<T> {
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Error;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn params() {
        use router::{Param, Params};

        let params = Params(vec![
            Param {
                key: "fuck".to_owned(),
                value: "you".to_owned(),
            },
            Param {
                key: "lalala".to_string(),
                value: "papapa".to_string(),
            },
        ]);

        assert_eq!(Some("you"), params.by_name("fuck"));
        assert_eq!(Some("papapa"), params.by_name("lalala"));
    }

    #[test]
    #[should_panic(expected = "path must begin with '/' in path 'something'")]
    fn handle_ivalid_path() {
        // use http::Response;
        use hyper::{Body, Request, Response};
        use router::Router;

        let path = "something";
        let mut router = Router::new();

        router.handle("GET", path, |_req: Request<Body>| {
            Response::new(Body::from("test"))
        });
    }
}
