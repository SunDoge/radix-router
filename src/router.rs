// use http::{Request, Response};
use hyper::{Body, Request, Response};
use std::collections::BTreeMap;
use tree::Node;

// pub type Handle = fn(Request<Body>) -> Response<Body>;

#[derive(Debug, Clone)]
pub struct Param {
    pub key: String,
    pub value: String,
}

#[derive(Debug)]
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

    pub fn get() {}

    pub fn post() {}

    pub fn put() {}

    pub fn patch() {}

    pub fn delete() {}

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
