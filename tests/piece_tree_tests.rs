use textio::PieceTree;

#[test]
fn test_new_empty() {
    let tree = PieceTree::new("");
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_new_with_content() {
    let tree = PieceTree::new("Hello, World!");
    assert_eq!(tree.get_text(), "Hello, World!");
}

#[test]
fn test_insert_empty_into_empty() {
    let mut tree = PieceTree::new("");
    tree.insert(0, "");
    assert_eq!(tree.get_text(), "");
}

#[test]
fn test_insert_empty_into_existing() {
    let mut tree = PieceTree::new("abc");
    tree.insert(1, "");
    assert_eq!(tree.get_text(), "abc");
}

#[test]
fn test_insert_into_empty_tree() {
    let mut tree = PieceTree::new("");
    tree.insert(0, "Hello");
    assert_eq!(tree.get_text(), "Hello");

    tree.insert(5, " World");
    assert_eq!(tree.get_text(), "Hello World");
}

#[test]
fn test_insert_at_start() {
    let mut tree = PieceTree::new("World");
    tree.insert(0, "Hello ");
    assert_eq!(tree.get_text(), "Hello World");

    tree.insert(0, "Say: ");
    assert_eq!(tree.get_text(), "Say: Hello World");
}

#[test]
fn test_insert_at_end() {
    let mut tree = PieceTree::new("Hello");
    tree.insert(5, " World");
    assert_eq!(tree.get_text(), "Hello World");

    tree.insert(11, "!");
    assert_eq!(tree.get_text(), "Hello World!");
}

#[test]
fn test_insert_past_end() {
    let mut tree = PieceTree::new("Hello");
    tree.insert(100, " World");
    assert_eq!(tree.get_text(), "Hello World");
}

#[test]
fn test_insert_in_middle_single_split() {
    let mut tree = PieceTree::new("HelloWorld");
    tree.insert(5, " ");
    assert_eq!(tree.get_text(), "Hello World");
}

#[test]
fn test_insert_in_middle_multiple_splits() {
    let mut tree = PieceTree::new("abcdef");
    tree.insert(1, "1"); // a1bcdef
    assert_eq!(tree.get_text(), "a1bcdef");

    tree.insert(3, "2"); // a1b2cdef
    assert_eq!(tree.get_text(), "a1b2cdef");

    tree.insert(5, "3"); // a1b2c3def
    assert_eq!(tree.get_text(), "a1b2c3def");
}

#[test]
fn test_repeated_middle_insertions() {
    let mut tree = PieceTree::new("()");
    for i in 0..50 {
        tree.insert(i + 1, "x");
    }
    assert_eq!(tree.get_text(), format!("({})", "x".repeat(50)));
}

#[test]
fn test_sequential_character_typing() {
    let mut tree = PieceTree::new("");
    let word = "The quick brown fox jumps over the lazy dog";
    for (i, ch) in word.chars().enumerate() {
        tree.insert(i, &ch.to_string());
    }
    assert_eq!(tree.get_text(), word);
}

#[test]
fn test_reverse_character_typing() {
    let mut tree = PieceTree::new("");
    let word = "antigravity";
    for ch in word.chars().rev() {
        tree.insert(0, &ch.to_string());
    }
    assert_eq!(tree.get_text(), word);
}

#[test]
fn test_interleaved_html_editing() {
    let mut tree = PieceTree::new("<div></div>");
    tree.insert(5, "<p></p>");
    assert_eq!(tree.get_text(), "<div><p></p></div>");

    tree.insert(8, "Hello");
    assert_eq!(tree.get_text(), "<div><p>Hello</p></div>");

    tree.insert(13, " World");
    assert_eq!(tree.get_text(), "<div><p>Hello World</p></div>");
}

#[test]
fn test_unicode_multibyte() {
    let mut tree = PieceTree::new("🦀 Rust 🚀");
    assert_eq!(tree.get_text(), "🦀 Rust 🚀");

    // "🦀 " is 4 + 1 = 5 bytes
    tree.insert(5, "is awesome ");
    assert_eq!(tree.get_text(), "🦀 is awesome Rust 🚀");

    // Chinese characters
    let mut cjk_tree = PieceTree::new("你好世界");
    cjk_tree.insert(6, "，美丽"); // insert between "好" (byte 6) and "世"
    assert_eq!(cjk_tree.get_text(), "你好，美丽世界");
}

#[test]
fn test_large_insertions_and_multiline() {
    let mut tree = PieceTree::new("");
    let paragraph = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\n";
    for _ in 0..50 {
        tree.insert(0, paragraph);
    }
    assert_eq!(tree.get_text(), paragraph.repeat(50));
}

#[test]
fn test_fuzz_random_insertions_against_reference() {
    let mut tree = PieceTree::new("Initial document text.");
    let mut reference = String::from("Initial document text.");

    // Deterministic pseudo-random number generator (LCG)
    let mut state: u64 = 123456789;
    let mut next_rand = || -> u64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        state
    };

    let snippets = [
        "a",
        "hello",
        " ",
        "\n",
        "piece-tree",
        "12345",
        "xyz",
        " lorem ipsum ",
        "!",
        "🚀",
        "🦀",
    ];

    for _ in 0..1000 {
        let snippet = snippets[(next_rand() as usize) % snippets.len()];

        // Collect all valid UTF-8 character boundary byte positions
        let mut char_boundaries: Vec<usize> = reference.char_indices().map(|(idx, _)| idx).collect();
        char_boundaries.push(reference.len());

        let pos = char_boundaries[(next_rand() as usize) % char_boundaries.len()];

        tree.insert(pos, snippet);
        reference.insert_str(pos, snippet);

        assert_eq!(tree.get_text(), reference);
    }
}
