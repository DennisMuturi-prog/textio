use std::usize;

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
                let y = nodes[nodes[nodes[z].parent].parent].right;
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
                let y = nodes[nodes[nodes[z].parent].parent].left;
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
    fn split_and_insert_for_delete(
        &mut self,
        nodes: &mut Vec<Node>,
        index: usize,
        curr_node_idx: usize,
        length: usize,
    ) {
        let offset_in_node = index - nodes[curr_node_idx].left_subtree_length;

        if offset_in_node > 0 && offset_in_node < nodes[curr_node_idx].length {
            let right_start = nodes[curr_node_idx].start + offset_in_node + length;
            let right_len = nodes[curr_node_idx].length - offset_in_node - length;
            let buffer_type = nodes[curr_node_idx].buffer_type;

            // 1. Shorten left piece in-place
            nodes[curr_node_idx].length = offset_in_node;
            Self::update_subtree_length(nodes, curr_node_idx);

            // 2. Create right piece
            let right_node_idx = nodes.len();
            nodes.push(Node::new(buffer_type, right_start, right_len));

            // 3. Insert new_node and right_node as consecutive successors in the tree
            self.insert_node_after(nodes, curr_node_idx, right_node_idx);
        }
    }
    fn split_and_insert_for_bigger_delete(
        &mut self,
        nodes: &mut Vec<Node>,
        index: usize,
        curr_node_idx: usize,
    ) -> usize {
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

            self.insert_node_after(nodes, curr_node_idx, right_node_idx);
            return right_node_idx;
        }
        0
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
            let parent = nodes[z].parent;
            self.transplant(nodes, z, nodes[z].right);
            Self::update_ancestors(nodes, parent); // Fix 1: Update ancestors when left is 0
        } else if nodes[z].right == 0 {
            x = nodes[z].left;
            let parent = nodes[z].parent;
            self.transplant(nodes, z, nodes[z].left);
            Self::update_ancestors(nodes, parent); // Fix 1: Update ancestors when right is 0
        } else {
            y = Self::minimum(nodes, nodes[z].right);
            y_orig_color = nodes[y].color;
            x = nodes[y].right;
            let y_parent = nodes[y].parent;

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

            // Fix 2: If y was deeper in the right subtree, update from y's former parent
            if y_parent != z {
                Self::update_ancestors(nodes, y_parent);
            }

            Self::update_subtree_length(nodes, y);
            Self::update_ancestors(nodes, nodes[y].parent);
        }

        if y_orig_color == Color::Black {
            self.delete_fix(nodes, x);
        }

        // Fix 3: Clean up sentinel NIL node
        nodes[0].parent = 0;
        nodes[0].left = 0;
        nodes[0].right = 0;
        nodes[0].subtree_length = 0;
        nodes[0].left_subtree_length = 0;

        if self.root_node == 0 {
            self.black_height = 0;
        }
    }
    fn delete_node_legacy(&mut self, nodes: &mut [Node], z: usize) {
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
            Self::update_subtree_length(nodes, y);
            Self::update_ancestors(nodes, nodes[y].parent);
        }
        if y_orig_color == Color::Black {
            self.delete_fix(nodes, x);
        }
        nodes[0].parent = 0;
        nodes[0].left = 0;
        nodes[0].right = 0;
        nodes[0].subtree_length = 0;
        nodes[0].left_subtree_length = 0;
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
                    if x == self.root_node {
                        self.black_height -= 1;
                    }
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
                    self.right_rotate(nodes, nodes[x].parent);
                    w = nodes[nodes[x].parent].left;
                }
                if nodes[nodes[w].right].color == Color::Black
                    && nodes[nodes[w].left].color == Color::Black
                {
                    nodes[w].color = Color::Red;
                    x = nodes[x].parent;
                    if x == self.root_node {
                        self.black_height -= 1;
                    }
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
        if nodes[x].color == Color::Red {
            nodes[x].color = Color::Black;
        }
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
    }

    fn catenate(&mut self, nodes: &mut [Node], t2: RedBlackTree) {
        if self.root_node == 0 {
            self.root_node = t2.root_node;
            self.black_height = t2.black_height;
            return;
        }
        if t2.root_node == 0 {
            return;
        }
        let mut t2 = t2;
        let v = t2.detach_leftmost_node(nodes);
        if v == 0 {
            return;
        }
        if t2.root_node == 0 {
            // Find rightmost node of T1
            let mut rightmost = self.root_node;
            while nodes[rightmost].right != 0 {
                rightmost = nodes[rightmost].right;
            }
            self.insert_node_after(nodes, rightmost, v);
            return;
        }
        if self.black_height <= t2.black_height {
            // T2 is taller: descend T2's left spine to find rho where BH == self.black_height
            let mut curr_bh = t2.black_height;
            let mut curr = t2.root_node;
            let mut rho = t2.root_node;

            while curr != 0 {
                rho = curr;
                if nodes[curr].color == Color::Black {
                    if curr_bh == self.black_height {
                        break;
                    }
                    curr_bh -= 1;
                }
                curr = nodes[curr].left;
            }
            let rho_parent = nodes[rho].parent;

            nodes[v].left = self.root_node;
            nodes[v].right = rho;
            nodes[nodes[v].left].parent = v;
            nodes[nodes[v].right].parent = v;

            if rho_parent == 0 {
                self.root_node = v;
            } else {
                nodes[v].parent = rho_parent;
                nodes[rho_parent].left = v;
                self.root_node = t2.root_node;
                self.black_height = t2.black_height;
            }
            Self::update_subtree_length(nodes, v);
            Self::update_ancestors(nodes, rho_parent);
            self.insert_fix(nodes, v);
        } else {
            // T1 (self) is taller: descend T1's right spine to find sigma where BH == t2.black_height
            let mut curr_bh = self.black_height;
            let mut curr = self.root_node;
            let mut sigma = self.root_node;

            while curr != 0 {
                sigma = curr;
                if nodes[curr].color == Color::Black {
                    if curr_bh == t2.black_height {
                        break;
                    }
                    curr_bh -= 1;
                }
                curr = nodes[curr].right;
            }
            let sigma_parent = nodes[sigma].parent;

            nodes[v].left = sigma;
            nodes[v].right = t2.root_node;
            nodes[nodes[v].left].parent = v;
            nodes[nodes[v].right].parent = v;

            if sigma_parent == 0 {
                self.root_node = v;
            } else {
                nodes[v].parent = sigma_parent;
                nodes[sigma_parent].right = v;
            }
            Self::update_subtree_length(nodes, v);
            Self::update_ancestors(nodes, sigma_parent);
            self.insert_fix(nodes, v);
        }
    }
    /// Finds the node containing `index` and returns (node_idx, offset_within_node).
    fn find_node(&self, nodes: &[Node], mut index: usize) -> (usize, usize) {
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
                return (curr, index - left_len);
            }
        }
        (0, 0)
    }

    /// In-order successor of a node in the tree (same as Multiset.h successor)
    fn successor(nodes: &[Node], node: usize) -> usize {
        if nodes[node].right != 0 {
            let mut curr = nodes[node].right;
            while nodes[curr].left != 0 {
                curr = nodes[curr].left;
            }
            curr
        } else {
            let mut curr = node;
            let mut parent = nodes[curr].parent;
            while parent != 0 && curr == nodes[parent].right {
                curr = parent;
                parent = nodes[parent].parent;
            }
            parent
        }
    }
    fn delete(&mut self, nodes: &mut Vec<Node>, start: usize, length: usize) {
        if length == 0 || self.root_node == 0 {
            return;
        }

        let total_len = nodes[self.root_node].subtree_length;
        if start >= total_len {
            return;
        }
        let length = length.min(total_len - start);

        let (start_node, start_offset) = self.find_node(nodes, start);
        let end = start + length;
        let (end_node, end_offset) = if end >= total_len {
            (0, 0) // past the end
        } else {
            self.find_node(nodes, end)
        };

        // Case 1: Deletion is entirely within a single piece
        if start_node == end_node && start_node != 0 {
            let node_len = nodes[start_node].length;
            if start_offset == 0 && length == node_len {
                // Whole node deleted
                self.delete_node(nodes, start_node);
            } else if start_offset == 0 {
                // Trim prefix
                nodes[start_node].start += length;
                nodes[start_node].length -= length;
                Self::update_ancestors(nodes, start_node);
            } else if start_offset + length == node_len {
                // Trim suffix
                nodes[start_node].length -= length;
                Self::update_ancestors(nodes, start_node);
            } else {
                // Middle of node: split into left piece & new right piece
                let right_start = nodes[start_node].start + start_offset + length;
                let right_len = node_len - start_offset - length;
                let buffer_type = nodes[start_node].buffer_type;

                // Shorten left piece
                nodes[start_node].length = start_offset;
                Self::update_subtree_length(nodes, start_node);
                let parent = nodes[start_node].parent;
                Self::update_ancestors(nodes, start_node);

                // Create and insert right piece
                let right_node_idx = nodes.len();
                nodes.push(Node::new(buffer_type, right_start, right_len));
                self.insert_node_after(nodes, start_node, right_node_idx);
            }
            return;
        }

        // Case 2: Deletion spans multiple pieces
        let mut nodes_to_delete = Vec::new();

        // 1. Handle start_node
        if start_offset == 0 {
            nodes_to_delete.push(start_node);
        } else {
            nodes[start_node].length = start_offset;
            Self::update_ancestors(nodes, start_node);
        }

        // 2. Collect all whole nodes between start_node and end_node
        let mut curr = Self::successor(nodes, start_node);
        while curr != 0 && curr != end_node {
            nodes_to_delete.push(curr);
            curr = Self::successor(nodes, curr);
        }

        // 3. Handle end_node
        if end_node != 0 {
            if end_offset == nodes[end_node].length {
                nodes_to_delete.push(end_node);
            } else if end_offset > 0 {
                nodes[end_node].start += end_offset;
                nodes[end_node].length -= end_offset;
                Self::update_ancestors(nodes, end_node);
            }
        }

        // 4. Delete collected whole nodes
        for node in nodes_to_delete {
            self.delete_node(nodes, node);
        }
    }
    fn delete_legacy(&mut self, nodes: &mut Vec<Node>, index: usize, length: usize) {
        let mut index = index;
        if self.root_node == 0 {
            return;
        }

        // Case 1: Inserting at or past the end of the entire document
        if index >= nodes[self.root_node].subtree_length {
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
                if offset == 0 && length == nodes[curr].length {
                    self.delete_node(nodes, curr);
                } else if offset == 0 && length < nodes[curr].length {
                    nodes[curr].length -= length;
                    nodes[curr].start += length;
                    Self::update_ancestors(nodes, curr);
                } else if offset == 0 && length > nodes[curr].length {
                    let mut t2 = self.split(nodes, curr);
                    t2.delete(nodes, 0, length);
                    self.catenate(nodes, t2);
                } else {
                    // Split: Insert in the middle of this piece
                    if offset + length == nodes[curr].length {
                        nodes[curr].length -= length;
                        Self::update_ancestors(nodes, curr);
                    } else if offset + length < nodes[curr].length {
                        self.split_and_insert_for_delete(nodes, index, curr, length);
                    } else {
                        let position = self.split_and_insert_for_bigger_delete(nodes, index, curr);
                        let mut t2 = self.split(nodes, position);
                        t2.delete(nodes, 0, length);
                        self.catenate(nodes, t2);
                    }
                }
                break;
            }
        }
    }
    fn split(&mut self, nodes: &mut [Node], node: usize) -> RedBlackTree {
        let mut depth: i32 = 0;
        let mut current = node;
        let mut path = vec![PathSpecifier::Equal; self.black_height * 2];
        path[depth as usize] = PathSpecifier::Equal;
        while nodes[current].parent != 0 {
            depth += 1;
            if current == nodes[nodes[current].parent].left {
                path[depth as usize] = PathSpecifier::Smaller;
            } else {
                path[depth as usize] = PathSpecifier::Larger;
            }
            current = nodes[current].parent;
        }
        let mut current_b_height = self.black_height;
        let mut left_tree = RedBlackTree::new(0, 0);
        let mut left_b_height = 0;
        let mut spine_left = 0;
        let mut aux_left = 0;
        let mut right_tree = RedBlackTree::new(0, 0);
        let mut right_b_height = 0;
        let mut spine_right = 0;
        let mut aux_right = 0;
        let mut child;
        let mut next = 0;

        while depth >= 0 {
            if nodes[current].color == Color::Black {
                current_b_height -= 1;
            }
            if path[depth as usize] != PathSpecifier::Larger {
                child = nodes[current].right;
                next = nodes[current].left;
                if child != 0 && right_tree.root_node == 0 {
                    right_tree.root_node = child;
                    right_tree.black_height = current_b_height;
                    nodes[right_tree.root_node].parent = 0;
                    if nodes[right_tree.root_node].color == Color::Red {
                        nodes[right_tree.root_node].color = Color::Black;
                        right_tree.black_height += 1;
                    }
                    right_b_height = right_tree.black_height;
                    spine_right = right_tree.root_node;
                } else if child != 0 {
                    let mut current_right_black_height = current_b_height;
                    if nodes[child].color == Color::Red {
                        nodes[child].color = Color::Black;
                        current_right_black_height += 1;
                    }
                    while right_b_height > current_right_black_height {
                        if nodes[spine_right].color == Color::Black {
                            right_b_height -= 1;
                        }
                        spine_right = nodes[spine_right].left;
                    }
                    if nodes[spine_right].color == Color::Red {
                        spine_right = nodes[spine_right].left;
                    }
                    nodes[aux_right].parent = nodes[spine_right].parent;
                    nodes[aux_right].color = Color::Red;
                    nodes[aux_right].left = child;
                    nodes[aux_right].right = spine_right;
                    if nodes[aux_right].parent != 0 {
                        nodes[nodes[aux_right].parent].left = aux_right;
                    } else {
                        right_tree.root_node = aux_right;
                    }
                    nodes[child].parent = aux_right;
                    nodes[spine_right].parent = aux_right;
                    right_tree.insert_fix(nodes, aux_right);
                    aux_right = 0;
                    right_b_height = current_right_black_height;
                    spine_right = child;
                }
                if aux_right != 0 {
                    if right_tree.root_node != 0 {
                        while nodes[spine_right].left != 0 {
                            spine_right = nodes[spine_right].left;
                        }
                        nodes[aux_right].parent = spine_right;
                        nodes[aux_right].color = Color::Red;
                        nodes[aux_right].right = 0;
                        nodes[aux_right].left = 0;
                        nodes[spine_right].left = aux_right;
                        right_tree.insert_fix(nodes, aux_right);
                    } else {
                        right_tree.root_node = aux_right;
                        right_tree.black_height = 1;
                        nodes[aux_right].parent = 0;
                        nodes[aux_right].color = Color::Black;
                        nodes[aux_right].right = 0;
                        nodes[aux_right].left = 0;
                    }
                    spine_right = aux_right;
                    right_b_height = if nodes[spine_right].color == Color::Black {
                        1
                    } else {
                        0
                    };
                    aux_right = 0;
                }
                aux_right = current;
            }
            if path[depth as usize] != PathSpecifier::Smaller {
                child = nodes[current].left;
                next = nodes[current].right;
                if child != 0 && left_tree.root_node == 0 {
                    left_tree.root_node = child;
                    left_tree.black_height = current_b_height;
                    nodes[left_tree.root_node].parent = 0;
                    if nodes[left_tree.root_node].color == Color::Red {
                        nodes[left_tree.root_node].color = Color::Black;
                        left_tree.black_height += 1;
                    }
                    left_b_height = left_tree.black_height;
                    spine_left = left_tree.root_node;
                } else if child != 0 {
                    let mut current_left_black_height = current_b_height;
                    if nodes[child].color == Color::Red {
                        nodes[child].color = Color::Black;
                        current_left_black_height += 1;
                    }
                    while left_b_height > current_left_black_height {
                        if nodes[spine_left].color == Color::Black {
                            left_b_height -= 1;
                        }
                        spine_left = nodes[spine_left].right;
                    }
                    if nodes[spine_left].color == Color::Red {
                        spine_left = nodes[spine_left].right;
                    }
                    nodes[aux_left].parent = nodes[spine_left].parent;
                    nodes[aux_left].color = Color::Red;
                    nodes[aux_left].left = spine_left;
                    nodes[aux_left].right = child;
                    if nodes[aux_left].parent != 0 {
                        nodes[nodes[aux_left].parent].right = aux_left;
                    } else {
                        left_tree.root_node = aux_left;
                    }
                    nodes[child].parent = aux_left;
                    nodes[spine_left].parent = aux_left;
                    left_tree.insert_fix(nodes, aux_left);
                    aux_left = 0;
                    left_b_height = current_left_black_height;
                    spine_left = child;
                }
                if aux_left != 0 {
                    if left_tree.root_node != 0 {
                        while nodes[spine_left].right != 0 {
                            spine_left = nodes[spine_left].right;
                        }
                        nodes[aux_left].parent = spine_left;
                        nodes[aux_left].color = Color::Red;
                        nodes[aux_left].right = 0;
                        nodes[aux_left].left = 0;
                        nodes[spine_left].right = aux_left;
                        left_tree.insert_fix(nodes, aux_left);
                    } else {
                        left_tree.root_node = aux_left;
                        left_tree.black_height = 1;
                        nodes[aux_left].parent = 0;
                        nodes[aux_left].color = Color::Black;
                        nodes[aux_left].right = 0;
                        nodes[aux_left].left = 0;
                    }
                    spine_left = aux_left;
                    left_b_height = if nodes[spine_left].color == Color::Black {
                        1
                    } else {
                        0
                    };
                    aux_left = 0;
                }
                if depth > 0 {
                    aux_left = current;
                }
            }
            current = next;
            depth -= 1;
        }
        drop(path);
        if right_tree.root_node != 0 {
            while nodes[spine_right].left != 0 {
                spine_right = nodes[spine_right].left;
            }
            nodes[node].parent = spine_right;
            nodes[node].color = Color::Red;
            nodes[node].right = 0;
            nodes[node].left = 0;
            nodes[spine_right].left = node;
            right_tree.insert_fix(nodes, node);
        } else {
            right_tree.root_node = node;
            right_tree.black_height = 1;
            nodes[node].parent = 0;
            nodes[node].color = Color::Black;

            nodes[node].right = 0;
            nodes[node].left = 0;
        }
        self.root_node = left_tree.root_node;
        self.black_height = left_tree.black_height;
        right_tree
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
    pub fn delete(&mut self, index: usize, length: usize) {
        self.red_black_tree.delete(&mut self.nodes, index, length);
    }

    /// Validates all Red-Black Tree invariants and PieceTree metadata invariants.
    /// Returns Ok(()) if all rules are satisfied, or Err(String) with a detailed description of any violation.
    pub fn validate_invariants(&self) -> Result<(), String> {
        // 1. Validate nil node at index 0
        if self.nodes.is_empty() {
            return Err("Nodes vector is empty, missing NIL node at index 0".to_string());
        }
        let nil = &self.nodes[0];
        if nil.color != Color::Black {
            return Err(format!(
                "NIL node (index 0) must be Black, but is {:?}",
                nil.color
            ));
        }
        if nil.left != 0 || nil.right != 0 {
            return Err(format!(
                "NIL node (index 0) must have left=0 and right=0, found left={}, right={}",
                nil.left, nil.right
            ));
        }
        if nil.subtree_length != 0 || nil.left_subtree_length != 0 || nil.length != 0 {
            return Err(format!(
                "NIL node (index 0) must have length=0 and subtree_lengths=0, found length={}, subtree_len={}, left_subtree_len={}",
                nil.length, nil.subtree_length, nil.left_subtree_length
            ));
        }

        // 2. If tree is empty (root == 0)
        let root = self.red_black_tree.root_node;
        if root == 0 {
            if self.red_black_tree.black_height != 0 {
                return Err(format!(
                    "Empty tree must have black_height 0, found {}",
                    self.red_black_tree.black_height
                ));
            }
            return Ok(());
        }

        if root >= self.nodes.len() {
            return Err(format!(
                "Root node index {} is out of bounds (nodes len {})",
                root,
                self.nodes.len()
            ));
        }

        // 3. Root property: Root must be Black
        if self.nodes[root].color != Color::Black {
            return Err(format!(
                "Root node {} must be Black, but is {:?}",
                root, self.nodes[root].color
            ));
        }

        // 4. Root parent must be 0
        if self.nodes[root].parent != 0 {
            return Err(format!(
                "Root node {} must have parent 0, but has parent {}",
                root, self.nodes[root].parent
            ));
        }

        // 5. Reachability, cycle detection, and parent-child consistency
        let mut visited = std::collections::HashSet::new();
        self.validate_node_structure(root, &mut visited)?;

        // 6. Red-Black Rule: No two consecutive Red nodes (Red node has Black children and Black parent)
        for &idx in &visited {
            let node = &self.nodes[idx];
            if node.color == Color::Red {
                if node.parent != 0 && self.nodes[node.parent].color == Color::Red {
                    return Err(format!(
                        "Red-Red violation: Red node {} has Red parent {}",
                        idx, node.parent
                    ));
                }
                if node.left != 0 && self.nodes[node.left].color == Color::Red {
                    return Err(format!(
                        "Red-Red violation: Red node {} has Red left child {}",
                        idx, node.left
                    ));
                }
                if node.right != 0 && self.nodes[node.right].color == Color::Red {
                    return Err(format!(
                        "Red-Red violation: Red node {} has Red right child {}",
                        idx, node.right
                    ));
                }
            }
        }

        // 7. Red-Black Rule: Black Height consistency across all paths to NIL leaves
        let computed_bh = self.validate_black_height(root)?;
        if computed_bh != self.red_black_tree.black_height {
            return Err(format!(
                "Tree black_height attribute ({}) does not match computed black height ({})",
                self.red_black_tree.black_height, computed_bh
            ));
        }

        // 8. Augmented PieceTree metadata: subtree_length and left_subtree_length
        for &idx in &visited {
            let node = &self.nodes[idx];
            let left_len = self.nodes[node.left].subtree_length;
            let right_len = self.nodes[node.right].subtree_length;
            let expected_subtree_len = left_len + right_len + node.length;

            if node.left_subtree_length != left_len {
                return Err(format!(
                    "Node {} left_subtree_length is {}, but left child ({}) subtree_length is {}",
                    idx, node.left_subtree_length, node.left, left_len
                ));
            }
            if node.subtree_length != expected_subtree_len {
                return Err(format!(
                    "Node {} subtree_length is {}, but expected left({}) + right({}) + length({}) = {}",
                    idx,
                    node.subtree_length,
                    left_len,
                    right_len,
                    node.length,
                    expected_subtree_len
                ));
            }

            // Buffer bounds check
            match node.buffer_type {
                BufferType::Original => {
                    if node.start + node.length > self.original.len() {
                        return Err(format!(
                            "Node {} slice [{}..{}] exceeds original buffer length {}",
                            idx,
                            node.start,
                            node.start + node.length,
                            self.original.len()
                        ));
                    }
                }
                BufferType::Add => {
                    if node.start + node.length > self.add.len() {
                        return Err(format!(
                            "Node {} slice [{}..{}] exceeds add buffer length {}",
                            idx,
                            node.start,
                            node.start + node.length,
                            self.add.len()
                        ));
                    }
                }
            }
        }

        // 9. In-order text length consistency
        let root_subtree_len = self.nodes[root].subtree_length;
        let actual_text_len = self.get_text().len();
        if root_subtree_len != actual_text_len {
            return Err(format!(
                "Root subtree_length ({}) does not match get_text() length ({})",
                root_subtree_len, actual_text_len
            ));
        }

        Ok(())
    }

    fn validate_node_structure(
        &self,
        idx: usize,
        visited: &mut std::collections::HashSet<usize>,
    ) -> Result<(), String> {
        if idx == 0 {
            return Ok(());
        }
        if idx >= self.nodes.len() {
            return Err(format!(
                "Node index {} is out of bounds (len: {})",
                idx,
                self.nodes.len()
            ));
        }
        if !visited.insert(idx) {
            return Err(format!(
                "Cycle detected in tree: node {} visited more than once",
                idx
            ));
        }

        let node = &self.nodes[idx];
        if node.left != 0 {
            if node.left >= self.nodes.len() {
                return Err(format!(
                    "Node {} left child {} is out of bounds",
                    idx, node.left
                ));
            }
            if self.nodes[node.left].parent != idx {
                return Err(format!(
                    "Parent link mismatch: node {} left child is {}, but node {} parent is {}",
                    idx, node.left, node.left, self.nodes[node.left].parent
                ));
            }
            self.validate_node_structure(node.left, visited)?;
        }

        if node.right != 0 {
            if node.right >= self.nodes.len() {
                return Err(format!(
                    "Node {} right child {} is out of bounds",
                    idx, node.right
                ));
            }
            if self.nodes[node.right].parent != idx {
                return Err(format!(
                    "Parent link mismatch: node {} right child is {}, but node {} parent is {}",
                    idx, node.right, node.right, self.nodes[node.right].parent
                ));
            }
            self.validate_node_structure(node.right, visited)?;
        }

        Ok(())
    }

    fn validate_black_height(&self, idx: usize) -> Result<usize, String> {
        if idx == 0 {
            return Ok(0);
        }
        let node = &self.nodes[idx];
        let left_bh = self.validate_black_height(node.left)?;
        let right_bh = self.validate_black_height(node.right)?;

        if left_bh != right_bh {
            return Err(format!(
                "Black height mismatch at node {}: left subtree black height is {}, right subtree black height is {}",
                idx, left_bh, right_bh
            ));
        }

        let is_black = if node.color == Color::Black { 1 } else { 0 };
        Ok(left_bh + is_black)
    }

    /// Panics if any Red-Black Tree or PieceTree invariant is violated.
    pub fn assert_invariants(&self) {
        if let Err(err) = self.validate_invariants() {
            panic!(
                "Red-Black Tree invariant violated: {}\nTree structure:\n{}",
                err,
                self.debug_dump()
            );
        }
    }

    /// Returns the number of active nodes in the tree (excluding NIL node).
    pub fn node_count(&self) -> usize {
        let mut count = 0;
        let mut stack = vec![self.red_black_tree.root_node];
        while let Some(idx) = stack.pop() {
            if idx != 0 && idx < self.nodes.len() {
                count += 1;
                stack.push(self.nodes[idx].left);
                stack.push(self.nodes[idx].right);
            }
        }
        count
    }

    /// Returns the maximum height/depth of the tree from root to leaf.
    pub fn tree_height(&self) -> usize {
        fn depth(nodes: &[Node], idx: usize) -> usize {
            if idx == 0 {
                0
            } else {
                1 + depth(nodes, nodes[idx].left).max(depth(nodes, nodes[idx].right))
            }
        }
        depth(&self.nodes, self.red_black_tree.root_node)
    }

    /// Returns the tree's recorded black height.
    pub fn black_height(&self) -> usize {
        self.red_black_tree.black_height
    }

    /// Returns root node index.
    pub fn root_node_index(&self) -> usize {
        self.red_black_tree.root_node
    }

    /// Returns the root node's subtree_length metadata, or 0 if tree is empty.
    pub fn root_subtree_length(&self) -> usize {
        if self.red_black_tree.root_node != 0 && self.red_black_tree.root_node < self.nodes.len() {
            self.nodes[self.red_black_tree.root_node].subtree_length
        } else {
            0
        }
    }

    /// Returns the total sum of `length` across all active nodes in the tree.
    pub fn sum_node_lengths(&self) -> usize {
        let mut total = 0;
        let mut stack = vec![self.red_black_tree.root_node];
        while let Some(idx) = stack.pop() {
            if idx != 0 && idx < self.nodes.len() {
                total += self.nodes[idx].length;
                stack.push(self.nodes[idx].left);
                stack.push(self.nodes[idx].right);
            }
        }
        total
    }

    /// Returns detailed metadata for all reachable active nodes in the tree.
    pub fn get_all_nodes_info(&self) -> Vec<NodeInfo> {
        let mut result = Vec::new();
        let mut stack = vec![self.red_black_tree.root_node];
        while let Some(idx) = stack.pop() {
            if idx != 0 && idx < self.nodes.len() {
                let n = &self.nodes[idx];
                result.push(NodeInfo {
                    index: idx,
                    length: n.length,
                    subtree_length: n.subtree_length,
                    left_subtree_length: n.left_subtree_length,
                    left: n.left,
                    right: n.right,
                    parent: n.parent,
                    is_red: n.color == Color::Red,
                });
                stack.push(n.left);
                stack.push(n.right);
            }
        }
        result
    }

    /// Generates a visual debug dump of the tree hierarchy.
    pub fn debug_dump(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "Tree [root={}, black_height={}, nodes_len={}, doc_len={}]:\n",
            self.red_black_tree.root_node,
            self.red_black_tree.black_height,
            self.nodes.len(),
            if self.red_black_tree.root_node != 0
                && self.red_black_tree.root_node < self.nodes.len()
            {
                self.nodes[self.red_black_tree.root_node].subtree_length
            } else {
                0
            }
        ));
        self.dump_node(self.red_black_tree.root_node, 0, &mut s, "ROOT");
        s
    }

    fn dump_node(&self, idx: usize, indent: usize, out: &mut String, prefix: &str) {
        let pad = "  ".repeat(indent);
        if idx == 0 {
            out.push_str(&format!("{}{} NIL (0)\n", pad, prefix));
            return;
        }
        if idx >= self.nodes.len() {
            out.push_str(&format!("{}{} INVALID_IDX ({})\n", pad, prefix, idx));
            return;
        }
        let node = &self.nodes[idx];
        let color_char = match node.color {
            Color::Red => "RED",
            Color::Black => "BLACK",
        };
        let buf_name = match node.buffer_type {
            BufferType::Original => "Orig",
            BufferType::Add => "Add",
        };
        out.push_str(&format!(
            "{}{} [{}] Node#{}: color={}, buf={}[{}..{}], len={}, sub_len={}, left_sub_len={}, parent={}\n",
            pad, prefix, color_char, idx, color_char, buf_name, node.start, node.start + node.length,
            node.length, node.subtree_length, node.left_subtree_length, node.parent
        ));
        self.dump_node(node.left, indent + 1, out, "L--");
        self.dump_node(node.right, indent + 1, out, "R--");
    }
}

/// Metadata snapshot for a node in the Red-Black tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    pub index: usize,
    pub length: usize,
    pub subtree_length: usize,
    pub left_subtree_length: usize,
    pub left: usize,
    pub right: usize,
    pub parent: usize,
    pub is_red: bool,
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum PathSpecifier {
    Smaller,
    Larger,
    Equal,
}
