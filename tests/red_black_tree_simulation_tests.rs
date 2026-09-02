use textio::PieceTree;

/// =========================================================================
/// 1. INDIVIDUAL RED-BLACK TREE RULE VERIFICATION TESTS
/// =========================================================================

/// Rule 1 & 2: Root is always Black (or 0 for empty tree) and NIL node is Black.
#[test]
fn test_rb_rule_root_and_nil_are_black() {
    let empty = PieceTree::new("");
    empty.assert_invariants();
    assert_eq!(empty.root_node_index(), 0);
    assert_eq!(empty.black_height(), 0);

    let mut tree = PieceTree::new("Initial Root");
    tree.assert_invariants();
    assert_ne!(tree.root_node_index(), 0);
    assert_eq!(tree.black_height(), 1);

    // Insert 50 elements and ensure root remains Black after every rotation/recoloring
    for i in 0..50 {
        tree.insert(0, &format!("{}_", i));
        tree.assert_invariants();
    }
}

/// Rule 4: No two consecutive Red nodes (Red node has only Black children & Black parent).
#[test]
fn test_rb_rule_no_consecutive_red_nodes() {
    let mut tree = PieceTree::new("A");
    for i in 0..100 {
        // Insert at alternating locations to stress test recoloring and rotation branches
        let pos = if i % 2 == 0 { 0 } else { tree.get_text().len() };
        tree.insert(pos, &format!("N{:03}", i));
        tree.assert_invariants();
    }
}

/// Rule 5: Uniform Black Height across all paths from root to NIL leaves.
#[test]
fn test_rb_rule_uniform_black_height_all_paths() {
    let mut tree = PieceTree::new("Base");
    for i in 0..80 {
        let len = tree.get_text().len();
        let pos = (i * 17) % (len + 1);
        tree.insert(pos, &format!("[B{}]", i));

        // validate_invariants explicitly verifies that every branch to NIL has identical black height
        assert!(tree.validate_invariants().is_ok());
    }
}

/// Rule 6: Bidirectional Parent-Child Pointer Consistency & Acyclic Structure.
#[test]
fn test_rb_rule_parent_child_pointers_consistency() {
    let mut tree = PieceTree::new("Start");
    for i in 0..60 {
        let pos = tree.get_text().len() / 2;
        tree.insert(pos, &format!("split_{}", i));
        tree.assert_invariants();
    }
}

/// Rule 7: Augmented Subtree Lengths & Left Subtree Lengths Integrity.
#[test]
fn test_rb_rule_augmented_subtree_lengths_integrity() {
    let mut tree = PieceTree::new("0123456789");
    for i in 0..50 {
        let pos = (i * 13) % (tree.get_text().len() + 1);
        let s = format!("-{:02}-", i);
        tree.insert(pos, &s);
        tree.assert_invariants();
    }
}

/// =========================================================================
/// 2. CONCEPTUAL GROWTH PATTERN SIMULATIONS (BIG RED-BLACK TREES)
/// =========================================================================

/// Simulation 1: Monotonic Right-Spine Growth (Sequential Appends)
/// In an unbalanced tree, appending N items degenerates into a singly-linked list of depth N.
/// In a Red-Black tree, left rotations and uncle-recoloring rebalance the tree so height is O(log N).
#[test]
fn test_simulation_sequential_appends_right_spine_rotations() {
    let mut tree = PieceTree::new("");
    let mut expected = String::new();

    const NUM_PIECES: usize = 120;
    for i in 0..NUM_PIECES {
        let chunk = format!("item_{:04}; ", i);
        let insert_pos = tree.get_text().len();
        tree.insert(insert_pos, &chunk);
        expected.push_str(&chunk);

        // Verify RB invariants on every single append step
        tree.assert_invariants();
        assert_eq!(tree.get_text(), expected);

        // RB height property: height <= 2 * floor(log2(n + 1)) + 1
        let n = tree.node_count();
        let max_allowed_height = 2 * ((n + 1) as f64).log2().ceil() as usize + 1;
        assert!(
            tree.tree_height() <= max_allowed_height,
            "Tree height {} exceeded theoretical max {} for {} nodes",
            tree.tree_height(),
            max_allowed_height,
            n
        );
    }

    assert_eq!(tree.node_count(), NUM_PIECES);
    assert!(tree.tree_height() <= 12);
    assert!(tree.black_height() >= 3 && tree.black_height() <= 7);
}

/// Simulation 2: Monotonic Left-Spine Growth (Sequential Prepends)
/// Prepending N items at index 0 triggers right rotations and recoloring upward.
#[test]
fn test_simulation_sequential_prepends_left_spine_rotations() {
    let mut tree = PieceTree::new("INIT");
    let mut expected = String::from("INIT");

    const NUM_PIECES: usize = 100;
    for i in 0..NUM_PIECES {
        let chunk = format!("[pre_{:03}]", i);
        tree.insert(0, &chunk);
        expected.insert_str(0, &chunk);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), expected);
    }

    assert_eq!(tree.node_count(), NUM_PIECES + 1);
    assert!(tree.tree_height() <= 12);
    assert!(tree.black_height() >= 3);
}

/// Simulation 3: Deep Nested Middle Splits (Split-and-Insert Cascades)
/// Inserting in the middle of pieces causes piece splitting, generating 2 pieces per split
/// and testing parent-to-child links, left_subtree_length metadata recalculation, and rotations.
#[test]
fn test_simulation_deep_nested_middle_splits() {
    let mut tree = PieceTree::new("<<<<>>>>");
    let mut expected = String::from("<<<<>>>>");

    for i in 0..60 {
        let mid = tree.get_text().len() / 2;
        let token = format!("<{}>", i);
        tree.insert(mid, &token);
        expected.insert_str(mid, &token);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), expected);
    }

    assert!(tree.node_count() >= 60);
    assert!(tree.tree_height() <= 14);
}

/// Simulation 4: Binary Subdivision Balanced Tree Simulation
/// Inserts elements at fractional subdivisions (1/2, 1/4, 3/4, 1/8, ...)
/// simulating optimal balanced piece tree population.
#[test]
fn test_simulation_binary_subdivision_balanced_tree() {
    let mut tree = PieceTree::new("0_______________________________1");
    let mut expected = String::from("0_______________________________1");

    for step in 1..=50 {
        let doc_len = tree.get_text().len();
        let pos = (doc_len * step * 7) % (doc_len - 1) + 1;
        let snippet = format!("[s{}]", step);

        tree.insert(pos, &snippet);
        expected.insert_str(pos, &snippet);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), expected);
    }

    assert!(tree.node_count() >= 50);
}

/// Simulation 5: Zig-Zag Alternating Insertions (Case 2 & Case 3 Rotations)
/// Alternates inserting at index 0 and at the end of the document,
/// specifically exercising left-right and right-left double rotation rebalancing cases.
#[test]
fn test_simulation_zigzag_alternating_insertions() {
    let mut tree = PieceTree::new("CENTER");
    let mut expected = String::from("CENTER");

    for i in 0..50 {
        // Prepend on left
        let left_chunk = format!("L{:02}-", i);
        tree.insert(0, &left_chunk);
        expected.insert_str(0, &left_chunk);
        tree.assert_invariants();

        // Append on right
        let right_chunk = format!("-R{:02}", i);
        let end_pos = tree.get_text().len();
        tree.insert(end_pos, &right_chunk);
        expected.push_str(&right_chunk);
        tree.assert_invariants();

        assert_eq!(tree.get_text(), expected);
    }

    assert_eq!(tree.node_count(), 101);
    assert!(tree.tree_height() <= 12);
}

/// Simulation 6: Massive Scale Insertions (300+ Nodes)
/// Constructs a massive tree to test deep rebalancing cascades and stack safety.
#[test]
fn test_simulation_massive_scale_300_nodes() {
    let mut tree = PieceTree::new("LargeTree");
    for i in 0..300 {
        let end = tree.get_text().len();
        tree.insert(end, &format!("_{}", i));
    }
    tree.assert_invariants();
    assert_eq!(tree.node_count(), 301);
    assert!(tree.tree_height() <= 16);
    assert!(tree.black_height() >= 4 && tree.black_height() <= 9);
}

/// =========================================================================
/// 3. CONCEPTUAL DELETION PATTERN SIMULATIONS (BIG RED-BLACK TREES)
/// =========================================================================

/// Simulation 7: Right-to-Left Systematic Deletion
/// Builds a large tree with 60 pieces and deletes them one by one from the right edge.
#[test]
fn test_simulation_delete_from_large_tree_right_to_left() {
    let mut tree = PieceTree::new("");
    const CHUNK_COUNT: usize = 60;
    const CHUNK_SIZE: usize = 5;

    for i in 0..CHUNK_COUNT {
        let chunk = format!("{:04} ", i);
        let end = tree.get_text().len();
        tree.insert(end, &chunk);
    }
    tree.assert_invariants();
    assert_eq!(tree.node_count(), CHUNK_COUNT);

    while tree.get_text().len() > 0 {
        let len = tree.get_text().len();
        let del_len = CHUNK_SIZE.min(len);
        let start = len - del_len;
        tree.delete(start, del_len);
        tree.assert_invariants();
    }

    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.node_count(), 0);
    assert_eq!(tree.black_height(), 0);
}

/// Simulation 8: Left-to-Right Systematic Deletion
/// Builds a large tree and deletes from index 0 repeatedly.
#[test]
fn test_simulation_delete_from_large_tree_left_to_right() {
    let mut tree = PieceTree::new("");
    const CHUNK_COUNT: usize = 50;

    for i in 0..CHUNK_COUNT {
        let chunk = format!("{:03}-", i);
        let end = tree.get_text().len();
        tree.insert(end, &chunk);
    }
    tree.assert_invariants();

    for _ in 0..CHUNK_COUNT {
        tree.delete(0, 4);
        tree.assert_invariants();
    }

    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.node_count(), 0);
    assert_eq!(tree.black_height(), 0);
}

/// Simulation 9: Interleaved Middle Deletions
/// Builds a large tree and deletes pieces from the middle, testing interior node deletion.
#[test]
fn test_simulation_delete_middle_pieces_single_nodes() {
    let mut tree = PieceTree::new("HEAD|");
    let mut expected_pieces: Vec<String> = vec!["HEAD|".to_string()];

    for i in 0..40 {
        let piece = format!("P{:02}|", i);
        let end = tree.get_text().len();
        tree.insert(end, &piece);
        expected_pieces.push(piece);
    }
    tree.assert_invariants();

    while expected_pieces.len() > 2 {
        let mid_idx = expected_pieces.len() / 2;
        let mut byte_offset = 0;
        for j in 0..mid_idx {
            byte_offset += expected_pieces[j].len();
        }
        let piece_len = expected_pieces[mid_idx].len();

        tree.delete(byte_offset, piece_len);
        expected_pieces.remove(mid_idx);

        tree.assert_invariants();
        let expected_text: String = expected_pieces.concat();
        assert_eq!(tree.get_text(), expected_text);
    }
}

/// Simulation 10: Multi-Piece Range Deletion
/// Deletes ranges spanning across multiple pieces.
#[test]
fn test_simulation_delete_multi_piece_spanning_ranges() {
    let mut tree = PieceTree::new("ROOT");
    for i in 0..30 {
        let end = tree.get_text().len();
        tree.insert(end, &format!("[BLOCK_{:02}]", i));
    }
    tree.assert_invariants();

    let total_len = tree.get_text().len();
    let delete_start = total_len / 4;
    let delete_len = total_len / 2;

    tree.delete(delete_start, delete_len);
    tree.assert_invariants();
    assert_eq!(tree.get_text().len(), total_len - delete_len);
}

/// Simulation 11: Complete Tree Drain and Re-growth (Lifecycle Simulation)
#[test]
fn test_simulation_complete_drain_and_refill() {
    let mut tree = PieceTree::new("InitialBaseText");
    for i in 0..40 {
        let end = tree.get_text().len();
        tree.insert(end, &format!("+{}", i));
    }
    tree.assert_invariants();

    let total_len = tree.get_text().len();
    tree.delete(0, total_len);
    tree.assert_invariants();
    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.node_count(), 0);
    assert_eq!(tree.black_height(), 0);

    for i in 0..40 {
        let end = tree.get_text().len();
        tree.insert(end, &format!("Refill_{} ", i));
        tree.assert_invariants();
    }
    assert!(tree.node_count() >= 40);
    assert!(tree.black_height() >= 2);
}

/// =========================================================================
/// 4. REAL-WORLD CODE EDITOR & WORKFLOW SIMULATIONS
/// =========================================================================

/// Simulation 12: Source Code Editing Simulation
#[test]
fn test_simulation_source_code_editing_session() {
    let mut tree = PieceTree::new("//! Rust Module\n\nfn main() {\n    println!(\"init\");\n}\n");
    tree.assert_invariants();

    tree.insert(0, "use std::collections::HashMap;\nuse std::sync::Arc;\n\n");
    tree.assert_invariants();

    let main_pos = tree.get_text().find("fn main").unwrap();
    tree.insert(main_pos, "fn calculate_sum(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n");
    tree.assert_invariants();

    let end_pos = tree.get_text().len();
    tree.insert(end_pos, "\nstruct State {\n    count: usize,\n}\n");
    tree.assert_invariants();

    let print_pos = tree.get_text().find("println!(\"init\");").unwrap();
    let print_len = "println!(\"init\");".len();
    tree.delete(print_pos, print_len);
    tree.assert_invariants();

    tree.insert(print_pos, "let sum = calculate_sum(10, 20);\n    println!(\"sum = {}\", sum);");
    tree.assert_invariants();

    let use_len = "use std::collections::HashMap;\nuse std::sync::Arc;\n\n".len();
    tree.delete(0, use_len);
    tree.assert_invariants();

    let code = tree.get_text();
    assert!(code.starts_with("//! Rust Module"));
    assert!(code.contains("fn calculate_sum"));
    assert!(code.contains("calculate_sum(10, 20)"));
    assert!(code.contains("struct State"));
}

/// Simulation 13: Markdown Document Editing Simulation
#[test]
fn test_simulation_markdown_document_editing() {
    let mut tree = PieceTree::new("# Project Title\n\n## Overview\nThis is a piece tree.\n");
    tree.assert_invariants();

    for i in 1..=15 {
        let end = tree.get_text().len();
        tree.insert(end, &format!("- Feature item {}\n", i));
        tree.assert_invariants();
    }

    let overview_end = tree.get_text().find("This is a piece tree.\n").unwrap() + "This is a piece tree.\n".len();
    tree.insert(overview_end, "\n> Note: High performance text buffer.\n\n");
    tree.assert_invariants();

    let feat_start = tree.get_text().find("- Feature item 1\n").unwrap();
    let mut feat_5_len = 0;
    for i in 1..=5 {
        feat_5_len += format!("- Feature item {}\n", i).len();
    }
    tree.delete(feat_start, feat_5_len);
    tree.assert_invariants();

    let doc = tree.get_text();
    assert!(!doc.contains("Feature item 1\n"));
    assert!(doc.contains("Feature item 6\n"));
    assert!(doc.contains("High performance text buffer"));
}

/// =========================================================================
/// 5. MATHEMATICAL RED-BLACK TREE HEIGHT & BLACK HEIGHT BOUNDS
/// =========================================================================

#[test]
fn test_rb_tree_height_logarithmic_bounds() {
    let mut tree = PieceTree::new("");

    for i in 0..150 {
        let chunk = format!("node_{:03} ", i);
        let len = tree.get_text().len();
        let pos = if len == 0 { 0 } else { (i * 37) % (len + 1) };

        tree.insert(pos, &chunk);
        tree.assert_invariants();

        let n = tree.node_count();
        let bh = tree.black_height();
        let h = tree.tree_height();

        assert!(bh <= h, "Black height {} cannot exceed tree height {}", bh, h);
        assert!(h <= 2 * bh + 1, "Height {} exceeded 2 * black_height + 1 ({})", h, 2 * bh + 1);

        let max_theoretical_height = (2.0 * ((n + 1) as f64).log2()).floor() as usize + 2;
        assert!(h <= max_theoretical_height, "Height {} exceeded max theoretical {} for n={}", h, max_theoretical_height, n);
    }
}

/// =========================================================================
/// 6. FUZZING WITH STRICT INVARIANT VALIDATION AT EVERY STEP
/// =========================================================================

#[test]
fn test_simulation_fuzz_random_insertions_with_strict_rb_invariants() {
    let mut tree = PieceTree::new("Start of document. ");
    let mut reference = String::from("Start of document. ");

    let mut state: u64 = 5432109876;
    let mut next_rand = || -> u64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state
    };

    let words = [
        "alpha ", "beta ", "gamma ", "delta ", "epsilon ",
        "zeta ", "eta ", "theta ", "iota ", "kappa ",
        "\n", "   ", "====", "fn test() ", "let x = 42; ",
    ];

    for _ in 0..200 {
        let snippet = words[(next_rand() as usize) % words.len()];
        let pos = if reference.is_empty() {
            0
        } else {
            (next_rand() as usize) % (reference.len() + 1)
        };

        tree.insert(pos, snippet);
        reference.insert_str(pos, snippet);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), reference);
    }
}
