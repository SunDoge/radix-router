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
pub struct Node<T>
where
    T: Clone,
{
    path: Vec<u8>,
    wild_child: bool,
    n_type: NodeType,
    max_params: u8,
    indices: Vec<u8>,
    children: Vec<Box<Node<T>>>,
    handle: Option<Handle<T>>,
    priority: u32,
}

impl<T> Node<T>
where
    T: Clone,
{
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

    pub fn add_route(mut self: Box<Self>, path: &str, handle: Handle<T>) {
        let full_path = path.clone();

        self.priority += 1;
        let path = path.as_bytes();
        let mut num_params = count_params(path);

        if self.path.len() > 0 || self.children.len() > 0 {
            'walk: loop {
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
                        self = self.children.into_iter().next().unwrap();
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
                            continue 'walk;
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

                    let c = path[0];

                    // slash after param
                    if self.n_type == NodeType::Param && c == b'/' && self.children.len() == 1 {
                        self = self.children.into_iter().next().unwrap();
                        self.priority += 1;
                        continue 'walk;
                    }

                    // Check if a child with the next path byte exists
                    for mut i in 0..self.indices.len() - 1 {
                        if c == self.indices[i] {
                            i = self.increment_child_prio(i);
                            self = self.children.into_iter().nth(i).unwrap();
                            continue 'walk;
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
                        self.increment_child_prio(len - 1);
                        self = self.children.into_iter().next().unwrap();
                    }

                    self.insert_child(num_params, path, full_path, handle);
                    return;
                } else if i == path.len() {
                    if self.handle.is_some() {
                        panic!("a handle is already registered for path '{}'", full_path);
                    }
                    self.handle = Some(handle);
                }

                return;
            }
        } else {
            // Empty tree
            self.n_type = NodeType::Root;
            self.insert_child(num_params, path, full_path, handle);
        }
    }

    fn insert_child(
        mut self: Box<Self>,
        mut num_params: u8,
        path: &[u8],
        full_path: &str,
        handle: Handle<T>,
    ) {
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
                        full_path
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
                    full_path,
                )
            }

            // check if the wildcard has a name
            if end - i < 2 {
                panic!(
                    "wildcards must be named with a non-empty name in path '{}'",
                    full_path,
                );
            }

            if c == b':' {
                // param
                // split path at the beginning of the wildcard
                if i > 0 {
                    self.path = path[offset..i].to_vec();
                    offset = i;
                }

                let child: Box<Node<T>> = Box::new(Node {
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
                self = self.children.into_iter().next().unwrap();
                self.priority += 1;
                num_params -= 1;

                // if the path doesn't end with the wildcard, then there
                // will be another non-wildcard subpath starting with '/'

                if end < max {
                    self.path = path[offset..end].to_vec();
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

                    self.children = vec![child];

                    self = self.children.into_iter().next().unwrap();
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
                    panic!("catch-all conflicts with existing handle for the path segment root in path '{}'", full_path);
                }

                // currently fixed width 1 for '/'
                i -= 1;
                if path[i] != b'/' {
                    panic!("no / before catch-all in path '{}'", full_path);
                }

                self.path = path[offset..i].to_vec();

                // first node: catchAll node with empty path
                let child: Box<Node<T>> = Box::new(Node {
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
                self = self.children.into_iter().next().unwrap();
                self.priority += 1;

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

                self.children = vec![child];
                return;
            }
        }

        // insert remaining path part and handle to the leaf
        self.path = path[offset..].to_vec();
        self.handle = Some(handle);
    }

    pub fn get_value(mut self: Box<Self>, path: &str) -> (Option<Handle<T>>, Option<Params>, bool) {
        let mut path = path.as_bytes();
        let mut tsr = false;
        let mut p = None;
        let mut handle = None;
        'walk: loop {
            if path.len() > self.path.len() {
                if self.path == &path[..self.path.len()] {
                    path = &path[self.path.len()..];
                    // If this node does not have a wildcard (param or catchAll)
                    // child,  we can just look up the next child node and continue
                    // to walk down the tree
                    if !self.wild_child {
                        let c = path[0];
                        for i in 0..self.indices.len() - 1 {
                            if c == self.indices[i] {
                                self = self.children.into_iter().nth(i).unwrap();
                                continue 'walk;
                            }
                        }

                        // Nothing found.
                        // We can recommend to redirect to the same URL without a
                        // trailing slash if a leaf exists for that path.
                        tsr = (path == [b'/'] && self.handle.is_some());
                        return (None, p, tsr);
                    }

                    // handle wildcard child
                    self = self.children.into_iter().next().unwrap();

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
                            // let i = p.0.len();
                            // p.0.resize(
                            //     i + 1,
                            //     Param {
                            //         key: String::from_utf8(self.path[1..].to_vec()).unwrap(),
                            //         value: String::from_utf8(path[..end].to_vec()).unwrap(),
                            //     },
                            // );

                            // we need to go deeper!
                            if end < path.len() {
                                if self.children.len() > 0 {
                                    path = &path[end..];
                                    self = self.children.into_iter().next().unwrap();
                                    continue 'walk;
                                }

                                // ... but we can't
                                tsr = (path.len() == end + 1);
                                return (handle, p, tsr);
                            }

                            if let Some(handle) = self.handle {
                                return (Some(handle), p, tsr);
                            } else if self.children.len() == 1 {
                                self = self.children.into_iter().next().unwrap();
                                tsr = (self.path == [b'/'] && self.handle.is_some());
                            }

                            return (handle, p, tsr);
                        }
                        NodeType::CatchAll => {
                            if p.is_none() {
                                p = Some(Params(Vec::with_capacity(self.max_params as usize)));
                            }
                            // let i = p.0.len();
                            // p.0.resize(
                            //     i + 1,
                            //     Param {
                            //         key: String::from_utf8(self.path[2..].to_vec()).unwrap(),
                            //         value: String::from_utf8(path.to_vec()).unwrap(),
                            //     },
                            // );
                            p.as_mut().map(|ps| {
                                ps.0.push(Param {
                                    key: String::from_utf8(self.path[2..].to_vec()).unwrap(),
                                    value: String::from_utf8(path.to_vec()).unwrap(),
                                });
                            });

                            return (self.handle, p, tsr);
                        }
                        _ => panic!("invalid node type"),
                    }
                }
            } else if self.path == path {
                if let Some(handle) = self.handle {
                    return (Some(handle), p, tsr);
                }

                if path == [b'/'] && self.wild_child && self.n_type != NodeType::Root {
                    tsr = true;
                    return (self.handle, p, tsr);
                }

                for i in 0..self.indices.len() {
                    if self.indices[i] == b'/' {
                        self = self.children.into_iter().nth(i).unwrap();
                        tsr = (self.path.len() == 1 && self.handle.is_some())
                            || (self.n_type == NodeType::CatchAll
                                && self.children[0].handle.is_some());
                        return (handle, p, tsr);
                    }
                }

                return (handle, p, tsr);
            }

            tsr = (path == [b'/'])
                || (self.path.len() == path.len() + 1 && self.path[path.len()] == b'/'
                    && path == &self.path[..self.path.len() - 1]
                    && self.handle.is_some());

            return (handle, p, tsr);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use tree::Node;
        let mut node: Node<()> = Node::new();
    }
}
