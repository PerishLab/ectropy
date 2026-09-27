mod seat;
use grammar::Kind;
use seat::*;

#[test]
fn rust() {
    let wide = "fn read() -> (u8, u8, u8, u8) { let row = (1, 2, 3, 4); row }";
    let narrow = "fn read() -> (u8, u8, u8) { (1, 2, 3) }";
    assert_eq!(count(laws(wide), "tuple"), 2);
    assert_eq!(count(laws(narrow), "tuple"), 0);
}

#[test]
fn calls() {
    let source = "fn read() { send(1, 2, 3, 4); Name(1, 2, 3, 4); vec!(1, 2, 3, 4); }";
    assert_eq!(count(laws(source), "tuple"), 0);
}

#[test]
fn python() {
    let source = "def read():\n    row = 1, 2, 3, 4\n    return (1, 2, 3, 4)\n";
    assert_eq!(count(scan("t.py", source), "tuple"), 2);
    let lists = "def read():\n    row = [1, 2, 3, 4]\n    send(1, 2, 3, 4)\n";
    assert_eq!(count(scan("t.py", lists), "tuple"), 0);
    let lambda = "read = lambda a, b, c, d: a\n";
    assert_eq!(count(scan("t.py", lambda), "tuple"), 0);
}

#[test]
fn unpacking() {
    let source = "a, b, c, d = row\nfor a, b, c, d in rows:\n    send(a)\n";
    assert_eq!(count(scan("t.py", source), "tuple"), 2);
    assert_eq!(count(scan("t.py", "[a, b, c, d] = row\n"), "tuple"), 1);
}

#[test]
fn typescript() {
    let source =
        "type Row = [A, B, C, D]; const held: [A, B, C, D] = row; const [a, b, c, d] = row;";
    assert_eq!(count(scan("t.ts", source), "tuple"), 3);
    let arrays = "const row = [a, b, c, d]; const box = { row: [a, b, c, d] }; send(a, b, c, d);";
    assert_eq!(count(scan("t.ts", arrays), "tuple"), 0);
}

#[test]
fn shape() {
    let source = grammar::Source {
        path: "t.rs".to_string(),
        text: "const ROW: (u8, u8, u8, u8) = (1, 2, 3, 4);".to_string(),
    };
    let root = kernel::structure(&source);
    let tuples: Vec<_> = root
        .kids
        .iter()
        .filter(|node| node.kind == Kind::Tuple)
        .collect();
    assert_eq!(tuples.len(), 2);
    assert!(tuples.iter().all(|node| {
        node.kids.len() == 4 && node.kids.iter().all(|kid| kid.kind == Kind::Position)
    }));
}

#[test]
fn boundary() {
    let mut config = config(&[], 4);
    config.file.boundary.push(kernel::config::Boundary {
        paths: vec!["wire/**".to_string()],
        allow: vec!["tuple".to_string()],
        note: "the foreign wire is irreducibly positional".to_string(),
    });
    let source = "const ROW: (u8, u8, u8, u8) = (1, 2, 3, 4);";
    assert_eq!(
        count(
            findings("wire/row.rs", source, &config)
                .iter()
                .map(|finding| finding.law.clone())
                .collect(),
            "tuple"
        ),
        0
    );
}
