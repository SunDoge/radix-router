use router::{Handle, Param, Params};
use std::cell::RefCell;
use std::rc::Rc;
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
pub struct Node {
    path: Vec<u8>,
    wild_child: bool,
    n_type: NodeType,
    max_params: u8,
    indices: Vec<u8>,
    children: Vec<Box<Node>>,
    handle: Option<Handle>,
    priority: u32,
}

impl Node {
    pub fn new() -> Node {
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
            self.insert_child(num_params, path, full_path, handle);
            self.n_type = NodeType::Root;
        }
    }

    fn insert_child(&mut self, mut num_params: u8, path: &[u8], full_path: &[u8], handle: Handle) {}

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
