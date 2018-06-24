use router::{Handle, Param, Params};
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

    pub fn add_route(&mut self, path: &[u8], handle: Handle) {
        let full_path = path.clone();
        self.priority += 1;
        let mut num_params = count_params(path);
        if self.path.len() > 0 || self.children.len() > 0 {

        } else {
            // Empty tree
            self.insert_child(num_params, path, full_path, handle);
            self.n_type = NodeType::Root;
        }
    }

    fn insert_child(&mut self, mut num_params: u8, path: &[u8], full_path: &[u8], handle: Handle) {
        let mut offset = 0;
        let mut i = 0;
        let max = path.len();

        while num_params > 0 {
            let c = path[i];

            if c != b':' && c != b'*' {
                i += 1;
                continue;
            }

            let mut end = i + 1;
            while end < max && path[end] != b'/' {
                match path[end] {
                    b':' | b'*' => panic!(
                        "only one wildcard per path segment is allowed, has: '{}' in path '{}'",
                        str::from_utf8(&path[i..]).unwrap(),
                        str::from_utf8(full_path).unwrap()
                    ),
                    _ => end += 1,
                }
            }

            // check if this Node existing children which would be
            // unreachable if we insert the wildcard here
            if self.children.len() > 0 {
                panic!(
                    "wildcard route '{}' conflicts with existing children in path '{}'",
                    str::from_utf8(&path[i..end]).unwrap(),
                    str::from_utf8(full_path).unwrap(),
                )
            }

            // check if the wildcard has a name
            if end - i < 2 {
                panic!(
                    "wildcards must be named with a non-empty name in path '{}'",
                    str::from_utf8(full_path).unwrap(),
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

            // TODO
            } else {
                // CatchAll
                if end != max || num_params > 1 {
                    panic!(
                        "catch-all routes are only allowed at the end of the path in path '{}'",
                        str::from_utf8(full_path).unwrap()
                    );
                }

                if self.path.len() > 0 && self.path[self.path.len() - 1] == b'/' {
                    panic!(
                        "catch-all conflicts with existing handle for the path segment root in path '{}'", 
                        str::from_utf8(full_path).unwrap()
                    );
                }

                // currently fixed width 1 for '/'
                i -= 1;
                if path[i] != b'/' {
                    panic!(
                        "no / before catch-all in path '{}'",
                        str::from_utf8(full_path).unwrap()
                    );
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

                // TODO
            }
        }

        // insert remaining path part and handle to the leaf
        self.path = path[offset..].to_vec();
        self.handle = Some(handle);
    }

    fn is_wildchild(&mut self, mut num_params: u8, path: &[u8], full_path: &[u8], handle: Handle) {
        self.priority += 1;

        // Update maxParams of the child node
        if num_params > self.max_params {
            self.max_params = num_params;
        }
        num_params -= 1;

        // Check if the wildcard matches
        if path.len() >= self.path.len()
            && self.path == &path[..self.path.len()]
            && (self.path.len() >= path.len() || path[self.path.len()] == b'/')
        {
            // continue 'walk;
            self.add_route_walk_loop(num_params, path, full_path, handle);
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
            let full_path = str::from_utf8(full_path).unwrap();
            let self_path = str::from_utf8(&self.path).unwrap();
            let prefix = [
                &full_path[..full_path.find(path_seg).unwrap()],
                // str::from_utf8(&self.path).unwrap(),
                &self_path,
            ].concat();
            panic!("'{}' in new path '{}' conflicts with existing wildcard '{}' in existing prefix '{}'", path_seg, full_path, self_path, prefix);
        }
    }

    #[allow(dead_code)]
    fn add_route_walk_loop(
        &mut self,
        num_params: u8,
        path: &[u8],
        full_path: &[u8],
        handle: Handle,
    ) {
        // Update maxParams of the current node
        if num_params > self.max_params {
            self.max_params = num_params;
        }

        // Find the longest common prefix.
        // This also implies that the common prefix contains no ':' or '*'
        // since the existing key can't contain those chars.
        let mut i = 0;
        let max = min(path.len(), self.path.len());
        while i < max && path[i] == self.path[i] {
            i += 1;
        }

        // Split edge
        if i < self.path.len() {
            let mut child = Node {
                path: self.path[i..].to_vec(),
                wild_child: self.wild_child,
                n_type: NodeType::Static,
                max_params: 0,
                indices: self.indices.clone(),
                children: self.children.clone(),
                handle: self.handle,
                priority: self.priority - 1,
            };

            for c in &child.children {
                if c.max_params > child.max_params {
                    child.max_params = c.max_params;
                }
            }

            self.children = vec![Box::new(child)];
            self.indices = vec![self.path[i]];
            self.path = path[..i].to_vec();
            self.handle = None;
            self.wild_child = false;
        }

        // Make new node a child of this node
        if i < path.len() {
            let path = &path[i..];
            if self.wild_child {
                // TODO
                self.children[0].is_wildchild(num_params, path, full_path, handle);
            }
        } else if i == path.len() {
            if self.handle.is_some() {
                panic!(
                    "a handle is already registered for path '{}'",
                    str::from_utf8(full_path).unwrap()
                );
            }
            self.handle = Some(handle);
        }

        return;
    }

    fn slash_after_param(&mut self, num_params: u8, path: &[u8], full_path: &[u8], handle: Handle) {
        self.priority += 1;
        self.add_route_walk_loop(num_params, path, full_path, handle);
    }

    pub fn get_value(&mut self, path: &[u8]) -> (Option<Handle>, Option<Params>, bool) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use tree::Node;
        let mut node = Node::new();
    }
}
