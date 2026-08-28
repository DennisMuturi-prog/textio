pub struct PieceTree {
    original: String,
    add: String,
    nodes: Vec<Node>,
    root_node: Option<usize>,
}

struct Node {
    #[allow(dead_code)]
    index: usize,
    buffer_type: BufferType,
    color: Color,
    start: usize,
    length: usize,
    subtree_length: usize,
    left_subtree_length: usize,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
}
impl Node {
    fn new(index: usize, buffer_type: BufferType, start: usize, length: usize) -> Self {
        Node {
            index,
            buffer_type,
            color: Color::Red,
            start,
            length,
            subtree_length: length,
            left_subtree_length: 0,
            left: None,
            right: None,
            parent: None,
        }
    }
}

impl PieceTree {
    pub fn new(original: &str) -> Self {
        if !original.is_empty() {
            let mut root = Node::new(0, BufferType::Original, 0, original.len());
            root.color = Color::Black;
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: vec![root],
                root_node: Some(0),
            }
        } else {
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: Vec::new(),
                root_node: None,
            }
        }
    }
    pub fn get_text(&self) -> String {
        let mut result = String::new();
        self.in_order(self.root_node, &mut result);
        result
    }

    fn in_order(&self, node_idx: Option<usize>, out: &mut String) {
        if let Some(idx) = node_idx {
            let node = &self.nodes[idx];
            self.in_order(node.left, out);

            let buffer = match node.buffer_type {
                BufferType::Original => &self.original,
                BufferType::Add => &self.add,
            };
            out.push_str(&buffer[node.start..node.start + node.length]);

            self.in_order(node.right, out);
        }
    }
    fn update_subtree_length(&mut self, node_idx: usize) {
        let node = &self.nodes[node_idx];
        let left_len = node.left.map_or(0, |l| self.nodes[l].subtree_length);
        let right_len = node.right.map_or(0, |r| self.nodes[r].subtree_length);
        self.nodes[node_idx].subtree_length = left_len + right_len + self.nodes[node_idx].length;
        self.nodes[node_idx].left_subtree_length = left_len;
    }

    fn left_rotate(&mut self, x: usize) {
        let y = match self.nodes[x].right {
            Some(index) => index,
            None => return,
        };
        let y_left = self.nodes[y].left;
        self.nodes[x].right = y_left;
        if let Some(y_l) = y_left {
            self.nodes[y_l].parent = Some(x);
        }
        let x_parent = self.nodes[x].parent;
        self.nodes[y].parent = x_parent;
        match x_parent {
            Some(parent) => {
                if self.nodes[parent].left == Some(x) {
                    self.nodes[parent].left = Some(y);
                } else if self.nodes[parent].right == Some(x) {
                    self.nodes[parent].right = Some(y);
                } else {
                    if self.nodes[parent].left.is_none() && self.nodes[parent].right.is_none() {
                        panic!("a parent cannot have empty children {parent}")
                    } else {
                        panic!(
                            "indices mismatch  parent {parent} left {:?} right {:?} x {x}",
                            self.nodes[parent].left, self.nodes[parent].right
                        )
                    }
                }
            }
            None => {
                self.root_node = Some(y);
            }
        }
        self.nodes[y].left = Some(x);
        self.nodes[x].parent = Some(y);
        self.update_subtree_length(x);
        self.update_subtree_length(y);
    }
    fn right_rotate(&mut self, y: usize) {
        let x = match self.nodes[y].left {
            Some(index) => index,
            None => return,
        };
        let x_right = self.nodes[x].right;
        self.nodes[y].left = x_right;
        if let Some(x_r) = x_right {
            self.nodes[x_r].parent = Some(y);
        }
        let y_parent = self.nodes[y].parent;
        self.nodes[x].parent = y_parent;
        match y_parent {
            Some(parent) => {
                if self.nodes[parent].left == Some(y) {
                    self.nodes[parent].left = Some(x);
                } else if self.nodes[parent].right == Some(y) {
                    self.nodes[parent].right = Some(x);
                } else {
                    if self.nodes[parent].left.is_none() && self.nodes[parent].right.is_none() {
                        panic!("a parent cannot have empty children {parent}")
                    } else {
                        panic!(
                            "indices mismatch  parent {parent} left {:?} right {:?} y {y}",
                            self.nodes[parent].left, self.nodes[parent].right
                        )
                    }
                }
            }
            None => {
                self.root_node = Some(x);
            }
        }
        self.nodes[x].right = Some(y);
        self.nodes[y].parent = Some(x);
        self.update_subtree_length(y);
        self.update_subtree_length(x);
    }
    fn pre_insert(&mut self, content: &str) -> usize {
        match self.root_node {
            Some(_) => {
                let previous_buffer_len = self.add.len();

                self.add.push_str(content);
                let previous_nodes_len = self.nodes.len();
                self.nodes.push(Node::new(
                    previous_nodes_len,
                    BufferType::Add,
                    previous_buffer_len,
                    content.len(),
                ));
                previous_nodes_len
            }
            None => {
                let previous_buffer_len = self.original.len();

                self.original.push_str(content);
                let previous_nodes_len = self.nodes.len();
                self.nodes.push(Node::new(
                    previous_nodes_len,
                    BufferType::Original,
                    previous_buffer_len,
                    content.len(),
                ));
                previous_nodes_len
            }
        }
    }
    fn insert_node_after(&mut self, target_node: usize, new_node: usize) {
        // Reset new_node connections
        self.nodes[new_node].left = None;
        self.nodes[new_node].right = None;
        self.nodes[new_node].color = Color::Red;

        if self.nodes[target_node].right.is_none() {
            // Case 1: Target has no right child, attach directly as right child
            self.nodes[target_node].right = Some(new_node);
            self.nodes[new_node].parent = Some(target_node);
        } else {
            // Case 2: Target has a right child, attach as leftmost child of the right subtree
            let mut curr = self.nodes[target_node].right.unwrap();
            while let Some(left) = self.nodes[curr].left {
                curr = left;
            }
            self.nodes[curr].left = Some(new_node);
            self.nodes[new_node].parent = Some(curr);
        }

        // 1. Update subtree lengths from new_node's parent up to root
        self.update_ancestors(self.nodes[new_node].parent);

        // 2. Fix red-black tree invariants (rotations & recoloring)
        self.insert_fix(new_node);
    }

    /// Walk up the parent chain and update lengths
    fn update_ancestors(&mut self, mut curr: Option<usize>) {
        while let Some(node_idx) = curr {
            self.update_subtree_length(node_idx);
            curr = self.nodes[node_idx].parent;
        }
    }

    fn insert_fix(&mut self, current_node: usize) {
        let mut current_node = current_node;
        while Some(current_node) != self.root_node
            && self.nodes[self.nodes[current_node].parent.unwrap()].color == Color::Red
        {
            let mut parent = match self.nodes[current_node].parent {
                Some(p) => p,
                None => {
                    break;
                }
            };
            let mut grand_parent = match self.nodes[parent].parent {
                Some(p) => p,
                None => {
                    break;
                }
            };
            if self.nodes[grand_parent].left == Some(parent) {
                if let Some(right) = self.nodes[grand_parent].right
                    && self.nodes[right].color == Color::Red
                {
                    self.nodes[right].color = Color::Black;
                    if let Some(left) = self.nodes[grand_parent].left {
                        self.nodes[left].color = Color::Black;
                    };
                    self.nodes[grand_parent].color = Color::Red;
                    current_node = grand_parent;
                } else {
                    if self.nodes[parent].right == Some(current_node) {
                        current_node = parent;
                        self.left_rotate(current_node);
                        parent = match self.nodes[current_node].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                        grand_parent = match self.nodes[parent].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                    }
                    self.nodes[parent].color = Color::Black;

                    self.nodes[grand_parent].color = Color::Red;
                    self.right_rotate(grand_parent);
                }
            } else {
                if let Some(left) = self.nodes[grand_parent].left
                    && self.nodes[left].color == Color::Red
                {
                    self.nodes[left].color = Color::Black;
                    if let Some(right) = self.nodes[grand_parent].right {
                        self.nodes[right].color = Color::Black;
                    };
                    self.nodes[grand_parent].color = Color::Red;
                    current_node = grand_parent;
                } else {
                    if self.nodes[parent].left == Some(current_node) {
                        current_node = parent;
                        self.right_rotate(current_node);
                        parent = match self.nodes[current_node].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                        grand_parent = match self.nodes[parent].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                    }

                    self.nodes[parent].color = Color::Black;

                    self.nodes[grand_parent].color = Color::Red;
                    self.left_rotate(grand_parent);
                }
            }
        }
        if let Some(root) = self.root_node {
            self.nodes[root].color = Color::Black;
        }
    }

    fn split_and_insert(&mut self, index: usize, curr_node_idx: usize, new_node: usize) {
        let offset_in_node = index - self.nodes[curr_node_idx].left_subtree_length;

        if offset_in_node > 0 && offset_in_node < self.nodes[curr_node_idx].length {
            let right_start = self.nodes[curr_node_idx].start + offset_in_node;
            let right_len = self.nodes[curr_node_idx].length - offset_in_node;
            let buffer_type = self.nodes[curr_node_idx].buffer_type;

            // 1. Shorten left piece in-place
            self.nodes[curr_node_idx].length = offset_in_node;
            self.update_subtree_length(curr_node_idx);

            // 2. Create right piece
            let right_node_idx = self.nodes.len();
            self.nodes.push(Node::new(
                right_node_idx,
                buffer_type,
                right_start,
                right_len,
            ));

            // 3. Insert new_node and right_node as consecutive successors in the tree
            self.insert_node_after(curr_node_idx, new_node);
            self.insert_node_after(new_node, right_node_idx);
        }
    }
    pub fn insert(&mut self, index: usize, content: &str) {
        if content.is_empty() {
            return;
        }
        let new_node = self.pre_insert(content);
        self.insert_into_tree(new_node, index);
    }
    fn insert_into_tree(&mut self, new_node: usize, mut index: usize) {
        let root = match self.root_node {
            Some(r) => r,
            None => {
                // Empty tree: new_node becomes the black root
                self.nodes[new_node].color = Color::Black;
                self.root_node = Some(new_node);
                return;
            }
        };

        // Case 1: Inserting at or past the end of the entire document
        if index >= self.nodes[root].subtree_length {
            let mut rightmost = root;
            while let Some(r) = self.nodes[rightmost].right {
                rightmost = r;
            }
            self.insert_node_after(rightmost, new_node);
            return;
        }

        // Case 2: Traverse to find the exact piece containing `index`
        let mut curr = root;
        loop {
            let left_len = self.nodes[curr].left_subtree_length;
            let node_len = self.nodes[curr].length;

            if index < left_len {
                curr = self.nodes[curr].left.unwrap();
            } else if index >= left_len + node_len {
                index -= left_len + node_len;
                curr = self.nodes[curr].right.unwrap();
            } else {
                // Node found!
                let offset = index - left_len;
                if offset == 0 {
                    // Non-split: Insert before this piece
                    self.insert_node_before(curr, new_node);
                } else {
                    // Split: Insert in the middle of this piece
                    self.split_and_insert(index, curr, new_node);
                }
                break;
            }
        }
    }
    fn insert_node_before(&mut self, target_node: usize, new_node: usize) {
        self.nodes[new_node].left = None;
        self.nodes[new_node].right = None;
        self.nodes[new_node].color = Color::Red;

        if self.nodes[target_node].left.is_none() {
            self.nodes[target_node].left = Some(new_node);
            self.nodes[new_node].parent = Some(target_node);
        } else {
            let mut curr = self.nodes[target_node].left.unwrap();
            while let Some(right) = self.nodes[curr].right {
                curr = right;
            }
            self.nodes[curr].right = Some(new_node);
            self.nodes[new_node].parent = Some(curr);
        }

        self.update_ancestors(self.nodes[new_node].parent);
        self.insert_fix(new_node);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BufferType {
    Original,
    Add,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Red,
    Black,
}


