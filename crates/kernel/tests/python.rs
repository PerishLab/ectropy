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

fn ladder(heads: &[&str]) -> String {
    let mut text = "def pick(a, b, c):\n".to_string();
    for (at, head) in heads.iter().enumerate() {
        text.push_str(&format!("    {head}:\n        return {at}\n"));
    }
    text
}

#[test]
fn dispatch() {
    let chain = ["if a == 1", "elif a == 2", "elif a == 3", "else"];
    assert_eq!(count(scan("vessel.py", &ladder(&chain)), "dispatch"), 2);
    let rust = "fn pick(a: u8) -> u8 { if a == 1 { 1 } else if a == 2 { 2 } else if a == 3 { 3 } else { 0 } }";
    assert_eq!(count(laws(rust), "dispatch"), 2);
    for heads in [
        ["if a == 1", "if a == 2"],
        ["if (a == 1)", "elif (a == 2)"],
        ["if a == 'x'", "elif a == \"y\""],
        ["if a == f\"x\"", "elif a == \"\"\"y\"\"\""],
        ["if a == True", "elif a == None"],
        ["if a.kind == 1", "elif a.kind == 2"],
        ["if self.kind == 1", "elif self.kind == 2"],
        ["if a.kind.tag == 1", "elif a.kind.tag == 2"],
        ["if a == 'x y'", "elif a == 'xy'"],
    ] {
        assert_eq!(count(scan("vessel.py", &ladder(&heads)), "dispatch"), 1);
    }
    let inline = "def pick(a):\n    if a == 1: return 1\n    elif a == 2: return 2\n";
    assert_eq!(count(scan("vessel.py", inline), "dispatch"), 1);
    let module = "if a == 1:\n    go()\nelif a == 2:\n    stop()\n";
    let found = findings("vessel.py", module, &config(&[], 4));
    let lines: Vec<usize> = found.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [3]);
    assert_eq!(found[0].note, "a");
    let method = format!("class Vessel:\n{}", indent(&ladder(&chain)));
    assert_eq!(count(scan("vessel.py", &method), "dispatch"), 2);
}

#[test]
fn probes() {
    for heads in [
        ["if a == 1", "elif b == 2"],
        ["if a == b", "elif a == c"],
        ["if a == 1 and b", "elif a == 2 and b"],
        ["if a.kind() == 1", "elif a.kind() == 2"],
        ["if a[0] == 1", "elif a[0] == 2"],
        ["if a == 1", "if a == 1"],
        ["if (a == 1)", "elif a == 1"],
        ["if a == 1 ", "elif a==1"],
        ["if a != 1", "elif a != 2"],
        ["if a is None", "elif a is True"],
        ["if a == -1", "elif a == -2"],
        ["if 1 == a", "elif 2 == a"],
        ["while a == 1", "while a == 2"],
    ] {
        assert_eq!(count(scan("vessel.py", &ladder(&heads)), "dispatch"), 0);
    }
    let split =
        "def pick(a):\n    if a == 1:\n        go()\n    probe()\n    if a == 2:\n        stop()\n";
    assert_eq!(count(scan("vessel.py", split), "dispatch"), 0);
    let nested = "def pick(a):\n    if a == 1:\n        go()\n    else:\n        if a == 2:\n            stop()\n";
    assert_eq!(count(scan("vessel.py", nested), "dispatch"), 0);
    let ternary = "def pick(a):\n    x = 1 if a == 1 else 2\n    y = 3 if a == 2 else 4\n";
    assert_eq!(count(scan("vessel.py", ternary), "dispatch"), 0);
}
