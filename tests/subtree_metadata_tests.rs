use textio::PieceTree;

/// =========================================================================
/// 1. BASELINE METADATA ON INITIALIZATION
/// =========================================================================

#[test]
fn test_metadata_empty_tree() {
    let tree = PieceTree::new("");
    tree.assert_invariants();
    assert_eq!(tree.root_subtree_length(), 0);
    assert_eq!(tree.sum_node_lengths(), 0);
    assert_eq!(tree.get_all_nodes_info().len(), 0);
}

#[test]
fn test_metadata_single_piece_tree() {
    let content = "Hello, Piece Tree Metadata!";
    let tree = PieceTree::new(content);
    tree.assert_invariants();

    assert_eq!(tree.root_subtree_length(), content.len());
    assert_eq!(tree.sum_node_lengths(), content.len());

    let nodes = tree.get_all_nodes_info();
    assert_eq!(nodes.len(), 1);
    let root = &nodes[0];
    assert_eq!(root.length, content.len());
    assert_eq!(root.subtree_length, content.len());
    assert_eq!(root.left_subtree_length, 0);
    assert_eq!(root.left, 0);
    assert_eq!(root.right, 0);
    assert_eq!(root.parent, 0);
}

/// =========================================================================
/// 2. METADATA PROPAGATION ON APPENDS & PREPENDS
/// =========================================================================

#[test]
fn test_metadata_updates_on_sequential_appends() {
    let mut tree = PieceTree::new("Initial");
    let chunks = [" - A", " - BB", " - CCC", " - DDDD", " - EEEEE", " - FFFFFF"];
    let mut expected_len = "Initial".len();

    for (i, &chunk) in chunks.iter().enumerate() {
        let pos = tree.get_text().len();
        tree.insert(pos, chunk);
        expected_len += chunk.len();

        tree.assert_invariants();
        assert_eq!(tree.root_subtree_length(), expected_len);
        assert_eq!(tree.sum_node_lengths(), expected_len);
        assert_eq!(tree.get_text().len(), expected_len);

        // Verify each individual node's subtree_length and left_subtree_length
        let nodes = tree.get_all_nodes_info();
        let nodes_by_idx: std::collections::HashMap<usize, &textio::NodeInfo> =
            nodes.iter().map(|n| (n.index, n)).collect();

        for n in &nodes {
            let left_len = if n.left != 0 {
                nodes_by_idx.get(&n.left).unwrap().subtree_length
            } else {
                0
            };
            let right_len = if n.right != 0 {
                nodes_by_idx.get(&n.right).unwrap().subtree_length
            } else {
                0
            };

            assert_eq!(
                n.left_subtree_length, left_len,
                "Node {} left_subtree_length mismatch at step {}", n.index, i
            );
            assert_eq!(
                n.subtree_length, left_len + right_len + n.length,
                "Node {} subtree_length mismatch at step {}", n.index, i
            );
        }
    }
}

#[test]
fn test_metadata_updates_on_sequential_prepends() {
    let mut tree = PieceTree::new("End");
    let chunks = ["First: ", "Second: ", "Third: ", "Fourth: ", "Fifth: "];
    let mut expected_len = "End".len();

    for (i, &chunk) in chunks.iter().enumerate() {
        tree.insert(0, chunk);
        expected_len += chunk.len();

        tree.assert_invariants();
        assert_eq!(tree.root_subtree_length(), expected_len);
        assert_eq!(tree.sum_node_lengths(), expected_len);

        let nodes = tree.get_all_nodes_info();
        let nodes_by_idx: std::collections::HashMap<usize, &textio::NodeInfo> =
            nodes.iter().map(|n| (n.index, n)).collect();

        for n in &nodes {
            let left_len = if n.left != 0 {
                nodes_by_idx.get(&n.left).unwrap().subtree_length
            } else {
                0
            };
            let right_len = if n.right != 0 {
                nodes_by_idx.get(&n.right).unwrap().subtree_length
            } else {
                0
            };

            assert_eq!(n.left_subtree_length, left_len, "Prepend step {} node {} left_subtree_length mismatch", i, n.index);
            assert_eq!(n.subtree_length, left_len + right_len + n.length, "Prepend step {} node {} subtree_length mismatch", i, n.index);
        }
    }
}

/// =========================================================================
/// 3. METADATA ON PIECE SPLITTING (SPLIT-AND-INSERT)
/// =========================================================================

#[test]
fn test_metadata_on_middle_piece_split() {
    // "0123456789" (length 10)
    let mut tree = PieceTree::new("0123456789");
    assert_eq!(tree.root_subtree_length(), 10);

    // Insert "ABC" (length 3) at index 4 -> "0123ABC456789" (total 13)
    // Left piece: "0123" (len 4)
    // New piece: "ABC" (len 3)
    // Right piece: "456789" (len 6)
    tree.insert(4, "ABC");
    tree.assert_invariants();

    assert_eq!(tree.get_text(), "0123ABC456789");
    assert_eq!(tree.root_subtree_length(), 13);
    assert_eq!(tree.sum_node_lengths(), 13);

    let nodes = tree.get_all_nodes_info();
    assert_eq!(nodes.len(), 3);

    let nodes_by_idx: std::collections::HashMap<usize, &textio::NodeInfo> =
        nodes.iter().map(|n| (n.index, n)).collect();

    for n in &nodes {
        let left_len = if n.left != 0 { nodes_by_idx.get(&n.left).unwrap().subtree_length } else { 0 };
        let right_len = if n.right != 0 { nodes_by_idx.get(&n.right).unwrap().subtree_length } else { 0 };
        assert_eq!(n.left_subtree_length, left_len);
        assert_eq!(n.subtree_length, left_len + right_len + n.length);
    }
}

#[test]
fn test_metadata_on_nested_cascading_splits() {
    let mut tree = PieceTree::new("()");

    for i in 0..40 {
        let mid = tree.get_text().len() / 2;
        let token = format!("[{}]", i);
        tree.insert(mid, &token);

        tree.assert_invariants();
        assert_eq!(tree.root_subtree_length(), tree.get_text().len());
        assert_eq!(tree.sum_node_lengths(), tree.get_text().len());
    }
}

/// =========================================================================
/// 4. METADATA CONSISTENCY ACROSS ROTATIONS (REBALANCING)
/// =========================================================================

#[test]
fn test_metadata_consistency_during_rebalancing_rotations() {
    let mut tree = PieceTree::new("root");

    // Adding 60 elements to force numerous left & right rotations
    for i in 0..60 {
        let chunk = format!("node_{:03} ", i);
        let pos = (i * 29) % (tree.get_text().len() + 1);
        tree.insert(pos, &chunk);

        tree.assert_invariants();

        let doc_len = tree.get_text().len();
        assert_eq!(tree.root_subtree_length(), doc_len);
        assert_eq!(tree.sum_node_lengths(), doc_len);

        let nodes = tree.get_all_nodes_info();
        let nodes_by_idx: std::collections::HashMap<usize, &textio::NodeInfo> =
            nodes.iter().map(|n| (n.index, n)).collect();

        for n in &nodes {
            let left_len = if n.left != 0 {
                nodes_by_idx.get(&n.left).unwrap().subtree_length
            } else {
                0
            };
            let right_len = if n.right != 0 {
                nodes_by_idx.get(&n.right).unwrap().subtree_length
            } else {
                0
            };

            assert_eq!(n.left_subtree_length, left_len, "Rotations: node {} left_subtree_length mismatch", n.index);
            assert_eq!(n.subtree_length, left_len + right_len + n.length, "Rotations: node {} subtree_length mismatch", n.index);
        }
    }
}

/// =========================================================================
/// 5. METADATA ON DELETIONS
/// =========================================================================

#[test]
fn test_metadata_on_prefix_trim_deletion() {
    let mut tree = PieceTree::new("0123456789");
    // Trim prefix "012" (len 3) -> "3456789" (len 7)
    tree.delete(0, 3);
    tree.assert_invariants();
    assert_eq!(tree.root_subtree_length(), 7);
    assert_eq!(tree.sum_node_lengths(), 7);
    assert_eq!(tree.get_text(), "3456789");
}

#[test]
fn test_metadata_on_suffix_trim_deletion() {
    let mut tree = PieceTree::new("0123456789");
    // Trim suffix "789" (len 3) -> "0123456" (len 7)
    tree.delete(7, 3);
    tree.assert_invariants();
    assert_eq!(tree.root_subtree_length(), 7);
    assert_eq!(tree.sum_node_lengths(), 7);
    assert_eq!(tree.get_text(), "0123456");
}

#[test]
fn test_metadata_on_middle_slice_deletion() {
    let mut tree = PieceTree::new("abcdefghij");
    // Delete "cdef" (start 2, len 4) -> "abghij" (len 6)
    tree.delete(2, 4);
    tree.assert_invariants();
    assert_eq!(tree.root_subtree_length(), 6);
    assert_eq!(tree.sum_node_lengths(), 6);
    assert_eq!(tree.get_text(), "abghij");
}

/// =========================================================================
/// 6. METADATA INTEGRITY IN LARGE SIMULATED TREES
/// =========================================================================

#[test]
fn test_metadata_integrity_in_large_simulated_tree() {
    let mut tree = PieceTree::new("INIT_DOCUMENT");

    // Construct a rich tree with 80 pieces of varying lengths
    for i in 0..80 {
        let chunk = format!("[CH_{}_{}]", i, "x".repeat((i % 7) + 1));
        let pos = (i * 43) % (tree.get_text().len() + 1);
        tree.insert(pos, &chunk);
    }

    tree.assert_invariants();
    let text = tree.get_text();
    assert_eq!(tree.root_subtree_length(), text.len());
    assert_eq!(tree.sum_node_lengths(), text.len());

    let nodes = tree.get_all_nodes_info();
    let nodes_by_idx: std::collections::HashMap<usize, &textio::NodeInfo> =
        nodes.iter().map(|n| (n.index, n)).collect();

    for n in &nodes {
        let left_len = if n.left != 0 {
            nodes_by_idx.get(&n.left).unwrap().subtree_length
        } else {
            0
        };
        let right_len = if n.right != 0 {
            nodes_by_idx.get(&n.right).unwrap().subtree_length
        } else {
            0
        };

        assert_eq!(n.left_subtree_length, left_len);
        assert_eq!(n.subtree_length, left_len + right_len + n.length);
    }
}
