use router::{Handle, Param, Params};
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
    handle: Option<T>,

    wild_child: bool,
    n_type: NodeType,
    max_params: u8,
    priority: u32,

    child: Option<Box<Node<T>>>,
    sibling: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new<P: Into<Vec<u8>>>(path: P, handle: T) -> Node<T> {
        Node {
            path: path.into(),
            handle: Some(handle),

            wild_child: false,
            n_type: NodeType::Static,
            max_params: 0,
            priority: 0,

            child: None,
            sibling: None,
        }
    }

    pub fn increment_child_prio(&mut self, pos: usize) -> usize {
        if pos > 0 {
            if let Some(ref mut sibling) = self.sibling {
                let new_pos = sibling.increment_child_prio(pos - 1);
                if self.priority < sibling.priority {
                    mem::swap(&mut self.path, &mut sibling.path);
                    mem::swap(&mut self.handle, &mut sibling.handle);
                    mem::swap(&mut self.n_type, &mut sibling.n_type);
                    mem::swap(&mut self.max_params, &mut sibling.max_params);
                    mem::swap(&mut self.wild_child, &mut sibling.wild_child);
                    mem::swap(&mut self.priority, &mut sibling.priority);
                    mem::swap(&mut self.child, &mut sibling.child);
                    return new_pos;
                } else {
                    return new_pos + 1;
                }
            } else {
                panic!("Out of range.");
            }
        } else {
            // == 0
            self.priority += 1;
            return pos;
        }
    }

    pub fn insert_child<P: AsRef<[u8]>>(
        &mut self,
        num_params: u8,
        path: P,
        full_path: P,
        handle: T,
    ) {
        let path = path.as_ref();
        let full_path = full_path.as_ref();
        let prefix = self.common_prefix(path);
        if prefix == 0 {
            match self.sibling {
                Some(ref mut sibling) => sibling.insert_child(num_params, path, full_path, handle),
                _ => self.sibling = Some(Box::new(Node::new(path, handle))),
            }
        } else if prefix < path.len() {
            if prefix < self.path.len() {
                self.child = Some(Box::new(Node {
                    path: self.path.split_off(prefix),
                    handle: self.handle.take(),

                    max_params: self.max_params,
                    n_type: self.n_type.clone(),
                    priority: self.priority,
                    wild_child: self.wild_child,

                    child: self.child.take(),
                    sibling: None,
                }));
                self.path.shrink_to_fit()
            }
            match self.child {
                Some(ref mut child) => {
                    child.insert_child(num_params, &path[prefix..], full_path, handle)
                }
                _ => self.child = Some(Box::new(Node::new(&path[prefix..], handle))),
            }
        }
    }

    fn common_prefix<K: AsRef<[u8]>>(&self, other: K) -> usize {
        self.path
            .iter()
            .zip(other.as_ref().into_iter())
            .take_while(|&(a, b)| a == b)
            .count()
    }

    pub fn add_route<P: AsRef<[u8]>>(&mut self, path: P, handle: T) {
        
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use tree::Node;
        // let mut node: Node = Node::new();
    }
}
