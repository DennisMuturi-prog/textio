pub struct PieceTree {
    original: String,
    add: String,
    nodes: Vec<Node>,
    red_black_tree: RedBlackTree,
}

struct RedBlackTree {
    black_height: usize,
    root_node: usize,
}

impl RedBlackTree {
    fn new(black_height: usize, root_node: usize) -> Self {
        Self {
            black_height,
            root_node,
        }
    }

    fn update_subtree_length(nodes: &mut [Node], node_idx: usize) {
        let node = &nodes[node_idx];
        let left_len = nodes[node.left].subtree_length;
        let right_len = nodes[node.right].subtree_length;
        nodes[node_idx].subtree_length = left_len + right_len + nodes[node_idx].length;
        nodes[node_idx].left_subtree_length = left_len;
    }

    fn left_rotate(&mut self, nodes: &mut [Node], x: usize) {
        let y = nodes[x].right;
        nodes[x].right = nodes[y].left;
        if nodes[y].left != 0 {
            nodes[nodes[y].left].parent = x;
        }
        nodes[y].parent = nodes[x].parent;
        if nodes[x].parent == 0 {
            self.root_node = y;
        } else if x == nodes[nodes[x].parent].left {
            nodes[nodes[x].parent].left = y;
        } else {
            nodes[nodes[x].parent].right = y;
        }

        nodes[y].left = x;
        nodes[x].parent = y;
        Self::update_subtree_length(nodes, x);
        Self::update_subtree_length(nodes, y);
    }
    fn right_rotate(&mut self, nodes: &mut [Node], x: usize) {
        let y = nodes[x].left;
        nodes[x].left = nodes[y].right;
        if nodes[y].right != 0 {
            nodes[nodes[y].right].parent = x;
        }
        nodes[y].parent = nodes[x].parent;
        if nodes[y].parent == 0 {
            self.root_node = y;
        } else if x == nodes[nodes[x].parent].right {
            nodes[nodes[x].parent].right = y;
        } else {
            nodes[nodes[x].parent].left = y;
        }
        nodes[y].right = x;
        nodes[x].parent = y;
        Self::update_subtree_length(nodes, x);
        Self::update_subtree_length(nodes, y);
    }
    fn insert_node_after(&mut self, nodes: &mut [Node], target_node: usize, new_node: usize) {
        // Reset new_node connections
        nodes[new_node].left = 0;
        nodes[new_node].right = 0;
        nodes[new_node].color = Color::Red;

        if nodes[target_node].right == 0 {
            // Case 1: Target has no right child, attach directly as right child
            nodes[target_node].right = new_node;
            nodes[new_node].parent = target_node;
        } else {
            // Case 2: Target has a right child, attach as leftmost child of the right subtree
            let mut curr = nodes[target_node].right;
            let mut dest = 0;
            while curr != 0 {
                dest = curr;
                curr = nodes[curr].left;
            }
            nodes[dest].left = new_node;
            nodes[new_node].parent = dest;
        }

        // 1. Update subtree lengths from new_node's parent up to root
        Self::update_ancestors(nodes, nodes[new_node].parent);

        // 2. Fix red-black tree invariants (rotations & recoloring)
        self.insert_fix(nodes, new_node);
    }

    /// Walk up the parent chain and update lengths
    fn update_ancestors(nodes: &mut [Node], mut curr: usize) {
        while curr != 0 {
            Self::update_subtree_length(nodes, curr);
            curr = nodes[curr].parent;
        }
    }

    fn insert_fix(&mut self, nodes: &mut [Node], z: usize) {
        let mut z = z;
        while nodes[z].parent != 0 && nodes[nodes[z].parent].color == Color::Red {
            if nodes[z].parent == nodes[nodes[nodes[z].parent].parent].left {
                let mut y = nodes[nodes[nodes[z].parent].parent].right;
                if nodes[y].color == Color::Red {
                    nodes[nodes[z].parent].color = Color::Black;
                    nodes[y].color = Color::Black;
                    nodes[nodes[nodes[z].parent].parent].color = Color::Red;
                    z = nodes[nodes[z].parent].parent;
                } else {
                    if z == nodes[nodes[z].parent].right {
                        z = nodes[z].parent;
                        self.left_rotate(nodes, z);
                    }
                    nodes[nodes[z].parent].color = Color::Black;
                    nodes[nodes[nodes[z].parent].parent].color = Color::Red;
                    self.right_rotate(nodes, nodes[nodes[z].parent].parent);
                }
            } else {
                let mut y = nodes[nodes[nodes[z].parent].parent].left;
                if nodes[y].color == Color::Red {
                    nodes[nodes[z].parent].color = Color::Black;
                    nodes[y].color = Color::Black;
                    nodes[nodes[nodes[z].parent].parent].color = Color::Red;
                    z = nodes[nodes[z].parent].parent;
                } else {
                    if z == nodes[nodes[z].parent].left {
                        z = nodes[z].parent;
                        self.right_rotate(nodes, z);
                    }
                    nodes[nodes[z].parent].color = Color::Black;
                    nodes[nodes[nodes[z].parent].parent].color = Color::Red;
                    self.left_rotate(nodes, nodes[nodes[z].parent].parent);
                }
            }
            if z == self.root_node {
                break;
            }
        }
        if nodes[self.root_node].color == Color::Red {
            self.black_height += 1;
        }
        nodes[self.root_node].color = Color::Black;
    }

    fn split_and_insert(
        &mut self,
        nodes: &mut Vec<Node>,
        index: usize,
        curr_node_idx: usize,
        new_node: usize,
    ) {
        let offset_in_node = index - nodes[curr_node_idx].left_subtree_length;

        if offset_in_node > 0 && offset_in_node < nodes[curr_node_idx].length {
            let right_start = nodes[curr_node_idx].start + offset_in_node;
            let right_len = nodes[curr_node_idx].length - offset_in_node;
            let buffer_type = nodes[curr_node_idx].buffer_type;

            // 1. Shorten left piece in-place
            nodes[curr_node_idx].length = offset_in_node;
            Self::update_subtree_length(nodes, curr_node_idx);

            // 2. Create right piece
            let right_node_idx = nodes.len();
            nodes.push(Node::new(buffer_type, right_start, right_len));

            // 3. Insert new_node and right_node as consecutive successors in the tree
            self.insert_node_after(nodes, curr_node_idx, new_node);
            self.insert_node_after(nodes, new_node, right_node_idx);
        }
    }

    fn insert_node_before(&mut self, nodes: &mut [Node], target_node: usize, new_node: usize) {
        nodes[new_node].left = 0;
        nodes[new_node].right = 0;
        nodes[new_node].color = Color::Red;

        if nodes[target_node].left == 0 {
            nodes[target_node].left = new_node;
            nodes[new_node].parent = target_node;
        } else {
            let mut curr = nodes[target_node].left;
            let mut dest = 0;
            while curr != 0 {
                dest = curr;
                curr = nodes[curr].right;
            }
            nodes[dest].right = new_node;
            nodes[new_node].parent = dest;
        }

        Self::update_ancestors(nodes, nodes[new_node].parent);
        self.insert_fix(nodes, new_node);
    }
    fn insert_into_tree(&mut self, nodes: &mut Vec<Node>, new_node: usize, mut index: usize) {
        if self.root_node == 0 {
            nodes[new_node].color = Color::Black;
            self.root_node = new_node;
            self.black_height = 1;
            return;
        }

        // Case 1: Inserting at or past the end of the entire document
        if index >= nodes[self.root_node].subtree_length {
            let mut rightmost = self.root_node;
            while nodes[rightmost].right != 0 {
                rightmost = nodes[rightmost].right;
            }
            self.insert_node_after(nodes, rightmost, new_node);
            return;
        }

        // Case 2: Traverse to find the exact piece containing `index`
        let mut curr = self.root_node;
        while curr != 0 {
            let left_len = nodes[curr].left_subtree_length;
            let node_len = nodes[curr].length;

            if index < left_len {
                curr = nodes[curr].left;
            } else if index >= left_len + node_len {
                index -= left_len + node_len;
                curr = nodes[curr].right;
            } else {
                // Node found!
                let offset = index - left_len;
                if offset == 0 {
                    // Non-split: Insert before this piece
                    self.insert_node_before(nodes, curr, new_node);
                } else {
                    // Split: Insert in the middle of this piece
                    self.split_and_insert(nodes, index, curr, new_node);
                }
                break;
            }
        }
    }
    fn delete_node(&mut self, nodes: &mut [Node], z: usize) {
        let mut y = z;
        let x: usize;
        let mut y_orig_color = nodes[y].color;
        if nodes[z].left == 0 {
            x = nodes[z].right;
            self.transplant(nodes, z, nodes[z].right);
        } else if nodes[z].right == 0 {
            x = nodes[z].left;
            self.transplant(nodes, z, nodes[z].left);
        } else {
            y = Self::minimum(nodes, nodes[z].right);
            y_orig_color = nodes[y].color;
            x = nodes[y].right;
            if nodes[y].parent == z {
                nodes[x].parent = y;
            } else {
                self.transplant(nodes, y, nodes[y].right);
                nodes[y].right = nodes[z].right;
                nodes[nodes[y].right].parent = y;
            }
            self.transplant(nodes, z, y);
            nodes[y].left = nodes[z].left;
            nodes[nodes[y].left].parent = y;
            nodes[y].color = nodes[z].color;
        }
        if y_orig_color == Color::Black {
            self.delete_fix(nodes, x);
        }
        if self.root_node == 0 {
            self.black_height = 0;
        }
    }
    fn detach_leftmost_node(&mut self, nodes: &mut [Node]) -> usize {
        let mut z = 0;
        let mut current_node = self.root_node;
        while current_node != 0 {
            z = current_node;
            current_node = nodes[current_node].left;
        }

        self.delete_node(nodes, z);

        // Isolate detached node
        nodes[z].left = 0;
        nodes[z].right = 0;
        nodes[z].parent = 0;
        nodes[z].color = Color::Red;
        z
    }
    fn delete_fix(&mut self, nodes: &mut [Node], mut x: usize) {
        while x != self.root_node && nodes[x].color == Color::Black {
            if x == nodes[nodes[x].parent].left {
                let mut w = nodes[nodes[x].parent].right;
                if nodes[w].color == Color::Red {
                    nodes[w].color = Color::Black;
                    nodes[nodes[x].parent].color = Color::Red;
                    self.left_rotate(nodes, nodes[x].parent);
                    w = nodes[nodes[x].parent].right;
                }
                if nodes[nodes[w].left].color == Color::Black
                    && nodes[nodes[w].right].color == Color::Black
                {
                    nodes[w].color = Color::Red;
                    x = nodes[x].parent;
                } else {
                    if nodes[nodes[w].right].color == Color::Black {
                        nodes[nodes[w].left].color = Color::Black;
                        nodes[w].color = Color::Red;
                        self.right_rotate(nodes, w);
                        w = nodes[nodes[x].parent].right;
                    }
                    nodes[w].color = nodes[nodes[x].parent].color;
                    nodes[nodes[x].parent].color = Color::Black;
                    nodes[nodes[w].right].color = Color::Black;
                    self.left_rotate(nodes, nodes[x].parent);
                    x = self.root_node;
                }
            } else {
                let mut w = nodes[nodes[x].parent].left;
                if nodes[w].color == Color::Red {
                    nodes[w].color = Color::Black;
                    nodes[nodes[x].parent].color = Color::Red;
                    self.left_rotate(nodes, nodes[x].parent);
                    w = nodes[nodes[x].parent].left;
                }
                if nodes[nodes[w].right].color == Color::Black
                    && nodes[nodes[w].left].color == Color::Black
                {
                    nodes[w].color = Color::Red;
                    x = nodes[x].parent;
                } else {
                    if nodes[nodes[w].left].color == Color::Black {
                        nodes[nodes[w].right].color = Color::Black;
                        nodes[w].color = Color::Red;
                        self.left_rotate(nodes, w);
                        w = nodes[nodes[x].parent].left;
                    }
                    nodes[w].color = nodes[nodes[x].parent].color;
                    nodes[nodes[x].parent].color = Color::Black;
                    nodes[nodes[w].left].color = Color::Black;
                    self.right_rotate(nodes, nodes[x].parent);
                    x = self.root_node;
                }
            }
        }
        if x == self.root_node {
            self.black_height -= 1;
        }
        nodes[x].color = Color::Black;
    }
    fn minimum(nodes: &[Node], x: usize) -> usize {
        let mut left_most = x;
        while nodes[left_most].left != 0 {
            left_most = nodes[left_most].left;
        }
        left_most
    }
    fn transplant(&mut self, nodes: &mut [Node], u: usize, v: usize) {
        if nodes[u].parent == 0 {
            self.root_node = v;
        } else if u == nodes[nodes[u].parent].left {
            nodes[nodes[u].parent].left = v;
        } else {
            nodes[nodes[u].parent].right = v;
        }
        nodes[v].parent = nodes[u].parent;
        Self::update_subtree_length(nodes, v);
        Self::update_ancestors(nodes, v);
    }

    fn catenate(nodes: &mut [Node], t1: RedBlackTree, t2: RedBlackTree) -> RedBlackTree {
        if t1.root_node == 0 {
            return t2;
        }
        if t2.root_node == 0 {
            return t1;
        }
        let mut t1 = t1;
        let mut t2 = t2;
        let v = t2.detach_leftmost_node(nodes);
        if v == 0 {
            return RedBlackTree {
                black_height: 0,
                root_node: 0,
            };
        }
        if t2.root_node == 0 {
            // Find rightmost node of T1
            let mut rightmost = t1.root_node;
            while nodes[rightmost].right != 0 {
                rightmost = nodes[rightmost].right;
            }
            t1.insert_node_after(nodes, rightmost, v);
            return t1;
        }
        let mut curr_black_height = t2.black_height;
        let mut current_node = t2.root_node;
        let mut rho = t2.root_node;

        while current_node != 0 {
            rho = current_node;
            if nodes[current_node].color == Color::Black {
                if curr_black_height == t1.black_height {
                    break;
                }
                curr_black_height -= 1;
            }
            current_node = nodes[current_node].left;
        }
        let rho_parent = nodes[rho].parent;

        nodes[v].left = t1.root_node;
        nodes[v].right = rho;
        nodes[nodes[v].left].parent = v;
        nodes[nodes[v].right].parent = v;
        if rho_parent == 0 {
            t1.root_node = v;
        } else {
            nodes[v].parent = rho_parent;
            nodes[rho_parent].left = v;
            t1.root_node = t2.root_node;
            t1.black_height = t2.black_height;
        }
        Self::update_subtree_length(nodes, v);
        t1.insert_fix(nodes, v);
        t1
    }
}

struct Node {
    buffer_type: BufferType,
    color: Color,
    start: usize,
    length: usize,
    subtree_length: usize,
    left_subtree_length: usize,
    left: usize,
    right: usize,
    parent: usize,
}
impl Node {
    fn new(buffer_type: BufferType, start: usize, length: usize) -> Self {
        Node {
            buffer_type,
            color: Color::Red,
            start,
            length,
            subtree_length: length,
            left_subtree_length: 0,
            left: 0,
            right: 0,
            parent: 0,
        }
    }
}

impl PieceTree {
    pub fn new(original: &str) -> Self {
        if !original.is_empty() {
            let mut nil_node = Node::new(BufferType::Original, 0, 0);
            nil_node.color = Color::Black;
            let mut root = Node::new(BufferType::Original, 0, original.len());
            root.color = Color::Black;
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: vec![nil_node, root],
                red_black_tree: RedBlackTree::new(1, 1),
            }
        } else {
            let mut nil_node = Node::new(BufferType::Original, 0, 0);
            nil_node.color = Color::Black;
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: vec![nil_node],
                red_black_tree: RedBlackTree::new(0, 0),
            }
        }
    }
    pub fn get_text(&self) -> String {
        if self.red_black_tree.root_node == 0 {
            return String::new();
        }
        let capacity = self.nodes[self.red_black_tree.root_node].subtree_length;
        let mut result = String::with_capacity(capacity);
        self.in_order(self.red_black_tree.root_node, &mut result);
        result
    }
    fn in_order(&self, node_idx: usize, out: &mut String) {
        if node_idx != 0 {
            let node = &self.nodes[node_idx];
            self.in_order(node.left, out);

            let buffer = match node.buffer_type {
                BufferType::Original => &self.original,
                BufferType::Add => &self.add,
            };
            out.push_str(&buffer[node.start..node.start + node.length]);

            self.in_order(node.right, out);
        }
    }
    pub fn insert(&mut self, index: usize, content: &str) {
        if content.is_empty() {
            return;
        }
        let new_node = self.pre_insert(content);
        self.red_black_tree
            .insert_into_tree(&mut self.nodes, new_node, index);
    }

    fn pre_insert(&mut self, content: &str) -> usize {
        let previous_buffer_len = self.add.len();
        let previous_nodes_len = self.nodes.len();
        self.add.push_str(content);
        self.nodes.push(Node::new(
            BufferType::Add,
            previous_buffer_len,
            content.len(),
        ));
        previous_nodes_len
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
