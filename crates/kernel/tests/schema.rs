use grammar::Kind;
use kernel::config::Config;
use kernel::schema::Record;

fn project(path: &str, text: &str) -> Vec<Record> {
    let source = grammar::Source {
        path: path.to_string(),
        text: text.to_string(),
    };
    kernel::schema::project(&source, &kernel::structure(&source))
}

fn findings(sources: &[(&str, &str)], config: &Config) -> Vec<kernel::Finding> {
    let mut records = Vec::new();
    for (path, text) in sources {
        records.extend(project(path, text));
    }
    kernel::schema::check(&records, config)
}

fn config() -> Config {
    Config {
        file: Default::default(),
    }
}

#[test]
fn threshold() {
    let two = [
        ("a.rs", "struct A { one: u8, two: u8, three: u8, four: u8 }"),
        ("b.rs", "struct B { four: i8, three: i8, two: i8, one: i8 }"),
    ];
    assert!(findings(&two, &config()).is_empty());
    let three = [
        two[0],
        two[1],
        (
            "c.rs",
            "struct C { one: bool, two: bool, three: bool, four: bool }",
        ),
    ];
    let found = findings(&three, &config());
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, "a.rs");
    assert_eq!(found[0].line, 1);
    assert!(
        found[0]
            .note
            .contains("3 records repeat 4 fields over limit 2: A, B, C")
    );
}

#[test]
fn languages() {
    let sources = [
        (
            "a.rs",
            "struct Rust { alpha: u8, beta: u8, gamma: u8, delta: u8 }",
        ),
        (
            "b.ts",
            "interface Web { delta: string; gamma: string; beta: string; alpha: string }",
        ),
        (
            "c.py",
            "class Python:\n    alpha: str\n    beta: str\n    gamma: str\n    delta: str\n",
        ),
    ];
    assert_eq!(findings(&sources, &config()).len(), 1);
}

#[test]
fn width() {
    let sources = [
        ("a.rs", "struct A { one: u8, two: u8, three: u8 }"),
        ("b.rs", "struct B { one: u8, two: u8, three: u8 }"),
        ("c.rs", "struct C { one: u8, two: u8, three: u8 }"),
    ];
    assert!(findings(&sources, &config()).is_empty());
}

#[test]
fn exclusions() {
    let sources = [
        (
            "a.rs",
            "struct A(u8, u8, u8, u8); enum E { A { one: u8, two: u8, three: u8, four: u8 } }",
        ),
        (
            "b.ts",
            "const value = { one: 1, two: 2, three: 3, four: 4 }; type Alias = string",
        ),
        (
            "c.py",
            "class Plain:\n    one = 1\n    two = 2\n    three = 3\n    four = 4\n",
        ),
    ];
    let count: usize = sources
        .iter()
        .map(|(path, text)| project(path, text).len())
        .sum();
    assert_eq!(count, 0);
}

#[test]
fn shape() {
    let source = grammar::Source {
        path: "a.rs".to_string(),
        text: "struct A { one: u8, two: u8, three: u8, four: u8 }".to_string(),
    };
    let root = kernel::structure(&source);
    let record = root
        .kids
        .iter()
        .find(|node| node.kind == Kind::Record)
        .expect("record");
    assert_eq!(
        record
            .kids
            .iter()
            .filter(|kid| kid.kind == Kind::Field)
            .count(),
        4
    );
    assert!(record.kids.iter().any(|kid| kid.kind == Kind::Label));
}

#[test]
fn label() {
    let source = grammar::Source {
        path: "a.rs".to_string(),
        text: "struct WideName { one: u8, two: u8, three: u8, four: u8 }".to_string(),
    };
    let root = kernel::structure(&source);
    let config = config();
    let words = kernel::check(&source, &root, &config)
        .iter()
        .filter(|finding| finding.law == "word")
        .count();
    assert_eq!(words, 1);
}

#[test]
fn boundary() {
    let mut config = config();
    config.file.boundary.push(kernel::config::Boundary {
        paths: vec!["wire/**".to_string()],
        allow: vec!["schema".to_string()],
        note: "the foreign wire owns this record".to_string(),
    });
    let sources = [
        ("a.rs", "struct A { one: u8, two: u8, three: u8, four: u8 }"),
        ("b.rs", "struct B { one: u8, two: u8, three: u8, four: u8 }"),
        (
            "wire/c.rs",
            "struct C { one: u8, two: u8, three: u8, four: u8 }",
        ),
    ];
    assert!(findings(&sources, &config).is_empty());
}
