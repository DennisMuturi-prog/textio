struct PieceTree {
    original: String,
    add: String,
    nodes: Vec<Node>,
    root_node: Option<usize>,
}

struct Node {
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
        self.insert_fixup(new_node);
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
        let mut parent = match self.nodes[current_node].parent {
            Some(p) => p,
            None => {
                if let Some(root) = self.root_node {
                    self.nodes[root].color = Color::Black;
                }
                return;
            }
        };
        while self.nodes[parent].color == Color::Red {
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
                    parent = match self.nodes[current_node].parent {
                        Some(p) => p,
                        None => {
                            break;
                        }
                    };
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
                    parent = match self.nodes[current_node].parent {
                        Some(p) => p,
                        None => {
                            break;
                        }
                    };
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
                    parent = match self.nodes[current_node].parent {
                        Some(p) => p,
                        None => {
                            break;
                        }
                    };
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
                    parent = match self.nodes[current_node].parent {
                        Some(p) => p,
                        None => {
                            break;
                        }
                    };
                }
            }
        }
        if let Some(root) = self.root_node {
            self.nodes[root].color = Color::Black;
        }
    }
    fn insert_fixup(&mut self, mut z: usize) {
        while let Some(parent) = self.nodes[z].parent {
            if matches!(self.nodes[parent].color, Color::Black) {
                break;
            }

            // Parent is Red -> Grandparent must exist
            let grandparent = match self.nodes[parent].parent {
                Some(gp) => gp,
                None => break,
            };

            if Some(parent) == self.nodes[grandparent].left {
                let uncle = self.nodes[grandparent].right;
                if uncle.map_or(false, |u| matches!(self.nodes[u].color, Color::Red)) {
                    // Case 1: Uncle is Red -> Recolor
                    let u = uncle.unwrap();
                    self.nodes[parent].color = Color::Black;
                    self.nodes[u].color = Color::Black;
                    self.nodes[grandparent].color = Color::Red;
                    z = grandparent;
                } else {
                    // Case 2: Uncle is Black and z is right child -> Left rotate
                    if Some(z) == self.nodes[parent].right {
                        z = parent;
                        self.left_rotate(z);
                    }
                    // Case 3: Uncle is Black and z is left child -> Right rotate
                    let parent = self.nodes[z].parent.unwrap();
                    let grandparent = self.nodes[parent].parent.unwrap();
                    self.nodes[parent].color = Color::Black;
                    self.nodes[grandparent].color = Color::Red;
                    self.right_rotate(grandparent);
                }
            } else {
                // Symmetric case: parent is right child of grandparent
                let uncle = self.nodes[grandparent].left;
                if uncle.map_or(false, |u| matches!(self.nodes[u].color, Color::Red)) {
                    let u = uncle.unwrap();
                    self.nodes[parent].color = Color::Black;
                    self.nodes[u].color = Color::Black;
                    self.nodes[grandparent].color = Color::Red;
                    z = grandparent;
                } else {
                    if Some(z) == self.nodes[parent].left {
                        z = parent;
                        self.right_rotate(z);
                    }
                    let parent = self.nodes[z].parent.unwrap();
                    let grandparent = self.nodes[parent].parent.unwrap();
                    self.nodes[parent].color = Color::Black;
                    self.nodes[grandparent].color = Color::Red;
                    self.left_rotate(grandparent);
                }
            }
        }

        // Root must always be Black
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
    fn insert_into_tree(&mut self, new_node: usize, index: usize) {
        let mut index = index;
        if self.root_node.is_none() {
            self.nodes[new_node].color = Color::Black;
            self.root_node = Some(new_node);
            return;
        }
        let mut current_node_index = self.root_node;

        while let Some(curr_node_idx) = current_node_index {
            if self.nodes[curr_node_idx].left.is_none() && self.nodes[curr_node_idx].right.is_none()
            {
                if index < self.nodes[curr_node_idx].left_subtree_length {
                    self.nodes[curr_node_idx].left = Some(new_node);
                    self.nodes[new_node].parent = Some(curr_node_idx);
                } else if index >= self.nodes[curr_node_idx].left_subtree_length
                    && index
                        < self.nodes[curr_node_idx].left_subtree_length
                            + self.nodes[curr_node_idx].length
                {
                    let offset_in_node = index - self.nodes[curr_node_idx].left_subtree_length;
                    if offset_in_node == 0 {
                        self.nodes[curr_node_idx].left = Some(new_node);
                        self.nodes[new_node].parent = Some(curr_node_idx);
                    } else {
                        self.split_and_insert(index, curr_node_idx, new_node);
                    }
                } else {
                    self.nodes[curr_node_idx].right = Some(new_node);
                    self.nodes[new_node].parent = Some(curr_node_idx);
                }
                break;
            }
            if index < self.nodes[curr_node_idx].left_subtree_length {
                current_node_index = self.nodes[curr_node_idx].left;
            } else if index >= self.nodes[curr_node_idx].left_subtree_length
                && index
                    < self.nodes[curr_node_idx].left_subtree_length
                        + self.nodes[curr_node_idx].length
            {
                let offset_in_node = index - self.nodes[curr_node_idx].left_subtree_length;
                if offset_in_node == 0 {
                    current_node_index = self.nodes[curr_node_idx].left;
                } else {
                    self.split_and_insert(index, curr_node_idx, new_node);
                    break;
                }
            } else {
                current_node_index = self.nodes[curr_node_idx].right;
                index -= self.nodes[curr_node_idx].left_subtree_length
                    + self.nodes[curr_node_idx].length;
            }
        }
    }
}

#[derive(Clone, Copy)]
enum BufferType {
    Original,
    Add,
}

#[derive(PartialEq)]
enum Color {
    Red,
    Black,
}
