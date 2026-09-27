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
