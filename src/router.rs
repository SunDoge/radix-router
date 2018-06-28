/// use http::{Request, Response};
use futures::{future, IntoFuture};
use hyper::error::Error;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use std::collections::BTreeMap;
use tree::Node;

/// TODO think more about what a handler looks like
/// Handle is a function that can be registered to a route to handle HTTP
/// requests. Like http.HandlerFunc, but has a third parameter for the values of
/// wildcards (variables).
pub type Handle = fn(Request<Body>, Option<Params>) -> Response<Body>;

/// Param is a single URL parameter, consisting of a key and a value.
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

/// Params is a Param-slice, as returned by the router.
/// The slice is ordered, the first URL parameter is also the first slice value.
/// It is therefore safe to read values by the index.
#[derive(Debug, PartialEq)]
pub struct Params(pub Vec<Param>);

impl Params {
    /// ByName returns the value of the first Param which key matches the given name.
    /// If no matching Param is found, an empty string is returned.
    pub fn by_name(&self, name: &str) -> Option<&str> {
        match self.0.iter().find(|param| param.key == name) {
            Some(param) => Some(&param.value),
            None => None,
        }
    }
}

/// Router is a http.Handler which can be used to dispatch requests to different
/// handler functions via configurable routes
pub struct Router<T> {
    pub trees: BTreeMap<String, Node<T>>,
}

impl<T> Router<T> {
    pub fn new() -> Router<T> {
        Router {
            trees: BTreeMap::new(),
        }
    }

    /// get is a shortcut for router.handle("GET", path, handle)
    pub fn get(&mut self, path: &str, handle: T) {
        self.handle("GET", path, handle);
    }

    /// head is a shortcut for router.handle("HEAD", path, handle)
    pub fn head(&mut self, path: &str, handle: T) {
        self.handle("HEAD", path, handle);
    }

    /// options is a shortcut for router.handle("OPTIONS", path, handle)
    pub fn options(&mut self, path: &str, handle: T) {
        self.handle("OPTIONS", path, handle);
    }

    /// post is a shortcut for router.handle("POST", path, handle)
    pub fn post(&mut self, path: &str, handle: T) {
        self.handle("POST", path, handle);
    }

    /// put is a shortcut for router.handle("PUT", path, handle)
    pub fn put(&mut self, path: &str, handle: T) {
        self.handle("PUT", path, handle);
    }

    /// patch is a shortcut for router.handle("PATCH", path, handle)
    pub fn patch(&mut self, path: &str, handle: T) {
        self.handle("PATCH", path, handle);
    }

    /// delete is a shortcut for router.handle("DELETE", path, handle)
    pub fn delete(&mut self, path: &str, handle: T) {
        self.handle("DELETE", path, handle);
    }

    /// Perhaps something like
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// router.group(vec![middelware], |router| {
    ///     router.get("/something", somewhere);
    ///     router.post("/something", somewhere);
    /// })
    /// ```
    pub fn group() {}

    /// ServeFiles serves files from the given file system root.
    /// The path must end with "/*filepath", files are then served from the local
    /// path /defined/root/dir/*filepath.
    /// For example if root is "/etc" and *filepath is "passwd", the local file
    /// "/etc/passwd" would be served.
    /// Internally a http.FileServer is used, therefore http.NotFound is used instead
    /// of the Router's NotFound handler.
    /// To use the operating system's file system implementation,
    /// use http.Dir:
    ///     router.serve_files("/src/*filepath", http.Dir("/var/www"))
    pub fn serve_files(&mut self, path: &str) {
        if path.as_bytes().len() < 10 || &path[path.len() - 10..] != "/*filepath" {
            panic!("path must end with /*filepath in path '{}'", path);
        }
    }

    /// Handle registers a new request handle with the given path and method.
    ///
    /// For GET, POST, PUT, PATCH and DELETE requests the respective shortcut
    /// functions can be used.
    ///
    /// This function is intended for bulk loading and to allow the usage of less
    /// frequently used, non-standardized or custom methods (e.g. for internal
    /// communication with a proxy).
    pub fn handle(&mut self, method: &str, path: &str, handle: T) {
        if !path.starts_with("/") {
            panic!("path must begin with '/' in path '{}'", path);
        }

        self.trees
            .entry(method.to_string())
            .or_insert(Node::new())
            .add_route(path, handle);
    }

    /// Lookup allows the manual lookup of a method + path combo.
    /// This is e.g. useful to build a framework around this router.
    /// If the path was found, it returns the handle function and the path parameter
    /// values. Otherwise the third return value indicates whether a redirection to
    /// the same path with an extra / without the trailing slash should be performed.
    pub fn lookup(&mut self, method: &str, path: &str) -> (Option<&T>, Option<Params>, bool) {
        self.trees
            .get_mut(method)
            .and_then(|n| Some(n.get_value(path)))
            .unwrap_or((None, None, false))
    }
}

/// Service makes the router implement the router.handler interface.
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
            /// Handle 404
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
        /// use http::Response;
        use hyper::{Body, Request, Response};
        use router::Router;

        let path = "something";
        let mut router = Router::new();

        router.handle("GET", path, |_req: Request<Body>| {
            Response::new(Body::from("test"))
        });
    }
}
