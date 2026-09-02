use textio::PieceTree;

/// Linear Congruential Generator for reproducible pseudo-random numbers in tests
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

    fn next_bool(&mut self, prob_percent: usize) -> bool {
        (self.next_u64() as usize % 100) < prob_percent
    }
}

/// Helper function to find a valid UTF-8 char boundary in a String
fn random_char_boundary(s: &str, rng: &mut TestRng) -> usize {
    if s.is_empty() {
        return 0;
    }
    let mut boundaries: Vec<usize> = s.char_indices().map(|(idx, _)| idx).collect();
    boundaries.push(s.len());
    boundaries[rng.next_range(0, boundaries.len())]
}

/// =========================================================================
/// 1. MASSIVE CHURN SIMULATION (THOUSANDS OF INTERLEAVED OPERATIONS)
/// =========================================================================

#[test]
fn test_massive_churn_5000_operations() {
    let initial_text = "The story begins with an empty canvas waiting for words.\n";
    let mut tree = PieceTree::new(initial_text);
    let mut oracle = String::from(initial_text);

    let mut rng = TestRng::new(42_1337_9999);

    let snippets = [
        "a", "b", "c", " ", "\n", "foo", "bar", "hello_world",
        "1234567890", "fn process_data(item: &Item) -> Result<(), Error> {\n    Ok(())\n}\n",
        "/* block comment */", "x = y + z;", "🦀", "🚀", "✨",
        "println!(\"debug: {}\", val);",
    ];

    for step in 0..5000 {
        // 60% insert, 40% delete (unless oracle is small, then bias towards insert)
        let is_insert = oracle.len() < 10 || rng.next_bool(60);

        if is_insert {
            let snippet = snippets[rng.next_range(0, snippets.len())];
            let pos = random_char_boundary(&oracle, &mut rng);

            tree.insert(pos, snippet);
            oracle.insert_str(pos, snippet);
        } else {
            // Delete a range
            let boundaries: Vec<usize> = oracle.char_indices().map(|(idx, _)| idx).collect();
            let start_idx = rng.next_range(0, boundaries.len());
            let end_idx = rng.next_range(start_idx, boundaries.len() + 1);

            let start_byte = boundaries[start_idx];
            let end_byte = if end_idx < boundaries.len() {
                boundaries[end_idx]
            } else {
                oracle.len()
            };

            let delete_len = end_byte - start_byte;
            tree.delete(start_byte, delete_len);
            oracle.replace_range(start_byte..end_byte, "");
        }

        // Fast checks every step
        assert_eq!(tree.get_text().len(), oracle.len(), "Length mismatch at step {}", step);

        // Full invariant check every 50 steps or on small trees
        if step % 50 == 0 || tree.node_count() < 10 {
            tree.assert_invariants();
            assert_eq!(tree.get_text(), oracle, "Content mismatch at step {}", step);
        }
    }

    // Final full verification
    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}

/// =========================================================================
/// 2. BIG TREE POPULATION AND HEIGHT BOUND VERIFICATION (1000+ NODES)
/// =========================================================================

#[test]
fn test_large_tree_growing_to_thousands_of_nodes() {
    let mut tree = PieceTree::new("ROOT_PIECE\n");
    let mut oracle = String::from("ROOT_PIECE\n");

    let mut rng = TestRng::new(0xDEADBEEF_CAFE);

    // Phase 1: Grow to >1,000 nodes using varied insertion patterns
    for i in 0..1200 {
        let snippet = format!("piece_{:05}_payload\n", i);
        let pos = match i % 4 {
            0 => 0, // Prepend
            1 => oracle.len(), // Append
            2 => oracle.len() / 2, // Middle
            _ => random_char_boundary(&oracle, &mut rng), // Random
        };

        // Align pos to char boundary if needed
        let safe_pos = if pos >= oracle.len() {
            oracle.len()
        } else {
            oracle.char_indices().map(|(idx, _)| idx).find(|&p| p >= pos).unwrap_or(oracle.len())
        };

        tree.insert(safe_pos, &snippet);
        oracle.insert_str(safe_pos, &snippet);

        if i % 100 == 0 {
            tree.assert_invariants();
            let n = tree.node_count();
            let h = tree.tree_height();
            let max_h = 2 * ((n + 1) as f64).log2().ceil() as usize + 2;
            assert!(h <= max_h, "Tree height {} exceeds max RB height {} for {} nodes", h, max_h, n);
        }
    }

    assert!(tree.node_count() >= 1000, "Expected at least 1000 nodes, got {}", tree.node_count());
    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);

    // Phase 2: Random deletions shrinking down to ~200 nodes
    for step in 0..400 {
        if oracle.is_empty() {
            break;
        }
        let boundaries: Vec<usize> = oracle.char_indices().map(|(idx, _)| idx).collect();
        let start_idx = rng.next_range(0, boundaries.len());
        let max_del_nodes = 5.min(boundaries.len() - start_idx);
        let end_idx = start_idx + rng.next_range(1, max_del_nodes + 1);

        let start_byte = boundaries[start_idx];
        let end_byte = if end_idx < boundaries.len() { boundaries[end_idx] } else { oracle.len() };
        let del_len = end_byte - start_byte;

        tree.delete(start_byte, del_len);
        oracle.replace_range(start_byte..end_byte, "");

        if step % 50 == 0 {
            tree.assert_invariants();
            assert_eq!(tree.get_text(), oracle);
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);

    // Phase 3: Re-growth by 500 insertions
    for i in 0..500 {
        let snippet = format!("regrowth_{} ", i);
        let pos = random_char_boundary(&oracle, &mut rng);
        tree.insert(pos, &snippet);
        oracle.insert_str(pos, &snippet);
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}

/// =========================================================================
/// 3. MASSIVE MULTI-PIECE DELETIONS SPANNING DOZENS/HUNDREDS OF PIECES
/// =========================================================================

#[test]
fn test_multi_piece_mass_deletions_across_large_trees() {
    let mut tree = PieceTree::new("");
    let mut oracle = String::new();

    const NUM_PIECES: usize = 300;
    // Build tree with 300 pieces
    for i in 0..NUM_PIECES {
        let chunk = format!("[P{:03}:data_block_{}]", i, i * 11);
        let pos = tree.get_text().len();
        tree.insert(pos, &chunk);
        oracle.push_str(&chunk);
    }

    tree.assert_invariants();
    assert_eq!(tree.node_count(), NUM_PIECES);
    assert_eq!(tree.get_text(), oracle);

    let mut rng = TestRng::new(0x1234_5678_9ABC);

    // Perform deletions spanning large swaths of the tree (e.g. 20% to 50% of document)
    for _ in 0..10 {
        if oracle.len() < 50 {
            break;
        }

        let doc_len = oracle.len();
        let span_len = rng.next_range(doc_len / 10, doc_len / 2);
        let start_pos = rng.next_range(0, doc_len - span_len);

        // Find char boundaries
        let start_byte = oracle.char_indices().map(|(i, _)| i).find(|&p| p >= start_pos).unwrap_or(0);
        let end_target = start_byte + span_len;
        let end_byte = oracle.char_indices().map(|(i, _)| i).find(|&p| p >= end_target).unwrap_or(oracle.len());

        let del_len = end_byte - start_byte;
        tree.delete(start_byte, del_len);
        oracle.replace_range(start_byte..end_byte, "");

        tree.assert_invariants();
        assert_eq!(tree.get_text(), oracle);
    }
}

/// =========================================================================
/// 4. DEEP FRACTAL MIDDLE SPLITS
/// =========================================================================

#[test]
fn test_deep_cascading_fractal_splits() {
    // Repeatedly inserting in the exact center splits pieces into smaller and smaller pieces
    let mut tree = PieceTree::new("<|>");
    let mut oracle = String::from("<|>");

    for i in 0..150 {
        let mid = tree.get_text().len() / 2;
        let insert_str = format!("-{:03}-", i);
        tree.insert(mid, &insert_str);
        oracle.insert_str(mid, &insert_str);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), oracle);
    }

    // Now delete from the center outwards
    for _ in 0..50 {
        let doc_len = tree.get_text().len();
        if doc_len <= 4 {
            break;
        }
        let mid = doc_len / 2;
        let del_len = 4.min(doc_len - mid);
        tree.delete(mid - 2, del_len);
        oracle.replace_range(mid - 2..mid - 2 + del_len, "");

        tree.assert_invariants();
        assert_eq!(tree.get_text(), oracle);
    }
}

/// =========================================================================
/// 5. EXTREMITY STRESS (RAPID HEAD / TAIL CHURN)
/// =========================================================================

#[test]
fn test_extremity_stress_head_and_tail() {
    let mut tree = PieceTree::new("MIDDLE");
    let mut oracle = String::from("MIDDLE");

    // Rapidly prepend and append 200 times
    for i in 0..200 {
        let head_str = format!("H{:03}_", i);
        tree.insert(0, &head_str);
        oracle.insert_str(0, &head_str);

        let tail_str = format!("_T{:03}", i);
        let end_pos = tree.get_text().len();
        tree.insert(end_pos, &tail_str);
        oracle.push_str(&tail_str);

        if i % 20 == 0 {
            tree.assert_invariants();
            assert_eq!(tree.get_text(), oracle);
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);

    // Rapidly delete from head and tail
    for _ in 0..100 {
        // Delete 5 bytes from head
        if tree.get_text().len() > 10 {
            tree.delete(0, 5);
            oracle.replace_range(0..5, "");
        }

        // Delete 5 bytes from tail
        let len = tree.get_text().len();
        if len > 10 {
            tree.delete(len - 5, 5);
            oracle.replace_range(len - 5..len, "");
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}
