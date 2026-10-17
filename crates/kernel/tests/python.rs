mod seat;
use grammar::Kind;
use seat::*;

fn holds(node: &kernel::Node, kind: Kind) -> bool {
    node.kind == kind || node.kids.iter().any(|kid| holds(kid, kind))
}

#[test]
fn coverage() {
    let source = r#"@sealed
class Vessel(Base):
    """A vessel."""

    async def sail(self, route: Path, *, dry=False):
        if route and not dry:
            with route.open() as stream:
                return {"stream": stream}
        elif dry:
            return None
        else:
            raise ValueError("route")

def choose(values):
    try:
        for value in values:
            if value:
                break
    except ValueError as fault:
        raise fault
    finally:
        pass
"#;
    let found = scan("vessel.py", source);
    assert!(!found.contains(&"coverage".to_string()));
}

#[test]
fn depth() {
    let source = "def sail():\n    if a:\n        for b in c:\n            while d:\n                if e:\n                    pass\n";
    assert!(scan("vessel.py", source).contains(&"block".to_string()));
}

#[test]
fn literal() {
    let source = "def sail():\n    if a:\n        for b in c:\n            value = {\"one\": {\"two\": 2}}\n";
    assert!(!scan("vessel.py", source).contains(&"block".to_string()));
}

#[test]
fn names() {
    let source = "def read_file(source):\n    bad_name = source\n";
    let found = scan("vessel.py", source);
    assert_eq!(count(found, "word"), 2);
}

#[test]
fn parameters() {
    let source = "def sail(one, two, three, four, five):\n    pass\n";
    assert_eq!(count(scan("vessel.py", source), "param"), 1);
}

#[test]
fn decisions() {
    let source = "def sail():\n    if a and b and c and d:\n        pass\n";
    assert_eq!(count(scan("vessel.py", source), "combination"), 1);
}

#[test]
fn comments() {
    let source =
        "\"\"\"Module.\n\"\"\"\n# note\ndef sail():\n    \"\"\"Sail.\n    \"\"\"\n    pass\n";
    assert_eq!(count(scan("vessel.py", source), "comment"), 3);
}

#[test]
fn environment() {
    let source = "import os\nhome = os.environ.get(\"HOME\")\n";
    let bare = config(&[], 4);
    assert_eq!(hits("vessel.py", source, &bare), 1);
    let mut granted = config(&[], 4);
    granted.file.grant.push(kernel::config::Grant {
        syntax: "environment".to_string(),
        paths: vec!["*.py".to_string()],
    });
    assert_eq!(hits("vessel.py", source, &granted), 0);
}

#[test]
fn identity() {
    let source = grammar::Source {
        path: "vessel.py".to_string(),
        text: "def test_sails(self):\n    assert True\n".to_string(),
    };
    let tree = kernel::structure(&source);
    assert!(!holds(&tree, Kind::Test));
}

#[test]
fn malformed() {
    let glyph = scan("vessel.py", "value = $bad\n");
    assert!(glyph.contains(&"coverage".to_string()));
    let margin = scan("vessel.py", "if ready:\n    go()\n  stop()\n");
    assert!(margin.contains(&"coverage".to_string()));
}

#[test]
fn stub() {
    let source = grammar::Source {
        path: "vessel.pyi".to_string(),
        text: "def sail() -> None: ...\n".to_string(),
    };
    assert!(grammar::parse(&source).kids.is_empty());
}

fn blocks(path: &str, text: &str) -> Vec<usize> {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    findings(path, text, &config)
        .into_iter()
        .filter(|finding| finding.law == "block")
        .map(|finding| finding.line)
        .collect()
}

fn indent(text: &str) -> String {
    text.lines().map(|line| format!("    {line}\n")).collect()
}

const SAIL: &str = "def sail(self, items):\n    for item in items:\n        if item:\n            while item:\n                return item\n";

const DEEP: &str = "def sail(self, items):\n    for item in items:\n        if item:\n            while item:\n                if item:\n                    return item\n";

#[test]
fn class() {
    let method = format!("class Vessel:\n{}", indent(SAIL));
    assert_eq!(blocks("vessel.py", &method), Vec::<usize>::new());
    assert_eq!(blocks("vessel.py", DEEP), [5]);
    let deep = format!("class Vessel:\n{}", indent(DEEP));
    assert_eq!(blocks("vessel.py", &deep), [6]);
    let marked = format!(
        "@sealed\nclass Vessel(Base, metaclass=Meta):\n{}",
        indent(DEEP)
    );
    assert_eq!(blocks("vessel.py", &marked), [7]);
    let inner = format!(
        "class Fleet:\n{}",
        indent(&format!("class Vessel:\n{}", indent(DEEP)))
    );
    assert_eq!(blocks("vessel.py", &inner), [7]);
    let local = format!(
        "def build():\n{}",
        indent(&format!("class Vessel:\n{}", indent(SAIL)))
    );
    assert_eq!(blocks("vessel.py", &local), [6]);
    let empty = "class Empty: pass\n\n\nclass Marker:\n    pass\n";
    assert!(scan("vessel.py", empty).is_empty());
}

#[test]
fn parity() {
    let shallow = "struct Vessel;\n\nimpl Vessel {\n    fn sail(&self, items: &[u8]) -> u8 {\n        for item in items {\n            if *item > 0 {\n                while *item > 0 {\n                    return *item;\n                }\n            }\n        }\n        0\n    }\n}\n";
    let deep = "struct Vessel;\n\nimpl Vessel {\n    fn sail(&self, items: &[u8]) -> u8 {\n        for item in items {\n            if *item > 0 {\n                while *item > 0 {\n                    if *item > 1 {\n                        return *item;\n                    }\n                }\n            }\n        }\n        0\n    }\n}\n";
    let method = format!("class Vessel:\n{}", indent(SAIL));
    let wide = format!("class Vessel:\n{}", indent(DEEP));
    assert_eq!(
        blocks("vessel.rs", shallow).len(),
        blocks("vessel.py", &method).len()
    );
    assert_eq!(
        blocks("vessel.rs", deep).len(),
        blocks("vessel.py", &wide).len()
    );
    assert_eq!(blocks("vessel.rs", deep).len(), 1);
}
