type Marks = Vec<(String, String)>;

const FIELDS: &str = "
    /// The listening port.
    #[arg(long)]
    alpha_beta: u8,
    pub gamma_delta: Vec<Option<HashMap<String, Vec<u8>>>>,
    #[serde(rename = \"epsilon\")]
    #[arg(long, default_value_t = 20)]
    epsilon_zeta: u16,
";

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

fn kinds(text: &str, kind: grammar::Kind) -> usize {
    fn walk(node: &kernel::Node, kind: grammar::Kind) -> usize {
        let own = usize::from(node.kind == kind);
        own + node.kids.iter().map(|kid| walk(kid, kind)).sum::<usize>()
    }
    walk(&kernel::structure(&source(text)), kind)
}

fn host(body: &str) -> String {
    format!("pub fn host(flag: bool) -> u8 {{\n{body}\n1\n}}\n")
}

#[test]
fn words() {
    let named = marks(&format!("pub struct Circle {{{FIELDS}}}"));
    assert_eq!(named.len(), 4, "{named:?}");
    for text in [
        format!("pub enum Shape {{ Circle {{{FIELDS}}} }}"),
        format!("pub enum Shape {{ Plain, Circle {{{FIELDS}}}, Pair(u8, u8) }}"),
        format!("enum Shape {{ #[command(about = \"x\")] Circle {{{FIELDS}}} = 1, }}"),
        host(&format!("enum Shape {{ Circle {{{FIELDS}}} }}")),
    ] {
        assert_eq!(marks(&text), named, "{text}");
    }
}

#[test]
fn records() {
    let named = format!("pub struct Circle {{{FIELDS}}}");
    let variant = format!("pub enum Shape {{ Circle {{{FIELDS}}}, Pair(u8, u8), Plain = 3 }}");
    for kind in [
        grammar::Kind::Record,
        grammar::Kind::Label,
        grammar::Kind::Field,
    ] {
        assert_eq!(kinds(&variant, kind), kinds(&named, kind), "{variant}");
    }
    assert_eq!(kinds(&variant, grammar::Kind::Record), 1);
    assert_eq!(kinds(&variant, grammar::Kind::Field), 3);
}

#[test]
fn levels() {
    let deep = "if flag { if flag { if flag { go(); } } }";
    let enumed = host(&format!("enum Shape {{ Circle {{{FIELDS}}} }}\n{deep}"));
    assert_eq!(kinds(&enumed, grammar::Kind::Scope), 4);
    assert_eq!(
        marks(&enumed),
        marks(&host(&format!("struct Circle {{{FIELDS}}}\n{deep}")))
    );
}

#[test]
fn coverage() {
    let refused = vec![("coverage".to_string(), "unparsed region".to_string())];
    for item in [
        "enum Shape { Circle { 1 } }",
        "enum Shape { Circle { alpha: u8, 2 } }",
        "enum Shape { Circle { alpha: u8 }, Square { ? } }",
    ] {
        assert_eq!(marks(item), refused, "{item}");
        assert_eq!(marks(&host(item)), refused, "{item}");
    }
}

#[test]
fn others() {
    let text = "pub enum Shape { Plain, Pair(u8, Vec<u8>), Fixed = 3, Wide(u8, u8, u8) = 4 }";
    assert_eq!(marks(text), Marks::new());
    assert_eq!(kinds(text, grammar::Kind::Word), 5);
    assert_eq!(kinds(text, grammar::Kind::Record), 0);
    assert_eq!(kinds(text, grammar::Kind::Loose), 0);
}
