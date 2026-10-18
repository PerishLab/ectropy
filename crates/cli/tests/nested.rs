type Marks = Vec<(String, String)>;

const ITEMS: &str = "struct InnerThing;
impl InnerThing {
    fn inner_method(&self, first: u8, second: u8, third: u8, fourth: u8) -> u8 { first }
}
impl Clone for InnerThing {
    fn clone(&self) -> Self { InnerThing }
}
enum InnerKind { Alpha }
union InnerBits { whole: u32 }
trait InnerTrait { fn trait_method(&self); }
mod inner_mod { pub fn mod_helper() {} }
type InnerAlias = u8;
const INNER_LIMIT: u8 = 1;
static INNER_SEED: u8 = 2;
fn nested_helper(view: &str, first: u8, second: u8, third: u8, fourth: u8) -> u8 { first }
macro_rules! inner_macro { () => { fn spelled_helper() {} }; }";

const DECLS: &str =
    "fn foreign_probe(first: u8, second: u8, third: u8, fourth: u8, fifth: u8) -> i32;
    static FOREIGN_SEED: u8;";

fn foreign() -> String {
    format!("unsafe extern \"C\" {{\n    {DECLS}\n}}")
}

fn source(text: &str) -> grammar::Source {
    grammar::Source {
        path: "lib.rs".to_string(),
        text: text.to_string(),
    }
}

fn marks(text: &str) -> Marks {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    let source = source(text);
    let found = kernel::check(&source, &kernel::structure(&source), &config).into_iter();
    let mut found: Marks = found.map(|hit| (hit.law, hit.note)).collect();
    found.sort();
    found
}

fn host(body: &str) -> String {
    format!("pub fn host(flag: bool) -> u8 {{\n{body}\n1\n}}\n")
}

fn wrap(depth: usize, body: &str) -> String {
    let open = "if flag { ".repeat(depth);
    let close = " }".repeat(depth);
    format!("{open}{body}{close}")
}

fn blocks(found: Marks) -> usize {
    found.iter().filter(|mark| mark.0 == "block").count()
}

fn kinds(text: &str, kind: grammar::Kind) -> usize {
    fn walk(node: &kernel::Node, kind: grammar::Kind) -> usize {
        let own = usize::from(node.kind == kind);
        own + node.kids.iter().map(|kid| walk(kid, kind)).sum::<usize>()
    }
    walk(&kernel::structure(&source(text)), kind)
}

#[test]
fn parity() {
    for (text, flat) in [(ITEMS.to_string(), ITEMS), (foreign(), DECLS)] {
        let lifted = marks(&format!("{}\n{}", flat, host("")));
        assert!(lifted.iter().any(|mark| mark.0 == "param"), "{text}");
        assert_eq!(marks(&host(&text)), lifted, "{text}");
        assert_eq!(marks(&host(&wrap(1, &text))), lifted, "{text}");
        assert_eq!(marks(&host(&format!("let run = || {{ {text} }};"))), lifted);
    }
}

#[test]
fn depth() {
    let deep = "if flag { if flag { go(); } }";
    for level in 0..=2 {
        let lifted = format!(
            "fn nested() -> u8 {{ {} }}\n{}",
            wrap(level + 1, deep),
            host("")
        );
        let methods = format!(
            "struct Unit;\nimpl Unit {{ fn nested() {{ {} }} }}",
            wrap(level, deep)
        );
        let nested = host(&format!("fn nested() -> u8 {{ {} }}", wrap(level, deep)));
        let want = blocks(marks(&lifted));
        assert_eq!(blocks(marks(&nested)), want, "{nested}");
        assert_eq!(blocks(marks(&host(&methods))), want, "{methods}");
        assert_eq!(want, level, "{nested}");
    }
}

#[test]
fn wrappers() {
    let scoped = "impl Clone for Unit { fn clone(&self) -> Self { Unit } }";
    assert_eq!(kinds(&host(scoped), grammar::Kind::Scope), 2);
    assert_eq!(kinds(&host(scoped), grammar::Kind::Item), 3);
    let fenced = format!(
        "{}\nmod inner {{ pub fn helper() -> u8 {{ 1 }} }}",
        foreign()
    );
    assert_eq!(kinds(&host(&fenced), grammar::Kind::Scope), 2);
    assert_eq!(kinds(&host(&fenced), grammar::Kind::Item), 6);
}

#[test]
fn coverage() {
    let refused = vec![("coverage".to_string(), "unparsed region".to_string())];
    for item in [
        "impl Thing { 1 }",
        "struct Thing { 1 }",
        "trait Thing { 1 }",
    ] {
        assert_eq!(marks(&host(item)), refused, "{item}");
    }
}

#[test]
fn keywords() {
    let plain = "let header = table as *const Row;
let other = table as *mut Row;
let name: &'static str = \"x\";
let pick: fn(u8) -> u8 = double;
let call: unsafe extern \"C\" fn(u8) -> u8 = double;
let seed = const { 1 };
let task = async move { 2 };
let joined = set.union(set);
'outer: loop { break 'outer; }
unsafe { go() }";
    let text = host(plain);
    assert_eq!(marks(&text), Marks::new());
    assert_eq!(kinds(&text, grammar::Kind::Item), 1);
    assert_eq!(kinds(&text, grammar::Kind::Loose), 0);
    assert_eq!(kinds(&text, grammar::Kind::Word), 9);
}
