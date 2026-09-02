use textio::PieceTree;

struct TestRng {
    state: u64,
}

impl TestRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        if min >= max {
            return min;
        }
        min + (self.next_u64() as usize % (max - min))
    }
}

/// =========================================================================
/// 1. STRICT RED-BLACK TREE INVARIANT CHECKING AFTER EVERY OPERATION
/// =========================================================================

#[test]
fn test_rb_invariants_checked_after_every_single_operation_in_churn() {
    let mut tree = PieceTree::new("Genesis block.\n");
    let mut oracle = String::from("Genesis block.\n");

    let mut rng = TestRng::new(0xFEEDFACE_CAFE);

    // 1,000 steps with strict invariant assert at EVERY SINGLE STEP
    for step in 0..1000 {
        let is_insert = oracle.len() < 5 || (rng.next_u64() % 100) < 60;

        if is_insert {
            let snippet = format!("item_{:03}_", step);
            let pos = if oracle.is_empty() {
                0
            } else {
                rng.next_range(0, oracle.len() + 1)
            };
            tree.insert(pos, &snippet);
            oracle.insert_str(pos, &snippet);
        } else {
            let start = rng.next_range(0, oracle.len());
            let max_del = 15.min(oracle.len() - start);
            let del_len = rng.next_range(1, max_del + 1);

            tree.delete(start, del_len);
            oracle.replace_range(start..start + del_len, "");
        }

        // Must pass full validation at every single step!
        tree.assert_invariants();
        assert_eq!(tree.get_text(), oracle, "Content mismatch at step {}", step);
    }
}

/// =========================================================================
/// 2. DEEP NODE-BY-NODE POINTER AND METADATA RECURSIVE VERIFICATION
/// =========================================================================

#[test]
fn test_exhaustive_node_metadata_and_pointer_integrity() {
    let mut tree = PieceTree::new("Base piece text for metadata exploration.\n");

    // Insert 200 pieces at varied positions
    for i in 0..200 {
        let pos = (i * 23) % (tree.get_text().len() + 1);
        let s = format!("[N{:03}:payload]", i);
        tree.insert(pos, &s);
    }

    tree.assert_invariants();

    let all_nodes = tree.get_all_nodes_info();
    let node_map: std::collections::HashMap<usize, &textio::NodeInfo> =
        all_nodes.iter().map(|n| (n.index, n)).collect();

    // Verify properties of every single node in the tree
    for node in &all_nodes {
        // 1. Left child relationship
        if node.left != 0 {
            let left_child = node_map.get(&node.left).expect("Left child must exist");
            assert_eq!(left_child.parent, node.index, "Left child's parent must point back to node");
            assert_eq!(node.left_subtree_length, left_child.subtree_length, "left_subtree_length must match left child subtree_length");
        } else {
            assert_eq!(node.left_subtree_length, 0, "NIL left child must mean left_subtree_length == 0");
        }

        // 2. Right child relationship
        if node.right != 0 {
            let right_child = node_map.get(&node.right).expect("Right child must exist");
            assert_eq!(right_child.parent, node.index, "Right child's parent must point back to node");
        }

        // 3. Subtree length consistency
        let left_sub = if node.left != 0 { node_map[&node.left].subtree_length } else { 0 };
        let right_sub = if node.right != 0 { node_map[&node.right].subtree_length } else { 0 };
        assert_eq!(
            node.subtree_length,
            left_sub + right_sub + node.length,
            "Node {} subtree_length must equal left + right + length",
            node.index
        );

        // 4. Red-Black color rules: Red node must not have red parent
        if node.is_red && node.parent != 0 {
            let parent = node_map.get(&node.parent).expect("Parent must exist");
            assert!(!parent.is_red, "Red node cannot have Red parent (Red-Red violation)");
        }
    }
}

/// =========================================================================
/// 3. REPEATED DRAIN AND REBUILD LIFECYCLE (TREE RECYCLING)
/// =========================================================================

#[test]
fn test_repeated_tree_drain_and_rebuild_cycles() {
    let mut tree = PieceTree::new("Seed");

    for cycle in 0..10 {
        // Grow to 100 pieces
        for i in 0..100 {
            let pos = tree.get_text().len();
            tree.insert(pos, &format!("c{}_{} ", cycle, i));
        }

        tree.assert_invariants();
        assert!(tree.node_count() >= 100);
        assert!(tree.black_height() >= 3);

        // Delete entire tree contents to empty
        let total_len = tree.get_text().len();
        tree.delete(0, total_len);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), "", "Tree must be empty at end of cycle {}", cycle);
        assert_eq!(tree.node_count(), 0, "Node count must be 0 after full drain in cycle {}", cycle);
        assert_eq!(tree.black_height(), 0, "Black height must be 0 after full drain in cycle {}", cycle);
        assert_eq!(tree.root_node_index(), 0, "Root node must be 0 after full drain in cycle {}", cycle);
    }
}

/// =========================================================================
/// 4. LOGARITHMIC DEPTH GUARANTEES UNDER ADVERSARIAL SEQUENCES
/// =========================================================================

#[test]
fn test_adversarial_zigzag_and_middle_deletion_depth_bounds() {
    let mut tree = PieceTree::new("root");

    // Adversarial zig-zag inserts: 200 nodes
    for i in 0..200 {
        if i % 2 == 0 {
            tree.insert(0, &format!("L{:03}_", i));
        } else {
            let end = tree.get_text().len();
            tree.insert(end, &format!("_R{:03}", i));
        }
    }

    tree.assert_invariants();

    let n = tree.node_count();
    let bh = tree.black_height();
    let h = tree.tree_height();

    // Red-black tree mathematical bounds:
    // 1. bh >= h / 2
    // 2. h <= 2 * log2(n + 1) + 1
    let max_theoretical_height = 2 * ((n + 1) as f64).log2().ceil() as usize + 1;
    assert!(h <= max_theoretical_height, "Tree height {} exceeded theoretical limit {} for {} nodes", h, max_theoretical_height, n);
    assert!(bh <= h, "Black height {} cannot exceed tree height {}", bh, h);
    assert!(h <= 2 * bh + 1, "Tree height {} cannot exceed 2 * bh + 1 ({})", h, 2 * bh + 1);

    // Now delete every other piece from the middle outwards
    for _ in 0..50 {
        let text = tree.get_text();
        if text.len() < 10 {
            break;
        }
        let mid = text.len() / 2;
        tree.delete(mid - 2, 4);
        tree.assert_invariants();
    }

    let n_after = tree.node_count();
    let h_after = tree.tree_height();
    let max_h_after = 2 * ((n_after + 1) as f64).log2().ceil() as usize + 2;
    assert!(h_after <= max_h_after, "Tree height {} after deletions exceeded max {}", h_after, max_h_after);
}

/// =========================================================================
/// 5. IN-ORDER PIECE CONTINUITY AND SUM-OF-LENGTHS INTEGRITY
/// =========================================================================

#[test]
fn test_in_order_piece_continuity_under_fuzzing() {
    let mut tree = PieceTree::new("Start of document stream.\n");
    let mut oracle = String::from("Start of document stream.\n");

    let mut rng = TestRng::new(0x7788_9900_AABB);

    for _ in 0..300 {
        let is_insert = oracle.len() < 10 || (rng.next_u64() % 100) < 65;
        if is_insert {
            let s = format!("w_{} ", rng.next_u64() % 1000);
            let pos = rng.next_range(0, oracle.len() + 1);
            tree.insert(pos, &s);
            oracle.insert_str(pos, &s);
        } else {
            let start = rng.next_range(0, oracle.len());
            let len = rng.next_range(1, 10.min(oracle.len() - start) + 1);
            tree.delete(start, len);
            oracle.replace_range(start..start + len, "");
        }

        // Validate that root_subtree_length == sum_node_lengths == get_text().len() == oracle.len()
        let root_sub = tree.root_subtree_length();
        let sum_lens = tree.sum_node_lengths();
        let text_len = tree.get_text().len();

        assert_eq!(root_sub, oracle.len(), "root_subtree_length mismatch with oracle");
        assert_eq!(sum_lens, oracle.len(), "sum_node_lengths mismatch with oracle");
        assert_eq!(text_len, oracle.len(), "get_text().len() mismatch with oracle");
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}
