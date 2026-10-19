use super::{Marks, host, kinds, marks, over};

const BODY: &str = "let alpha_beta = 1;
if a == 1 { 1 } else if a == 2 { 2 } else { if true { if true { if true { 0 } else { 0 } } else { 0 } } else { alpha_beta } }";

fn ctl() -> Marks {
    marks("pub fn table(a: u8) -> u8 { let alpha_beta = 1;
if a == 1 { 1 } else if a == 2 { 2 } else { if true { if true { if true { 0 } else { 0 } } else { 0 } } else { alpha_beta } } }")
}

#[test]
fn inits() {
    let want = ctl();
    let block = format!("pub const TABLE: u8 = {{ {BODY} }};");
    assert_eq!(marks(&block), want, "module const block");
    let closure = format!("pub static RUN: fn(u8) -> u8 = |a| {{ {BODY} }};");
    assert_eq!(marks(&closure), want, "module static closure");
    let inside = host(&format!("const TABLE: u8 = {{ {BODY} }};"));
    let nested = marks(&inside);
    assert!(nested.iter().any(|m| m.0 == "word"), "{nested:?}");
    assert!(nested.iter().any(|m| m.0 == "dispatch"), "{nested:?}");
    let assoc =
        format!("pub struct Unit;\nimpl Unit {{\n    pub const TABLE: u8 = {{ {BODY} }};\n}}");
    assert_eq!(marks(&assoc), want, "assoc const");
    let items = "pub const _: () = { impl Thing { fn helper() {} } };";
    assert!(
        kinds(items, grammar::Kind::Item) >= 2,
        "const item block holds items"
    );
}

#[test]
fn macros() {
    let want = marks(&format!(
        "pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 {{ {BODY} }}"
    ));
    let wrapped = format!(
        "wrap! {{\n    pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 {{ {BODY} }}\n}}"
    );
    assert_eq!(marks(&wrapped), want, "module macro brace");
    assert_eq!(
        kinds(&wrapped, grammar::Kind::Scope),
        kinds(
            &format!("pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 {{ {BODY} }}"),
            grammar::Kind::Scope
        ),
        "macro braces add no level"
    );
    let paren = format!(
        "wrap!(\n    pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 {{ {BODY} }}\n);"
    );
    let found = marks(&paren);
    assert!(
        found.iter().any(|m| m.0 == "dispatch"),
        "paren body read: {found:?}"
    );
    assert!(
        found.iter().any(|m| m.0 == "word"),
        "paren body word: {found:?}"
    );
}

#[test]
fn raws() {
    let ctl = marks(
        "pub fn text() -> &'static str { r#\"say hi\"# }
pub fn alpha_beta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }
pub fn quote() -> &'static str { \"x\" }
pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }",
    );
    let hash2 = marks(
        "pub fn text() -> &'static str { r##\"say \"hi\"## }
pub fn alpha_beta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }
pub fn quote() -> &'static str { \"x\" }
pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }",
    );
    assert_eq!(hash2, ctl, "r## raw string");
    let hash1q = marks(
        "pub fn text() -> &'static str { r#\"say \"hi\"# }
pub fn alpha_beta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }
pub fn quote() -> &'static str { \"x\" }
pub fn gamma_delta(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + b + c + d + e }",
    );
    assert_eq!(hash1q, ctl, "r# with quote");
}

#[test]
fn params() {
    let first = "pub fn first((a, b): (u8, u8), c: u8, d: u8, e: u8, f: u8) -> u8 { a + b + c + d + e + f }";
    let middle = "pub fn middle(a: u8, (b, c): (u8, u8), d: u8, e: u8, f: u8, g: u8) -> u8 { a + b + c + d + e + f + g }";
    let attr =
        "pub fn host(a: u8, #[allow(unused)] b: u8, c: u8, d: u8, e: u8) -> u8 { a + c + d + e }";
    let ctl = "pub fn host(a: u8, b: u8, c: u8, d: u8, e: u8) -> u8 { a + c + d + e }";
    assert_eq!(
        marks(first),
        marks(
            "pub fn first(a: (u8, u8), c: u8, d: u8, e: u8, f: u8) -> u8 { a.0 + c + d + e + f }"
        )
    );
    assert!(
        marks(middle).iter().any(|m| m.0 == "param"),
        "{:?}",
        marks(middle)
    );
    assert_eq!(marks(attr), marks(ctl));
}

#[test]
fn guards() {
    let guarded = "pub fn host(a: u8, v: Option<u8>) -> u8 {
    match v {
        Some(x) if x > 3 => 1,
        _ => if a == 1 { 1 } else if a == 2 { 2 } else { 0 },
    }
}";
    let plain = "pub fn host(a: u8, v: Option<u8>) -> u8 {
    match v {
        Some(x) => 1,
        _ => if a == 1 { 1 } else if a == 2 { 2 } else { 0 },
    }
}";
    assert_eq!(marks(guarded), marks(plain));
}

const FOUR: &str = "id: u8, op: u8, headers: u8, want: u8";

const PATTERNS: [&str; 16] = [
    "dock: u8",
    "mut dock: u8",
    "_: u8",
    "State(dock): State<Dock<W>>",
    "axum::extract::State(dock): State<u8>",
    "Path((a, b)): Path<(u8, u8)>",
    "Query(Form { k, .. }): Query<Form>",
    "Pair { a, b }: Pair",
    "(a, b): (u8, u8)",
    "[a, b]: [u8; 2]",
    "&dock: &u8",
    "&mut dock: &mut u8",
    "&mut (a, b): &mut (u8, u8)",
    "ref mut dock: u8",
    "whole @ (a, b): (u8, u8)",
    "#[allow(unused)] dock: u8",
];

fn unread(text: &str) -> Marks {
    let found = marks(text);
    found.into_iter().filter(|m| m.0 == "coverage").collect()
}

#[test]
fn patterns() {
    for seat in PATTERNS {
        let first = format!("pub fn upload<W: Wire>({seat}, {FOUR}) -> u8 {{ 0 }}");
        assert_eq!(marks(&first), over(1), "{first}");
        let later = format!("pub fn upload<W: Wire>(\n    {FOUR},\n    {seat},\n) -> u8 {{ 0 }}");
        assert_eq!(marks(&later), over(1), "{later}");
        let short = format!("pub fn upload<W: Wire>({seat}, id: u8) -> u8 {{ 0 }}");
        assert_eq!(kinds(&short, grammar::Kind::Receiver), 1, "{short}");
        assert_eq!(kinds(&short, grammar::Kind::Param), 1, "{short}");
    }
}

#[test]
fn selves() {
    for seat in [
        "self",
        "&self",
        "&mut self",
        "mut self",
        "&'a self",
        "&'a mut self",
        "&'_ self",
    ] {
        let text = format!(
            "pub struct S;\nimpl S {{\n    pub fn f<'a>({seat}, dock: u8, {FOUR}) -> u8 {{ 0 }}\n}}"
        );
        assert_eq!(marks(&text), over(1), "{text}");
    }
}

#[test]
fn tails() {
    let comma = format!("pub fn upload(dock: u8, {FOUR},) -> u8 {{ 0 }}");
    assert_eq!(marks(&comma), over(1), "{comma}");
    let dots = format!("extern \"C\" {{\n    pub fn printf(dock: u8, {FOUR}, ...) -> i32;\n}}");
    assert_eq!(marks(&dots), over(1), "{dots}");
    let arrow = format!("pub fn upload<F: Fn() -> u8>(dock: F, {FOUR}) -> u8 {{ 0 }}");
    assert_eq!(marks(&arrow), over(1), "{arrow}");
    for clean in [
        "pub fn upload() -> u8 { 0 }",
        "pub fn upload<F: Fn(u8) -> u8>() -> u8 { 0 }",
        "pub struct S;\nimpl S {\n    pub fn f(&self) {}\n}",
    ] {
        assert_eq!(marks(clean), Vec::new(), "{clean}");
    }
}

#[test]
fn refusals() {
    let hit = vec![("coverage".to_string(), "unparsed region".to_string())];
    for text in [
        "macro_rules! m {\n    ($t:ty) => {\n        fn f($b: $t) {}\n    };\n}",
        "macro_rules! m {\n    ($t:ty) => {\n        fn f(a: u8, $b: $t, c: u8) {}\n    };\n}",
        "macro_rules! m {\n    ($($p:tt)*) => {\n        fn f $($p)* {}\n    };\n}",
    ] {
        assert_eq!(unread(text), hit, "{text}");
    }
}

#[test]
fn extractors() {
    let free = |seat: &str| {
        ["a", "b", "c", "d"]
            .map(|name| format!("pub async fn {name}({seat}: State<Dock>, id: u8) {{}}\n"))
            .concat()
    };
    let pattern = marks(&free("State(dock)"));
    let names = "a, b, c, d; see: ectropy cookbook structure.receiver";
    let note = format!("4 functions share State(dock): {names}");
    assert_eq!(pattern, vec![("receiver".to_string(), note)]);
    let ident = marks(&free("dock"));
    let note = format!("4 functions share dock: {names}");
    assert_eq!(ident, vec![("receiver".to_string(), note)]);
    let fold = format!(
        "pub struct Routes;\nimpl Routes {{\n{}}}",
        free("State(dock)")
    );
    assert_eq!(marks(&fold), Vec::new(), "{fold}");
}

#[test]
fn renames() {
    let hit = |name: &str| vec![("word".to_string(), name.to_string())];
    assert_eq!(
        marks("pub use std::fmt::Display as alpha_beta;"),
        hit("alpha_beta")
    );
    assert_eq!(
        marks("pub use std::fmt::{Debug, Display as AlphaBeta};"),
        hit("AlphaBeta")
    );
    assert_eq!(
        marks("pub use crate::{a::{b as alpha_beta, c}, d::*};"),
        hit("alpha_beta")
    );
    assert_eq!(
        marks("pub(crate) use crate::x as gamma_delta;"),
        hit("gamma_delta")
    );
    assert_eq!(
        marks("pub fn host() { use std::fmt::Display as AlphaBeta; }"),
        hit("AlphaBeta")
    );
    for clean in [
        "pub use std::fmt::Display;",
        "pub use std::fmt::{Debug, Display};",
        "pub use std::fmt::*;",
        "use std::fmt::Display as _;",
        "extern crate core as alpha_beta;",
        "pub fn f(alpha_beta: u8) -> u8 { alpha_beta }",
        "pub fn f(v: (u8, u8)) -> u8 { let (alpha_beta, g) = v; alpha_beta + g }",
    ] {
        assert_eq!(marks(clean), Vec::new(), "{clean}");
    }
}
