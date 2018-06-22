use hyper::{Body, Request, Response};

pub type Handle = fn(Request<Body>) -> Response<Body>;

#[derive(Debug)]
pub struct Param {
    key: String,
    value: String,
}

#[derive(Debug)]
pub struct Params(Vec<Param>);

impl Params {
    pub fn by_name(&self, name: &str) -> Option<String> {
        match self.0.iter().find(|param| param.key == name) {
            Some(param) => Some(param.value.clone()),
            None => None,
        }
    }
}

pub struct Router {}

impl Router {
    pub fn new() -> Router {
        Router {}
    }

    pub fn get() {}

    pub fn post() {}

    pub fn put() {}

    pub fn patch() {}

    pub fn delete() {}

    pub fn group() {}

    pub fn serve_files() {}

    pub fn handle(&mut self, method: &str, path: &str, handle: Handle) {
        if !path.starts_with("/") {
            panic!("path must begin with '/' in path '{}'", path);
        }
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

        assert_eq!(Some(String::from("you")), params.by_name("fuck"));
        assert_eq!(Some(String::from("papapa")), params.by_name("lalala"));
    }

    #[test]
    #[should_panic(expected = "path must begin with '/' in path 'something'")]
    fn handle_ivalid_path() {
        use hyper::{Body, Response};
        use router::Router;

        let path = "something";
        let mut router = Router::new();

        router.handle("GET", path, |_req| Response::new(Body::from("something")));
    }
}
