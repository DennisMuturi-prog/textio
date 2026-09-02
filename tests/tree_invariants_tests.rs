use textio::PieceTree;

/// =========================================================================
/// 1. INITIALIZATION & BASIC INVARIANTS
/// =========================================================================

#[test]
fn test_init_empty_tree() {
    let tree = PieceTree::new("");
    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.get_text().len(), 0);
}

#[test]
fn test_init_with_single_character() {
    let tree = PieceTree::new("A");
    assert_eq!(tree.get_text(), "A");
    assert_eq!(tree.get_text().len(), 1);
}

#[test]
fn test_init_with_long_text() {
    let original = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(20);
    let tree = PieceTree::new(&original);
    assert_eq!(tree.get_text(), original);
    assert_eq!(tree.get_text().len(), original.len());
}

#[test]
fn test_init_with_unicode_multibyte() {
    let unicode_text = "🦀 Rust 🚀 - 你好世界 - 🌟✨🎉";
    let tree = PieceTree::new(unicode_text);
    assert_eq!(tree.get_text(), unicode_text);
    assert_eq!(tree.get_text().len(), unicode_text.len());
}

/// =========================================================================
/// 2. INSERTION FUNCTIONALITY & METADATA (SUBTREE LENGTHS & OFFSETS)
/// =========================================================================

#[test]
fn test_insert_empty_string_is_noop() {
    let mut tree = PieceTree::new("Initial Content");
    tree.insert(0, "");
    assert_eq!(tree.get_text(), "Initial Content");

    tree.insert(7, "");
    assert_eq!(tree.get_text(), "Initial Content");

    tree.insert(15, "");
    assert_eq!(tree.get_text(), "Initial Content");

    tree.insert(100, "");
    assert_eq!(tree.get_text(), "Initial Content");
}

#[test]
fn test_insert_into_empty_tree() {
    let mut tree = PieceTree::new("");
    tree.insert(0, "First");
    assert_eq!(tree.get_text(), "First");
    assert_eq!(tree.get_text().len(), 5);

    tree.insert(5, " Second");
    assert_eq!(tree.get_text(), "First Second");
    assert_eq!(tree.get_text().len(), 12);
}

#[test]
fn test_insert_at_start_repeatedly() {
    // Tests left-heavy growth and subtree re-indexing on the left spine
    let mut tree = PieceTree::new("Z");
    let mut expected = String::from("Z");

    for ch in ('A'..='Y').rev() {
        let s = ch.to_string();
        tree.insert(0, &s);
        expected.insert_str(0, &s);
        assert_eq!(tree.get_text(), expected);
        assert_eq!(tree.get_text().len(), expected.len());
    }
}

#[test]
fn test_insert_at_end_repeatedly() {
    // Tests right-heavy growth and subtree re-indexing on the right spine
    let mut tree = PieceTree::new("A");
    let mut expected = String::from("A");

    for ch in 'B'..='Z' {
        let s = ch.to_string();
        let len = tree.get_text().len();
        tree.insert(len, &s);
        expected.push_str(&s);
        assert_eq!(tree.get_text(), expected);
        assert_eq!(tree.get_text().len(), expected.len());
    }
}

#[test]
fn test_insert_past_end_appends_to_tree() {
    let mut tree = PieceTree::new("Hello");
    tree.insert(100, " World");
    assert_eq!(tree.get_text(), "Hello World");

    tree.insert(999, "!");
    assert_eq!(tree.get_text(), "Hello World!");
}

#[test]
fn test_insert_middle_single_split() {
    // "AC" -> insert "B" at index 1 -> "ABC"
    let mut tree = PieceTree::new("AC");
    tree.insert(1, "B");
    assert_eq!(tree.get_text(), "ABC");
    assert_eq!(tree.get_text().len(), 3);
}

#[test]
fn test_insert_middle_nested_splits() {
    // Tests that offsets and left_subtree_lengths remain consistent across splits
    let mut tree = PieceTree::new("()");
    for i in 0..40 {
        let content = format!("{}", i % 10);
        let middle = tree.get_text().len() / 2;
        tree.insert(middle, &content);
    }
    let text = tree.get_text();
    assert!(text.starts_with('('));
    assert!(text.ends_with(')'));
    assert_eq!(text.len(), 42);
}

#[test]
fn test_insert_sequential_typing() {
    let mut tree = PieceTree::new("");
    let sample = "The quick brown fox jumps over the lazy dog.";
    for (i, c) in sample.char_indices() {
        tree.insert(i, &c.to_string());
        assert_eq!(tree.get_text(), sample[..=i]);
    }
    assert_eq!(tree.get_text(), sample);
}

#[test]
fn test_insert_zigzag_rebalancing() {
    // Alternates inserts at beginning and end, forcing tree rotations and recoloring
    let mut tree = PieceTree::new("middle");
    let mut expected = String::from("middle");

    for i in 0..30 {
        let chunk_left = format!("[L{}]", i);
        let chunk_right = format!("[R{}]", i);

        tree.insert(0, &chunk_left);
        expected.insert_str(0, &chunk_left);
        assert_eq!(tree.get_text(), expected);

        let end_pos = tree.get_text().len();
        tree.insert(end_pos, &chunk_right);
        expected.push_str(&chunk_right);
        assert_eq!(tree.get_text(), expected);
    }
}

/// =========================================================================
/// 3. DELETION FUNCTIONALITY & METADATA (SUBTREE LENGTHS & OFFSETS)
/// =========================================================================

#[test]
fn test_delete_zero_length_is_noop() {
    let mut tree = PieceTree::new("Hello, World!");
    tree.delete(0, 0);
    assert_eq!(tree.get_text(), "Hello, World!");

    tree.delete(5, 0);
    assert_eq!(tree.get_text(), "Hello, World!");

    tree.delete(13, 0);
    assert_eq!(tree.get_text(), "Hello, World!");

    tree.delete(50, 0);
    assert_eq!(tree.get_text(), "Hello, World!");
}

#[test]
fn test_delete_from_empty_tree_is_noop() {
    let mut tree = PieceTree::new("");
    tree.delete(0, 5);
    assert_eq!(tree.get_text(), "");

    tree.delete(10, 10);
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_delete_past_end_of_document_is_noop() {
    let mut tree = PieceTree::new("Hello");
    tree.delete(10, 5);
    assert_eq!(tree.get_text(), "Hello");

    tree.delete(5, 5);
    assert_eq!(tree.get_text(), "Hello");
}

#[test]
fn test_delete_entire_document_single_piece() {
    let mut tree = PieceTree::new("FullDocument");
    tree.delete(0, "FullDocument".len());
    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.get_text().len(), 0);
}

#[test]
fn test_delete_entire_document_multiple_pieces() {
    let mut tree = PieceTree::new("Part1");
    tree.insert(5, " Part2");
    tree.insert(11, " Part3");
    assert_eq!(tree.get_text(), "Part1 Part2 Part3");

    let total_len = tree.get_text().len();
    tree.delete(0, total_len);
    assert_eq!(tree.get_text(), "");
    assert_eq!(tree.get_text().len(), 0);
}

#[test]
fn test_delete_prefix_of_single_piece() {
    let mut tree = PieceTree::new("0123456789");
    tree.delete(0, 4); // Remove "0123"
    assert_eq!(tree.get_text(), "456789");
    assert_eq!(tree.get_text().len(), 6);
}

#[test]
fn test_delete_suffix_of_single_piece() {
    let mut tree = PieceTree::new("0123456789");
    tree.delete(6, 4); // Remove "6789"
    assert_eq!(tree.get_text(), "012345");
    assert_eq!(tree.get_text().len(), 6);
}

#[test]
fn test_delete_middle_of_single_piece() {
    let mut tree = PieceTree::new("Hello Cruel World");
    tree.delete(5, 6); // Remove " Cruel"
    assert_eq!(tree.get_text(), "Hello World");
    assert_eq!(tree.get_text().len(), 11);
}

#[test]
fn test_delete_exact_whole_inserted_piece() {
    let mut tree = PieceTree::new("Hello World");
    tree.insert(5, " Beautiful");
    assert_eq!(tree.get_text(), "Hello Beautiful World");

    tree.delete(5, 10); // Remove " Beautiful"
    assert_eq!(tree.get_text(), "Hello World");
    assert_eq!(tree.get_text().len(), 11);
}

#[test]
fn test_delete_spanning_across_two_pieces() {
    let mut tree = PieceTree::new("Hello World");
    tree.insert(5, " Beautiful");
    assert_eq!(tree.get_text(), "Hello Beautiful World");

    // "Hello Beautiful World"
    // Delete from index 3 ("lo Beau") length 7 -> "Hel" + "tiful World"
    tree.delete(3, 7);
    assert_eq!(tree.get_text(), "Heltiful World");
}

#[test]
fn test_delete_spanning_across_multiple_intermediate_pieces() {
    let mut tree = PieceTree::new("AAA");
    tree.insert(3, "BBB");
    tree.insert(6, "CCC");
    tree.insert(9, "DDD");
    tree.insert(12, "EEE");
    assert_eq!(tree.get_text(), "AAABBBCCCDDDEEE");

    // Delete from index 2 (last 'A', all 'BBB', all 'CCC', first two 'D's) length 9
    // Remaining text: "AA" + "D" + "EEE" = "AADEEE"
    tree.delete(2, 9);
    assert_eq!(tree.get_text(), "AADEEE");
    assert_eq!(tree.get_text().len(), 6);
}

#[test]
fn test_delete_sequential_backspaces() {
    let mut tree = PieceTree::new("1234567890");
    let mut expected = String::from("1234567890");

    while !expected.is_empty() {
        let last_idx = expected.len() - 1;
        tree.delete(last_idx, 1);
        expected.pop();
        assert_eq!(tree.get_text(), expected);
        assert_eq!(tree.get_text().len(), expected.len());
    }
}

#[test]
fn test_delete_sequential_from_start() {
    let mut tree = PieceTree::new("abcdefghij");
    let mut expected = String::from("abcdefghij");

    while !expected.is_empty() {
        tree.delete(0, 1);
        expected.remove(0);
        assert_eq!(tree.get_text(), expected);
        assert_eq!(tree.get_text().len(), expected.len());
    }
}

#[test]
fn test_delete_with_length_greater_than_remaining_text() {
    let mut tree = PieceTree::new("Short Text");
    // Start at 6 ("Text"), request delete length 100
    tree.delete(6, 100);
    assert_eq!(tree.get_text(), "Short ");
    assert_eq!(tree.get_text().len(), 6);
}

/// =========================================================================
/// 4. INTERLEAVED OPERATIONS & DOCUMENT STATE INTEGRITY
/// =========================================================================

#[test]
fn test_interleaved_insertions_and_deletions() {
    let mut tree = PieceTree::new("let mut a = 1;");

    // Replace "1" with "100"
    tree.delete(12, 1);
    tree.insert(12, "100");
    assert_eq!(tree.get_text(), "let mut a = 100;");

    // Rename "a" to "counter"
    tree.delete(8, 1);
    tree.insert(8, "counter");
    assert_eq!(tree.get_text(), "let mut counter = 100;");

    // Prepend documentation
    tree.insert(0, "// Counter variable\n");
    assert_eq!(tree.get_text(), "// Counter variable\nlet mut counter = 100;");

    // Append increment
    let len = tree.get_text().len();
    tree.insert(len, "\ncounter += 1;");
    assert_eq!(tree.get_text(), "// Counter variable\nlet mut counter = 100;\ncounter += 1;");

    // Delete first line
    tree.delete(0, 20);
    assert_eq!(tree.get_text(), "let mut counter = 100;\ncounter += 1;");
}

#[test]
fn test_html_tag_editing_simulation() {
    let mut tree = PieceTree::new("<html><head></head><body></body></html>");

    // Insert title inside head (head opens at 6, closes at 12)
    tree.insert(12, "<title>My App</title>");
    assert_eq!(tree.get_text(), "<html><head><title>My App</title></head><body></body></html>");

    // Insert h1 inside body
    let body_pos = tree.get_text().find("<body>").unwrap() + "<body>".len();
    tree.insert(body_pos, "<h1>Hello</h1>");
    assert_eq!(tree.get_text(), "<html><head><title>My App</title></head><body><h1>Hello</h1></body></html>");

    // Delete title contents
    let title_start = tree.get_text().find("<title>").unwrap() + "<title>".len();
    tree.delete(title_start, "My App".len());
    assert_eq!(tree.get_text(), "<html><head><title></title></head><body><h1>Hello</h1></body></html>");

    // Insert new title
    tree.insert(title_start, "Piece Tree Test");
    assert_eq!(tree.get_text(), "<html><head><title>Piece Tree Test</title></head><body><h1>Hello</h1></body></html>");
}

#[test]
fn test_unicode_multibyte_interleaved_edits() {
    let mut tree = PieceTree::new("🦀 Rust 🚀 is fast!");

    // Delete " Rust " (bytes 4..10, length 6)
    tree.delete(4, 6);
    assert_eq!(tree.get_text(), "🦀🚀 is fast!");

    // Insert " Go " at byte 4
    tree.insert(4, " Go ");
    assert_eq!(tree.get_text(), "🦀 Go 🚀 is fast!");

    // Delete "🦀 Go 🚀" (bytes 0..12, 4 + 4 + 4 = 12 bytes)
    let prefix_len = "🦀 Go 🚀".len();
    tree.delete(0, prefix_len);
    assert_eq!(tree.get_text(), " is fast!");

    // Prepend Chinese greeting
    tree.insert(0, "你好，世界");
    assert_eq!(tree.get_text(), "你好，世界 is fast!");
}

#[test]
fn test_multiline_code_editing() {
    let mut tree = PieceTree::new("fn add(a: i32, b: i32) -> i32 {\n    a + b\n}");

    // Insert type annotations / documentation
    tree.insert(0, "/// Computes sum\n");
    assert_eq!(tree.get_text(), "/// Computes sum\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}");

    // Append another function
    let len = tree.get_text().len();
    tree.insert(len, "\n\nfn sub(a: i32, b: i32) -> i32 {\n    a - b\n}");
    assert_eq!(tree.get_text(), "/// Computes sum\nfn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\nfn sub(a: i32, b: i32) -> i32 {\n    a - b\n}");
}

#[test]
fn test_crlf_and_whitespace_preservation() {
    let mut tree = PieceTree::new("line1\r\nline2\r\n");
    tree.insert(7, "line1.5\r\n");
    assert_eq!(tree.get_text(), "line1\r\nline1.5\r\nline2\r\n");

    tree.delete(7, "line1.5\r\n".len());
    assert_eq!(tree.get_text(), "line1\r\nline2\r\n");
}
