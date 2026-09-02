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
/// 1. MULTI-CURSOR SIMULATION (PARALLEL DISPERSED EDITS)
/// =========================================================================

#[test]
fn test_multi_cursor_batch_editing_simulation() {
    let mut initial_lines = Vec::new();
    for i in 0..100 {
        initial_lines.push(format!("    let var_{:03} = compute_value({});\n", i, i));
    }
    let initial_content = initial_lines.concat();
    let mut tree = PieceTree::new(&initial_content);
    let mut oracle = initial_content;

    // Simulate 20 multi-cursors across 20 lines
    // At each cursor, insert "pub " at the start of the line, and append " // modified" at the end
    let cursor_lines = [2, 7, 12, 18, 25, 33, 40, 48, 55, 62, 70, 75, 80, 85, 90, 92, 95, 97, 98, 99];

    for &line_num in cursor_lines.iter().rev() {
        // Compute line start and end in oracle
        let mut current_line = 0;
        let mut line_start = 0;
        for (idx, ch) in oracle.char_indices() {
            if current_line == line_num {
                line_start = idx;
                break;
            }
            if ch == '\n' {
                current_line += 1;
            }
        }

        // Insert at start of statement (after 4 spaces)
        let insert_pos = line_start + 4;
        let prefix = "/* cached */ ";
        tree.insert(insert_pos, prefix);
        oracle.insert_str(insert_pos, prefix);

        // Find end of line
        let line_end = oracle[insert_pos..].find('\n').unwrap() + insert_pos;
        let suffix = " /* verified */";
        tree.insert(line_end, suffix);
        oracle.insert_str(line_end, suffix);

        tree.assert_invariants();
        assert_eq!(tree.get_text(), oracle);
    }

    // Now delete the inserted comments using multi-cursor
    for &line_num in &cursor_lines {
        let comment_str = "/* cached */ ";
        if let Some(pos) = tree.get_text().find(comment_str) {
            tree.delete(pos, comment_str.len());
            let oracle_pos = oracle.find(comment_str).unwrap();
            oracle.replace_range(oracle_pos..oracle_pos + comment_str.len(), "");
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}

/// =========================================================================
/// 2. GLOBAL FIND-AND-REPLACE SIMULATION IN LARGE DOCUMENTS
/// =========================================================================

#[test]
fn test_find_and_replace_all_occurrences_large_document() {
    let mut doc_builder = String::new();
    for i in 0..200 {
        doc_builder.push_str(&format!(
            "fn handle_event_{i}(ctx: &mut Context, old_handler: &OldHandler) -> Result<(), Error> {{\n    old_handler.process_{i}(ctx);\n    Ok(())\n}}\n\n"
        ));
    }

    let mut tree = PieceTree::new(&doc_builder);
    let mut oracle = doc_builder;

    let target = "old_handler";
    let replacement = "modern_async_event_dispatcher_v2";

    // Replace all occurrences one by one from beginning to end
    while let Some(pos) = oracle.find(target) {
        tree.delete(pos, target.len());
        tree.insert(pos, replacement);

        oracle.replace_range(pos..pos + target.len(), replacement);

        // Quick check
        assert_eq!(tree.get_text().len(), oracle.len());
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
    assert!(!tree.get_text().contains("old_handler"));
    assert!(tree.get_text().contains("modern_async_event_dispatcher_v2"));

    // Now replace back with a shorter token
    let short_replacement = "disp";
    while let Some(pos) = oracle.find(replacement) {
        tree.delete(pos, replacement.len());
        tree.insert(pos, short_replacement);

        oracle.replace_range(pos..pos + replacement.len(), short_replacement);
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}

/// =========================================================================
/// 3. BULK BLOCK COMMENTING AND INDENTATION
/// =========================================================================

#[test]
fn test_bulk_block_commenting_and_uncommenting() {
    let mut code_lines = Vec::new();
    for i in 0..80 {
        code_lines.push(format!("    let result_{} = calculate({});\n", i, i * 3));
    }
    let full_code = code_lines.concat();

    let mut tree = PieceTree::new(&full_code);
    let mut oracle = full_code;

    // Comment out all lines by inserting "// " at each line's start
    let mut line_indices = Vec::new();
    let mut idx = 0;
    while idx < oracle.len() {
        line_indices.push(idx);
        if let Some(next_nl) = oracle[idx..].find('\n') {
            idx += next_nl + 1;
        } else {
            break;
        }
    }

    // Apply from bottom to top to preserve index offsets
    for &start in line_indices.iter().rev() {
        tree.insert(start, "// ");
        oracle.insert_str(start, "// ");
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);

    // Uncomment all lines by deleting "// " from each line's start
    for _ in 0..line_indices.len() {
        if let Some(pos) = tree.get_text().find("// ") {
            tree.delete(pos, 3);
            let oracle_pos = oracle.find("// ").unwrap();
            oracle.replace_range(oracle_pos..oracle_pos + 3, "");
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}

/// =========================================================================
/// 4. UNDO / REDO EDIT HISTORY SIMULATION
/// =========================================================================

enum EditOp {
    Insert { pos: usize, text: String },
    Delete { pos: usize, deleted_text: String },
}

#[test]
fn test_undo_redo_history_simulation() {
    let initial = "fn main() {\n    // start\n}\n";
    let mut tree = PieceTree::new(initial);
    let mut oracle = String::from(initial);

    let mut history: Vec<EditOp> = Vec::new();
    let mut rng = TestRng::new(9988776655);

    let code_snippets = [
        "let x = 1;\n    ",
        "let y = 2;\n    ",
        "println!(\"x={}\", x);\n    ",
        "if x > 0 { y += 1; }\n    ",
        "// comment note\n    ",
    ];

    // Perform 100 forward edits, tracking inverse operations for undo
    for _ in 0..100 {
        if oracle.len() < 10 || rng.next_range(0, 10) < 6 {
            // INSERT
            let snippet = code_snippets[rng.next_range(0, code_snippets.len())];
            let pos = rng.next_range(0, oracle.len());
            // Align pos to char boundary
            let safe_pos = oracle.char_indices().map(|(i, _)| i).find(|&p| p >= pos).unwrap_or(oracle.len());

            tree.insert(safe_pos, snippet);
            oracle.insert_str(safe_pos, snippet);
            history.push(EditOp::Insert { pos: safe_pos, text: snippet.to_string() });
        } else {
            // DELETE
            let boundaries: Vec<usize> = oracle.char_indices().map(|(i, _)| i).collect();
            let start_idx = rng.next_range(0, boundaries.len());
            let max_del = 10.min(boundaries.len() - start_idx);
            let end_idx = start_idx + rng.next_range(1, max_del + 1);

            let start_byte = boundaries[start_idx];
            let end_byte = if end_idx < boundaries.len() { boundaries[end_idx] } else { oracle.len() };
            let del_text = oracle[start_byte..end_byte].to_string();
            let del_len = end_byte - start_byte;

            tree.delete(start_byte, del_len);
            oracle.replace_range(start_byte..end_byte, "");
            history.push(EditOp::Delete { pos: start_byte, deleted_text: del_text });
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);

    // Now UNDO all 100 operations in reverse order
    for op in history.iter().rev() {
        match op {
            EditOp::Insert { pos, text } => {
                // To undo an insert, delete the inserted text
                tree.delete(*pos, text.len());
                oracle.replace_range(*pos..*pos + text.len(), "");
            }
            EditOp::Delete { pos, deleted_text } => {
                // To undo a delete, re-insert the deleted text
                tree.insert(*pos, deleted_text);
                oracle.insert_str(*pos, deleted_text);
            }
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
    assert_eq!(tree.get_text(), initial, "Undo stack should restore document to initial state exactly");
}

/// =========================================================================
/// 5. HEAVY MULTILINGUAL UTF-8 & EMOJI EDITING
/// =========================================================================

#[test]
fn test_heavy_multilingual_utf8_cjk_emoji_editing() {
    let base_text = "🌍 Initial Global Document: 英语, 中文, 日本語, العربية, Русский.\n";
    let mut tree = PieceTree::new(base_text);
    let mut oracle = String::from(base_text);

    let multilingual_snippets = [
        "🦀 [Rust Lang] ",
        "你好世界 (Hello World) ",
        "こんにちは (Konnichiwa) ",
        "مرحبا بالعالم ",
        "Привет мир ",
        "🎉🚀✨🔥 ",
        "Élève français ",
        "Schöne Grüße ",
        "\n",
    ];

    let mut rng = TestRng::new(0xABCDEF_012345);

    for step in 0..500 {
        let is_insert = oracle.len() < 20 || rng.next_range(0, 10) < 6;

        if is_insert {
            let snippet = multilingual_snippets[rng.next_range(0, multilingual_snippets.len())];
            let boundaries: Vec<usize> = oracle.char_indices().map(|(i, _)| i).collect();
            let pos = if boundaries.is_empty() { 0 } else { boundaries[rng.next_range(0, boundaries.len())] };

            tree.insert(pos, snippet);
            oracle.insert_str(pos, snippet);
        } else {
            let boundaries: Vec<usize> = oracle.char_indices().map(|(i, _)| i).collect();
            let start_idx = rng.next_range(0, boundaries.len());
            let max_span = 4.min(boundaries.len() - start_idx);
            let end_idx = start_idx + rng.next_range(1, max_span + 1);

            let start_byte = boundaries[start_idx];
            let end_byte = if end_idx < boundaries.len() { boundaries[end_idx] } else { oracle.len() };
            let del_len = end_byte - start_byte;

            tree.delete(start_byte, del_len);
            oracle.replace_range(start_byte..end_byte, "");
        }

        if step % 50 == 0 {
            tree.assert_invariants();
            assert_eq!(tree.get_text(), oracle);
        }
    }

    tree.assert_invariants();
    assert_eq!(tree.get_text(), oracle);
}
