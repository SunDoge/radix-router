use router::{Param, Params};
use std::fmt::Debug;
use std::mem;
use std::str;

fn min(a: usize, b: usize) -> usize {
    if a <= b {
        return a;
    }
    b
}

fn count_params(path: &[u8]) -> u8 {
    let mut n = 0;
    for &c in path {
        if c != b':' && c != b'*' {
            continue;
        }
        n += 1;
    }
    if n > 255 {
        return 255;
    }
    n as u8
}

#[derive(PartialEq, Clone, Debug)]
pub enum NodeType {
    Static,
    Root,
    Param,
    CatchAll,
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    path: Vec<u8>,
    wild_child: bool,
    n_type: NodeType,
    max_params: u8,
    indices: Vec<u8>,
    children: Vec<Box<Node<T>>>,
    handle: Option<T>,
    priority: u32,
}

impl<T> Node<T> {
    pub fn new() -> Node<T> {
        Node {
            path: Vec::new(),
            wild_child: false,
            n_type: NodeType::Static,
            max_params: 0,
            indices: Vec::new(),
            children: Vec::new(),
            handle: None,
            priority: 0,
        }
    }

    fn increment_child_prio(&mut self, pos: usize) -> usize {
        self.children[pos].priority += 1;
        let prio = self.children[pos].priority;
        let mut new_pos = pos;

        while new_pos > 0 && self.children[new_pos - 1].priority < prio {
            self.children.swap(new_pos - 1, new_pos);
            new_pos -= 1;
        }

        if new_pos != pos {
            self.indices = [
                &self.indices[..new_pos],
                &self.indices[pos..pos + 1],
                &self.indices[new_pos..pos],
                &self.indices[pos + 1..],
            ].concat();
        }

        new_pos
    }

    pub fn add_route(&mut self, path: &str, handle: T) {
        let full_path = path.clone();
        let path = path.as_ref();
        self.priority += 1;
        let mut num_params = count_params(path);
        if self.path.len() > 0 || self.children.len() > 0 {
            self.add_route_loop(num_params, path, full_path, handle);
        } else {
            // Empty tree
            self.insert_child(num_params, path, full_path, handle);
            self.n_type = NodeType::Root;
        }
    }

    fn add_route_loop(&mut self, num_params: u8, path: &[u8], full_path: &str, handle: T) {
        if num_params > self.max_params {
            self.max_params = num_params;
        }

        let mut i = 0;
        let max = min(path.len(), self.path.len());

        while i < max && path[i] == self.path[i] {
            i += 1;
        }

        if i < self.path.len() {
            let mut child = Node {
                path: self.path[i..].to_vec(),
                wild_child: self.wild_child,
                n_type: NodeType::Static,
                indices: self.indices.clone(),
                children: Vec::new(),
                handle: self.handle.take(),
                priority: self.priority - 1,

                max_params: 0,
            };

            mem::swap(&mut self.children, &mut child.children);

            for c in &child.children {
                if c.max_params > child.max_params {
                    child.max_params = c.max_params;
                }
            }

            self.children = vec![Box::new(child)];
            self.indices = vec![self.path[i]];
            self.path = path[..i].to_vec();
            self.wild_child = false;
        }

        if i < path.len() {
            let path = &path[i..];

            if self.wild_child {
                // *n = * {n}.children[0].clone();
                return self.children[0].is_wild_child(num_params, path, full_path, handle);
            }

            let c = path[0];

            if self.n_type == NodeType::Param && c == b'/' && self.children.len() == 1 {
                self.children[0].priority += 1;
                return self.children[0].add_route_loop(num_params, path, full_path, handle);
            }

            for mut i in 0..self.indices.len() {
                if c == self.indices[i] {
                    i = self.increment_child_prio(i);
                    return self.children[i].add_route_loop(num_params, path, full_path, handle);
                }
            }

            // Otherwise insert it

            if c != b':' && c != b'*' {
                self.indices.push(c);

                let len = self.indices.len();

                let child: Box<Node<T>> = Box::new(Node {
                    path: Vec::new(),

                    wild_child: false,

                    n_type: NodeType::Static,

                    max_params: num_params,

                    indices: Vec::new(),

                    children: Vec::new(),

                    handle: None,

                    priority: 0,
                });

                self.children.push(child);

                let i = self.increment_child_prio(len - 1);

                return self.children[i].insert_child(num_params, path, full_path, handle);
            }

            return self.insert_child(num_params, path, full_path, handle);
        } else if i == path.len() {
            if self.handle.is_some() {
                panic!("a handle is already registered for path '{}'", full_path);
            }

            self.handle = Some(handle);
        }

        return;
    }

    fn is_wild_child(&mut self, mut num_params: u8, path: &[u8], full_path: &str, handle: T) {
        self.priority += 1;

        // Update maxParams of the child node

        if num_params > self.max_params {
            self.max_params = num_params;
        }

        num_params -= 1;

        // Check if the wildcard matches

        if path.len() >= self.path.len() && self.path == &path[..self.path.len()]
            && (self.path.len() >= path.len() || path[self.path.len()] == b'/')
        {
            self.add_route_loop(num_params, path, full_path, handle);
        } else {
            // Wildcard conflict

            let path_seg = if self.n_type == NodeType::CatchAll {
                str::from_utf8(path).unwrap()
            } else {
                str::from_utf8(path)
                    .unwrap()
                    .splitn(2, '/')
                    .into_iter()
                    .next()
                    .unwrap()
            };

            let prefix = [
                &full_path[..full_path.find(path_seg).unwrap()],
                str::from_utf8(&self.path).unwrap(),
            ].concat();

            panic!("'{}' in new path '{}' conflicts with existing wildcard '{}' in existing prefix '{}'", path_seg, full_path, str::from_utf8(&self.path).unwrap(), prefix);
        }
    }

    fn insert_child(&mut self, mut num_params: u8, path: &[u8], full_path: &str, handle: T) {
        self.insert_child_loop(0, 0, num_params, path, full_path, handle);
    }

    fn insert_child_loop(
        &mut self,
        mut offset: usize,
        mut i: usize,
        mut num_params: u8,
        path: &[u8],
        full_path: &str,
        handle: T,
    ) {
        if num_params > 0 {
            let max = path.len();
            let c = path[i];

            if c != b':' && c != b'*' {
                return self.insert_child_loop(offset, i + 1, num_params, path, full_path, handle);
            }

            let mut end = i + 1;
            while end < max && path[end] != b'/' {
                match path[end] {
                    b':' | b'*' => panic!(
                        "only one wildcard per path segment is allowed, has: '{}' in path '{}'",
                        str::from_utf8(&path[i..]).unwrap(),
                        full_path
                    ),
                    _ => end += 1,
                }
            }

            // println!("self path: {}", str::from_utf8(&self.path).unwrap());
            // println!("temp path: {}", str::from_utf8(path).unwrap());
            // println!("self {:?}", self.children[0]);
            // println!("self {:?}", self.children.len());

            // check if this Node existing children which would be
            // unreachable if we insert the wildcard here
            if self.children.len() > 0 {
                panic!(
                    "wildcard route '{}' conflicts with existing children in path '{}'",
                    str::from_utf8(&path[i..end]).unwrap(),
                    full_path
                )
            }

            // check if the wildcard has a name
            if end - i < 2 {
                panic!(
                    "wildcards must be named with a non-empty name in path '{}'",
                    full_path
                );
            }

            if c == b':' {
                // Param
                if i > 0 {
                    self.path = path[offset..i].to_vec();
                    offset = i;
                }

                let child = Box::new(Node {
                    path: Vec::new(),
                    wild_child: false,
                    n_type: NodeType::Param,
                    max_params: num_params,
                    indices: Vec::new(),
                    children: Vec::new(),
                    handle: None,
                    priority: 0,
                });

                self.children = vec![child];
                self.wild_child = true;

                self.children[0].priority += 1;
                num_params -= 1;

                if end < max {
                    self.children[0].path = path[offset..end].to_vec();
                    offset = end;

                    let child: Box<Node<T>> = Box::new(Node {
                        path: Vec::new(),
                        wild_child: false,
                        n_type: NodeType::Static,
                        max_params: num_params,
                        indices: Vec::new(),
                        children: Vec::new(),
                        handle: None,
                        priority: 1,
                    });

                    self.children[0].children.push(child);
                    self.children[0].children[0].insert_child_loop(
                        offset,
                        i + 1,
                        num_params,
                        path,
                        full_path,
                        handle,
                    );
                } else {
                    self.children[0].insert_child_loop(
                        offset,
                        i + 1,
                        num_params,
                        path,
                        full_path,
                        handle,
                    );
                }
            } else {
                // CatchAll
                if end != max || num_params > 1 {
                    panic!(
                        "catch-all routes are only allowed at the end of the path in path '{}'",
                        full_path
                    );
                }

                if self.path.len() > 0 && self.path[self.path.len() - 1] == b'/' {
                    panic!(
                        "catch-all conflicts with existing handle for the path segment root in path '{}'", 
                        full_path
                    );
                }

                // currently fixed width 1 for '/'
                i -= 1;
                if path[i] != b'/' {
                    panic!("no / before catch-all in path '{}'", full_path);
                }

                self.path = path[offset..i].to_vec();

                // first node: catchAll node with empty path
                let child = Box::new(Node {
                    path: Vec::new(),
                    wild_child: true,
                    n_type: NodeType::CatchAll,
                    max_params: 1,
                    indices: Vec::new(),
                    children: Vec::new(),
                    handle: None,
                    priority: 1,
                });

                self.children = vec![child];

                self.indices = vec![path[i]];

                self.children[0].priority += 1;

                let child: Box<Node<T>> = Box::new(Node {
                    path: path[i..].to_vec(),
                    wild_child: false,
                    n_type: NodeType::CatchAll,
                    max_params: 1,
                    indices: Vec::new(),
                    children: Vec::new(),
                    handle: Some(handle),
                    priority: 1,
                });

                self.children[0].children.push(child);

                return;
            }
        } else {
            // insert remaining path part and handle to the leaf
            self.path = path[offset..].to_vec();
            self.handle = Some(handle);
        }
    }
    pub fn get_value(&mut self, path: &str) -> (Option<&T>, Option<Params>, bool) {
        // let mut handle = None;
        let mut p = None;

        self.get_value_loop(path.as_ref(), p)
    }

    fn get_value_loop(
        &mut self,
        mut path: &[u8],

        mut p: Option<Params>,
    ) -> (Option<&T>, Option<Params>, bool) {
        if path.len() > self.path.len() {
            if self.path == &path[..self.path.len()] {
                path = &path[self.path.len()..];
                if !self.wild_child {
                    let c = path[0];
                    for i in 0..self.indices.len() {
                        if c == self.indices[i] {
                            return self.children[i].get_value_loop(path, p);
                        }
                    }

                    let tsr = path == [b'/'] && self.handle.is_some();
                    return (None, p, tsr);
                }

                return self.children[0].handle_wildcard_child(path, p);
            }
        } else if self.path == path {
            if self.handle.is_some() {
                return (self.handle.as_ref(), p, false);
            }

            if path == [b'/'] && self.wild_child && self.n_type != NodeType::Root {
                // tsr = true;
                return (self.handle.as_ref(), p, true);
            }

            for i in 0..self.indices.len() {
                if self.indices[i] == b'/' {
                    let tsr = (self.path.len() == 1 && self.children[i].handle.is_some())
                        || (self.children[i].n_type == NodeType::CatchAll
                            && self.children[i].children[0].handle.is_some());
                    return (self.handle.as_ref(), p, tsr);
                }
            }

            return (self.handle.as_ref(), p, false);
        }

        let tsr = (path == [b'/'])
            || (self.path.len() == path.len() + 1 && self.path[path.len()] == b'/'
                && path == &self.path[..self.path.len() - 1]
                && self.handle.is_some());

        return (None, p, tsr);
    }

    fn handle_wildcard_child(
        &mut self,
        mut path: &[u8],
        mut p: Option<Params>,
    ) -> (Option<&T>, Option<Params>, bool) {
        match self.n_type {
            NodeType::Param => {
                let mut end = 0;
                while end < path.len() && path[end] != b'/' {
                    end += 1;
                }

                if p.is_none() {
                    p = Some(Params(Vec::with_capacity(self.max_params as usize)));
                }

                p.as_mut().map(|ps| {
                    ps.0.push(Param {
                        key: String::from_utf8(self.path[1..].to_vec()).unwrap(),
                        value: String::from_utf8(path[..end].to_vec()).unwrap(),
                    });
                });

                if end < path.len() {
                    if self.children.len() > 0 {
                        path = &path[end..];

                        return self.children[0].get_value_loop(path, p);
                    }

                    let tsr = path.len() == end + 1;
                    return (None, p, tsr);
                }

                if self.handle.is_some() {
                    return (self.handle.as_ref(), p, false);
                } else if self.children.len() == 1 {
                    let tsr = self.children[0].path == &[b'/'] && self.children[0].handle.is_some();
                    return (None, p, tsr);
                }

                return (None, p, false);
            }
            NodeType::CatchAll => {
                if p.is_none() {
                    p = Some(Params(Vec::with_capacity(self.max_params as usize)));
                }

                p.as_mut().map(|ps| {
                    ps.0.push(Param {
                        key: String::from_utf8(self.path[2..].to_vec()).unwrap(),
                        value: String::from_utf8(path.to_vec()).unwrap(),
                    });
                });

                return (self.handle.as_ref(), p, false);
            }
            _ => panic!("invalid node type"),
        }
    }
}

#[cfg(test)]
mod tests {

    struct TestRequest {
        path: &str,
        nil_handler: bool,
        route: &str,
        ps: Option<Params>,
    }

    type TestRequests = Vec<TestRequest>;

    fn check_requests<T>(tree: Node<T>, requests: TestRequests) {}

    #[test]
    fn it_works() {
        // use tree::Node;
        // let mut node = Node::new();
    }
}
