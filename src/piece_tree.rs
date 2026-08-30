pub struct PieceTree {
    original: String,
    add: String,
    nodes: Vec<Node>,
    red_black_tree: RedBlackTree,
}

struct RedBlackTree {
    black_height: usize,
    root_node: Option<usize>,
}

impl RedBlackTree {
    fn new(black_height: usize, root_node: Option<usize>) -> Self {
        Self {
            black_height,
            root_node,
        }
    }

    fn update_subtree_length(nodes: &mut [Node], node_idx: usize) {
        let node = &nodes[node_idx];
        let left_len = node.left.map_or(0, |l| nodes[l].subtree_length);
        let right_len = node.right.map_or(0, |r| nodes[r].subtree_length);
        nodes[node_idx].subtree_length = left_len + right_len + nodes[node_idx].length;
        nodes[node_idx].left_subtree_length = left_len;
    }

    fn left_rotate(&mut self, nodes: &mut [Node], x: usize) {
        let y = match nodes[x].right {
            Some(index) => index,
            None => return,
        };
        let y_left = nodes[y].left;
        nodes[x].right = y_left;
        if let Some(y_l) = y_left {
            nodes[y_l].parent = Some(x);
        }
        let x_parent = nodes[x].parent;
        nodes[y].parent = x_parent;
        match x_parent {
            Some(parent) => {
                if nodes[parent].left == Some(x) {
                    nodes[parent].left = Some(y);
                } else if nodes[parent].right == Some(x) {
                    nodes[parent].right = Some(y);
                } else {
                    if nodes[parent].left.is_none() && nodes[parent].right.is_none() {
                        panic!("a parent cannot have empty children {parent}")
                    } else {
                        panic!(
                            "indices mismatch  parent {parent} left {:?} right {:?} x {x}",
                            nodes[parent].left, nodes[parent].right
                        )
                    }
                }
            }
            None => {
                self.root_node = Some(y);
            }
        }
        nodes[y].left = Some(x);
        nodes[x].parent = Some(y);
        Self::update_subtree_length(nodes, x);
        Self::update_subtree_length(nodes, y);
    }
    fn right_rotate(&mut self, nodes: &mut [Node], y: usize) {
        let x = match nodes[y].left {
            Some(index) => index,
            None => return,
        };
        let x_right = nodes[x].right;
        nodes[y].left = x_right;
        if let Some(x_r) = x_right {
            nodes[x_r].parent = Some(y);
        }
        let y_parent = nodes[y].parent;
        nodes[x].parent = y_parent;
        match y_parent {
            Some(parent) => {
                if nodes[parent].left == Some(y) {
                    nodes[parent].left = Some(x);
                } else if nodes[parent].right == Some(y) {
                    nodes[parent].right = Some(x);
                } else {
                    if nodes[parent].left.is_none() && nodes[parent].right.is_none() {
                        panic!("a parent cannot have empty children {parent}")
                    } else {
                        panic!(
                            "indices mismatch  parent {parent} left {:?} right {:?} y {y}",
                            nodes[parent].left, nodes[parent].right
                        )
                    }
                }
            }
            None => {
                self.root_node = Some(x);
            }
        }
        nodes[x].right = Some(y);
        nodes[y].parent = Some(x);
        Self::update_subtree_length(nodes, y);
        Self::update_subtree_length(nodes, x);
    }
    fn insert_node_after(&mut self, nodes: &mut [Node], target_node: usize, new_node: usize) {
        // Reset new_node connections
        nodes[new_node].left = None;
        nodes[new_node].right = None;
        nodes[new_node].color = Color::Red;

        if nodes[target_node].right.is_none() {
            // Case 1: Target has no right child, attach directly as right child
            nodes[target_node].right = Some(new_node);
            nodes[new_node].parent = Some(target_node);
        } else {
            // Case 2: Target has a right child, attach as leftmost child of the right subtree
            let mut curr = nodes[target_node].right.unwrap();
            while let Some(left) = nodes[curr].left {
                curr = left;
            }
            nodes[curr].left = Some(new_node);
            nodes[new_node].parent = Some(curr);
        }

        // 1. Update subtree lengths from new_node's parent up to root
        Self::update_ancestors(nodes, nodes[new_node].parent);

        // 2. Fix red-black tree invariants (rotations & recoloring)
        self.insert_fix(nodes, new_node);
    }

    /// Walk up the parent chain and update lengths
    fn update_ancestors(nodes: &mut [Node], mut curr: Option<usize>) {
        while let Some(node_idx) = curr {
            Self::update_subtree_length(nodes, node_idx);
            curr = nodes[node_idx].parent;
        }
    }

    fn insert_fix(&mut self, nodes: &mut [Node], current_node: usize) {
        let mut current_node = current_node;
        while Some(current_node) != self.root_node
            && nodes[nodes[current_node].parent.unwrap()].color == Color::Red
        {
            let mut parent = match nodes[current_node].parent {
                Some(p) => p,
                None => {
                    break;
                }
            };
            let mut grand_parent = match nodes[parent].parent {
                Some(p) => p,
                None => {
                    break;
                }
            };
            if nodes[grand_parent].left == Some(parent) {
                if let Some(right) = nodes[grand_parent].right
                    && nodes[right].color == Color::Red
                {
                    nodes[right].color = Color::Black;
                    if let Some(left) = nodes[grand_parent].left {
                        nodes[left].color = Color::Black;
                    };
                    nodes[grand_parent].color = Color::Red;
                    current_node = grand_parent;
                } else {
                    if nodes[parent].right == Some(current_node) {
                        current_node = parent;
                        self.left_rotate(nodes, current_node);
                        parent = match nodes[current_node].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                        grand_parent = match nodes[parent].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                    }
                    nodes[parent].color = Color::Black;

                    nodes[grand_parent].color = Color::Red;
                    self.right_rotate(nodes, grand_parent);
                }
            } else {
                if let Some(left) = nodes[grand_parent].left
                    && nodes[left].color == Color::Red
                {
                    nodes[left].color = Color::Black;
                    if let Some(right) = nodes[grand_parent].right {
                        nodes[right].color = Color::Black;
                    };
                    nodes[grand_parent].color = Color::Red;
                    current_node = grand_parent;
                } else {
                    if nodes[parent].left == Some(current_node) {
                        current_node = parent;
                        self.right_rotate(nodes, current_node);
                        parent = match nodes[current_node].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                        grand_parent = match nodes[parent].parent {
                            Some(p) => p,
                            None => {
                                break;
                            }
                        };
                    }

                    nodes[parent].color = Color::Black;

                    nodes[grand_parent].color = Color::Red;
                    self.left_rotate(nodes, grand_parent);
                }
            }
        }
        if let Some(root) = self.root_node
            && nodes[root].color == Color::Red
        {
            nodes[root].color = Color::Black;
            self.black_height += 1;
        }
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
        nodes[new_node].left = None;
        nodes[new_node].right = None;
        nodes[new_node].color = Color::Red;

        if nodes[target_node].left.is_none() {
            nodes[target_node].left = Some(new_node);
            nodes[new_node].parent = Some(target_node);
        } else {
            let mut curr = nodes[target_node].left.unwrap();
            while let Some(right) = nodes[curr].right {
                curr = right;
            }
            nodes[curr].right = Some(new_node);
            nodes[new_node].parent = Some(curr);
        }

        Self::update_ancestors(nodes, nodes[new_node].parent);
        self.insert_fix(nodes, new_node);
    }
    fn insert_into_tree(&mut self, nodes: &mut Vec<Node>, new_node: usize, mut index: usize) {
        let root = match self.root_node {
            Some(r) => r,
            None => {
                // Empty tree: new_node becomes the black root
                nodes[new_node].color = Color::Black;
                self.root_node = Some(new_node);
                self.black_height = 1;
                return;
            }
        };

        // Case 1: Inserting at or past the end of the entire document
        if index >= nodes[root].subtree_length {
            let mut rightmost = root;
            while let Some(r) = nodes[rightmost].right {
                rightmost = r;
            }
            self.insert_node_after(nodes, rightmost, new_node);
            return;
        }

        // Case 2: Traverse to find the exact piece containing `index`
        let mut curr = root;
        loop {
            let left_len = nodes[curr].left_subtree_length;
            let node_len = nodes[curr].length;

            if index < left_len {
                curr = nodes[curr].left.unwrap();
            } else if index >= left_len + node_len {
                index -= left_len + node_len;
                curr = nodes[curr].right.unwrap();
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
        let mut y_orig_color = nodes[z].color;
        let x: Option<usize>;
        let x_parent: Option<usize>;

        if nodes[z].left.is_none() {
            x = nodes[z].right;
            x_parent = nodes[z].parent;
            self.transplant(nodes, z, x);
            Self::update_ancestors(nodes, x_parent);
        } else if nodes[z].right.is_none() {
            x = nodes[z].left;
            x_parent = nodes[z].parent;
            self.transplant(nodes, z, x);
            Self::update_ancestors(nodes, x_parent);
        } else {
            let y_idx = Self::minimum(nodes, nodes[z].right).unwrap();
            y_orig_color = nodes[y_idx].color;
            x = nodes[y_idx].right;

            if nodes[y_idx].parent == Some(z) {
                x_parent = Some(y_idx);
                if let Some(x_idx) = x {
                    nodes[x_idx].parent = Some(y_idx);
                }
            } else {
                x_parent = nodes[y_idx].parent;
                self.transplant(nodes, y_idx, nodes[y_idx].right);
                nodes[y_idx].right = nodes[z].right;
                if let Some(r) = nodes[y_idx].right {
                    nodes[r].parent = Some(y_idx);
                }
                // Update subtree lengths starting from y's original parent
                Self::update_ancestors(nodes, x_parent);
            }

            self.transplant(nodes, z, Some(y_idx));
            nodes[y_idx].left = nodes[z].left;
            if let Some(l) = nodes[y_idx].left {
                nodes[l].parent = Some(y_idx);
            }
            nodes[y_idx].color = nodes[z].color;

            // Update y and its ancestors
            Self::update_subtree_length(nodes, y_idx);
            Self::update_ancestors(nodes, nodes[y_idx].parent);
        }

        if y_orig_color == Color::Black {
            self.delete_fix(nodes, x, x_parent);
        }
        if self.root_node.is_none() {
            self.black_height = 0;
        }
    }
    fn detach_leftmost_node(&mut self, nodes: &mut [Node]) -> Option<usize> {
        let mut z: Option<usize> = None;
        let mut current_node = self.root_node;
        while let Some(curr_node_idx) = current_node {
            z = Some(curr_node_idx);
            current_node = nodes[curr_node_idx].left;
        }
        let z = z?;
        self.delete_node(nodes, z);

        // Isolate detached node
        nodes[z].left = None;
        nodes[z].right = None;
        nodes[z].parent = None;
        nodes[z].color = Color::Red;

        Some(z)
    }
    fn delete_fix(
        &mut self,
        nodes: &mut [Node],
        mut x: Option<usize>,
        mut x_parent: Option<usize>,
    ) {
        while x != self.root_node && x.map_or(Color::Black, |n| nodes[n].color) == Color::Black {
            let parent = match x_parent {
                Some(p) => p,
                None => break,
            };

            if x == nodes[parent].left {
                let mut w = nodes[parent].right;

                // Case 1: Sibling w is Red
                if let Some(w_idx) = w
                    && nodes[w_idx].color == Color::Red
                {
                    nodes[w_idx].color = Color::Black;
                    nodes[parent].color = Color::Red;
                    self.left_rotate(nodes, parent);
                    w = nodes[parent].right;
                }

                let w_idx = w.unwrap();

                let w_left_black = nodes[w_idx]
                    .left
                    .is_none_or(|l| nodes[l].color == Color::Black);
                let w_right_black = nodes[w_idx]
                    .right
                    .is_none_or(|r| nodes[r].color == Color::Black);

                // Case 2: Sibling w is Black and both children are Black
                if w_left_black && w_right_black {
                    nodes[w_idx].color = Color::Red;
                    x = Some(parent);
                    x_parent = nodes[parent].parent;
                } else {
                    // Case 3: Sibling w is Black, left child is Red, right
                    if w_right_black {
                        if let Some(w_left) = nodes[w_idx].left {
                            nodes[w_left].color = Color::Black;
                        }
                        nodes[w_idx].color = Color::Red;
                        self.right_rotate(nodes, w_idx);
                        w = nodes[parent].right;
                    }

                    // Case 4: Sibling w is Black, right child is Red
                    let w_idx = w.unwrap();
                    nodes[w_idx].color = nodes[parent].color;
                    nodes[parent].color = Color::Black;
                    if let Some(w_right) = nodes[w_idx].right {
                        nodes[w_right].color = Color::Black;
                    }
                    self.left_rotate(nodes, parent);
                    if let Some(root) = self.root_node {
                        nodes[root].color = Color::Black;
                    }
                    return;
                }
            } else {
                // Symmetric cases when x is the right child
                let mut w = nodes[parent].left;

                if let Some(w_idx) = w
                    && nodes[w_idx].color == Color::Red
                {
                    nodes[w_idx].color = Color::Black;
                    nodes[parent].color = Color::Red;
                    self.right_rotate(nodes, parent);
                    w = nodes[parent].left;
                }

                let w_idx = w.unwrap();
                let w_left_black = nodes[w_idx]
                    .left
                    .is_none_or(|l| nodes[l].color == Color::Black);
                let w_right_black = nodes[w_idx]
                    .right
                    .is_none_or(|r| nodes[r].color == Color::Black);

                if w_left_black && w_right_black {
                    nodes[w_idx].color = Color::Red;
                    x = Some(parent);
                    x_parent = nodes[parent].parent;
                } else {
                    if w_left_black {
                        if let Some(w_right) = nodes[w_idx].right {
                            nodes[w_right].color = Color::Black;
                        }
                        nodes[w_idx].color = Color::Red;
                        self.left_rotate(nodes, w_idx);
                        w = nodes[parent].left;
                    }

                    let w_idx = w.unwrap();
                    nodes[w_idx].color = nodes[parent].color;
                    nodes[parent].color = Color::Black;
                    if let Some(w_left) = nodes[w_idx].left {
                        nodes[w_left].color = Color::Black;
                    }
                    self.right_rotate(nodes, parent);
                    if let Some(root) = self.root_node {
                        nodes[root].color = Color::Black;
                    }
                    return;
                }
            }
        }
        if x == self.root_node {
            self.black_height -= 1;
        }

        if let Some(x_idx) = x {
            nodes[x_idx].color = Color::Black;
        }
    }
    fn minimum(nodes: &[Node], node: Option<usize>) -> Option<usize> {
        let mut left_most = node;
        let mut current_node = node;
        while let Some(curr_node_idx) = current_node {
            left_most = Some(curr_node_idx);
            current_node = nodes[curr_node_idx].left;
        }
        left_most
    }
    fn transplant(&mut self, nodes: &mut [Node], u: usize, v: Option<usize>) {
        if nodes[u].parent.is_none() {
            self.root_node = v;
        } else if Some(u) == nodes[nodes[u].parent.unwrap()].left {
            nodes[nodes[u].parent.unwrap()].left = v;
        } else {
            nodes[nodes[u].parent.unwrap()].right = v;
        }
        if let Some(v_idx) = v {
            nodes[v_idx].parent = nodes[u].parent;
        }
    }

    pub fn delete(&mut self, nodes: &mut [Node], index: usize, length: usize) {
        #[allow(unused_variables, unused_mut)]
        let mut index = index;
        let mut parent_node: Option<usize> = None;
        let mut current_node = self.root_node;

        while let Some(curr) = current_node {
            let left_len = nodes[curr].left_subtree_length;
            let node_len = nodes[curr].length;

            if index < left_len {
                current_node = nodes[curr].left;
            } else if index >= left_len + node_len {
                index -= left_len + node_len;
                current_node = nodes[curr].right;
            } else {
                // Node found!
                let offset = index - left_len;
                if offset == 0 {
                } else {
                }
                break;
            }
        }
    }
    fn catenate(nodes: &mut [Node], t1: RedBlackTree, t2: RedBlackTree) -> RedBlackTree {
        if t1.root_node.is_none() {
            return t2;
        }
        if t2.root_node.is_none() {
            return t1;
        }
        let mut t1 = t1;
        let mut t2 = t2;
        let v = match t2.detach_leftmost_node(nodes) {
            Some(v_idx) => v_idx,
            None => {
                return RedBlackTree {
                    black_height: 0,
                    root_node: None,
                };
            }
        };
        if t2.root_node.is_none() {
            // Find rightmost node of T1
            let mut rightmost = t1.root_node.unwrap();
            while let Some(r) = nodes[rightmost].right {
                rightmost = r;
            }
            t1.insert_node_after(nodes, rightmost, v);
            return t1;
        }
        let mut curr_black_height = t2.black_height;
        let mut current_node: Option<usize> = t2.root_node;
        let mut rho: Option<usize> = t2.root_node;

        while let Some(curr_node_idx) = current_node {
            rho = Some(curr_node_idx);
            if nodes[curr_node_idx].color == Color::Black {
                if curr_black_height == t1.black_height {
                    break;
                }
                curr_black_height -= 1;
            }
            current_node = nodes[curr_node_idx].left;
        }
        let rho_parent = match rho {
            Some(rho_idx) => nodes[rho_idx].parent,
            None => {
                return RedBlackTree {
                    black_height: 0,
                    root_node: None,
                };
            }
        };

        nodes[v].left = t1.root_node;
        nodes[v].right = rho;
        if let Some(l) = nodes[v].left {
            nodes[l].parent = Some(v);
        }
        if let Some(r) = nodes[v].right {
            nodes[r].parent = Some(v);
        }
        match rho_parent {
            Some(rho_p_idx) => {
                nodes[v].parent = rho_parent;
                nodes[rho_p_idx].left = Some(v);
                t1.root_node = t2.root_node;
                t1.black_height = t2.black_height;
            }
            None => {
                t1.root_node = Some(v);
            }
        }
        Self::update_subtree_length(nodes, v);
        t1.insert_fix(nodes, v);
        t1
    }
    fn split(&mut self,)
}

struct Node {
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
    fn new(buffer_type: BufferType, start: usize, length: usize) -> Self {
        Node {
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
            let mut root = Node::new(BufferType::Original, 0, original.len());
            root.color = Color::Black;
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: vec![root],
                red_black_tree: RedBlackTree::new(1, Some(0)),
            }
        } else {
            Self {
                original: String::from(original),
                add: String::new(),
                nodes: Vec::new(),
                red_black_tree: RedBlackTree::new(0, None),
            }
        }
    }
    pub fn get_text(&self) -> String {
        let capacity = match self.red_black_tree.root_node {
            Some(root) => self.nodes[root].subtree_length,
            None => return String::new(),
        };
        let mut result = String::with_capacity(capacity);
        self.in_order(self.red_black_tree.root_node, &mut result);
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
