use router::Handle;

#[derive(PartialEq)]
pub enum NodeType {
    Static,
    Root,
    Param,
    CatchAll,
}

pub struct Node<T> {
    path: Vec<u8>,
    wild_child: bool,
    n_type: NodeType,
    max_params: u8,
    indices: Vec<u8>,
    children: Vec<Box<Node<T>>>,
    handle: Option<Handle<T>>,
    priority: u32,
}

impl<T> Node<T> {
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

    fn add_route(&mut self, path: &str, handle: Handle<T>) {}

    fn insert_child(&mut self, num_params: u8, path: &str, full_path: &str, handle: Handle<T>) {
        let mut offset: usize;
        let mut i = 0;
        let max = path.as_bytes().len();

        while num_params > 0 {

        }
    }
}
