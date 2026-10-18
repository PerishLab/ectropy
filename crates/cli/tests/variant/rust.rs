use super::{Marks, host, kinds, marks};

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
