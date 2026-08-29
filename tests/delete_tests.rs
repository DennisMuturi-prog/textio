use textio::PieceTree;

#[test]
fn test_delete_zero_length() {
    let mut tree = PieceTree::new("Hello, World!");
    tree.delete(0, 0);
    assert_eq!(tree.get_text(), "Hello, World!");

    tree.delete(5, 0);
    assert_eq!(tree.get_text(), "Hello, World!");

    tree.delete(13, 0);
    assert_eq!(tree.get_text(), "Hello, World!");
}

#[test]
fn test_delete_from_empty_tree() {
    let mut tree = PieceTree::new("");
    tree.delete(0, 5);
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_delete_entire_document_single_piece() {
    let mut tree = PieceTree::new("Hello World");
    tree.delete(0, 11);
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_delete_entire_document_multiple_pieces() {
    let mut tree = PieceTree::new("Hello");
    tree.insert(5, " Beautiful");
    tree.insert(15, " World");
    assert_eq!(tree.get_text(), "Hello Beautiful World");

    let total_len = "Hello Beautiful World".len();
    tree.delete(0, total_len);
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_delete_from_start() {
    let mut tree = PieceTree::new("Hello, World!");
    tree.delete(0, 7);
    assert_eq!(tree.get_text(), "World!");

    tree.delete(0, 5);
    assert_eq!(tree.get_text(), "!");
}

#[test]
fn test_delete_from_end() {
    let mut tree = PieceTree::new("Hello, World!");
    tree.delete(5, 8);
    assert_eq!(tree.get_text(), "Hello");

    tree.delete(4, 1);
    assert_eq!(tree.get_text(), "Hell");
}

#[test]
fn test_delete_in_middle_single_piece() {
    let mut tree = PieceTree::new("Hello Cruel World");
    // Delete " Cruel" (length 6 at index 5)
    tree.delete(5, 6);
    assert_eq!(tree.get_text(), "Hello World");

    let mut tree2 = PieceTree::new("abcdef");
    tree2.delete(2, 2); // Delete "cd"
    assert_eq!(tree2.get_text(), "abef");
}

#[test]
fn test_delete_entire_inserted_piece() {
    let mut tree = PieceTree::new("Hello World");
    tree.insert(5, " Beautiful");
    assert_eq!(tree.get_text(), "Hello Beautiful World");

    // Delete exact inserted piece " Beautiful" (index 5, length 10)
    tree.delete(5, 10);
    assert_eq!(tree.get_text(), "Hello World");
}

#[test]
fn test_delete_spanning_across_two_pieces() {
    let mut tree = PieceTree::new("Hello World");
    tree.insert(5, " Beautiful");
    assert_eq!(tree.get_text(), "Hello Beautiful World");

    // "Hello Beautiful World"
    // Delete from index 3 ("lo Beau") length 7: "Hel" + "tiful World"
    tree.delete(3, 7);
    assert_eq!(tree.get_text(), "Heltiful World");
}

#[test]
fn test_delete_spanning_multiple_pieces() {
    let mut tree = PieceTree::new("111");
    tree.insert(3, "222");
    tree.insert(6, "333");
    tree.insert(9, "444");
    assert_eq!(tree.get_text(), "111222333444");

    // Delete spanning middle of "111" through "222", "333", to middle of "444"
    // "11[12223334]44" -> start 2, length 8
    tree.delete(2, 8);
    assert_eq!(tree.get_text(), "1144");
}

#[test]
fn test_delete_backspace_simulation() {
    let mut tree = PieceTree::new("abcde");
    // Repeatedly delete the last character
    tree.delete(4, 1);
    assert_eq!(tree.get_text(), "abcd");
    tree.delete(3, 1);
    assert_eq!(tree.get_text(), "abc");
    tree.delete(2, 1);
    assert_eq!(tree.get_text(), "ab");
    tree.delete(1, 1);
    assert_eq!(tree.get_text(), "a");
    tree.delete(0, 1);
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_delete_and_insert_interleaved() {
    let mut tree = PieceTree::new("const x = 10;");

    // Replace "10" with "20"
    tree.delete(10, 2);
    assert_eq!(tree.get_text(), "const x = ;");
    tree.insert(10, "20");
    assert_eq!(tree.get_text(), "const x = 20;");

    // Replace "x" with "counter"
    tree.delete(6, 1);
    assert_eq!(tree.get_text(), "const  = 20;");
    tree.insert(6, "counter");
    assert_eq!(tree.get_text(), "const counter = 20;");
}

#[test]
fn test_delete_unicode_multibyte() {
    let mut tree = PieceTree::new("🦀 Rust 🚀 is fast");

    // Delete " Rust " (bytes 4..10, length 6)
    tree.delete(4, 6);
    assert_eq!(tree.get_text(), "🦀🚀 is fast");

    // Delete leading emoji "🦀" (4 bytes)
    tree.delete(0, 4);
    assert_eq!(tree.get_text(), "🚀 is fast");

    // Chinese characters: "你好，美丽世界"
    let mut cjk_tree = PieceTree::new("你好，美丽世界");
    // "美丽" starts at byte 9 (你好， is 3+3+3=9 bytes) and is 6 bytes
    cjk_tree.delete(9, 6);
    assert_eq!(cjk_tree.get_text(), "你好，世界");
}

#[test]
fn test_fuzz_random_inserts_and_deletes_against_reference() {
    let mut tree = PieceTree::new("Initial document text for fuzzing.");
    let mut reference = String::from("Initial document text for fuzzing.");

    // Deterministic pseudo-random number generator (LCG)
    let mut state: u64 = 987654321;
    let mut next_rand = || -> u64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        state
    };

    let snippets = [
        "a",
        "word",
        " ",
        "\n",
        "foo bar",
        "123",
        "🦀",
        "🚀",
    ];

    for _ in 0..500 {
        let op_type = next_rand() % 3; // 0, 1: insert, 2: delete

        if op_type < 2 || reference.is_empty() {
            // INSERT
            let snippet = snippets[(next_rand() as usize) % snippets.len()];
            let mut boundaries: Vec<usize> = reference.char_indices().map(|(i, _)| i).collect();
            boundaries.push(reference.len());
            let pos = boundaries[(next_rand() as usize) % boundaries.len()];

            tree.insert(pos, snippet);
            reference.insert_str(pos, snippet);
        } else {
            // DELETE
            let boundaries: Vec<usize> = reference.char_indices().map(|(i, _)| i).collect();
            let start_idx = (next_rand() as usize) % boundaries.len();
            let end_idx = start_idx + (next_rand() as usize) % (boundaries.len() - start_idx + 1);

            let start_byte = boundaries[start_idx];
            let end_byte = if end_idx < boundaries.len() {
                boundaries[end_idx]
            } else {
                reference.len()
            };

            let delete_len = end_byte - start_byte;
            tree.delete(start_byte, delete_len);
            reference.replace_range(start_byte..end_byte, "");
        }

        assert_eq!(tree.get_text(), reference);
    }
}
