use crate::format;
use crate::{Cst, Kind, Source, Span, braces, hook, sheet};
use std::sync::OnceLock;

const RUST: &str = include_str!("../grammars/rust.g4");
const TS: &str = include_str!("../grammars/ts.g4");
const JSX: &str = include_str!("../grammars/jsx.g4");

pub(crate) fn rust() -> &'static (Vec<format::Rule>, Vec<format::Rule>) {
    static CELL: OnceLock<(Vec<format::Rule>, Vec<format::Rule>)> = OnceLock::new();
    CELL.get_or_init(|| split(format::load(RUST)))
}

pub(crate) fn ts() -> &'static (Vec<format::Rule>, Vec<format::Rule>) {
    static CELL: OnceLock<(Vec<format::Rule>, Vec<format::Rule>)> = OnceLock::new();
    CELL.get_or_init(|| split(format::load(TS)))
}

pub(crate) fn tsx() -> &'static (Vec<format::Rule>, Vec<format::Rule>) {
    static CELL: OnceLock<(Vec<format::Rule>, Vec<format::Rule>)> = OnceLock::new();
    CELL.get_or_init(|| split(format::load(&[TS, JSX].concat())))
}

fn split(all: Vec<format::Rule>) -> (Vec<format::Rule>, Vec<format::Rule>) {
    let mut lexers = Vec::new();
    let mut parsers = Vec::new();
    for rule in all {
        route(rule, &mut lexers, &mut parsers);
    }
    (lexers, parsers)
}

fn route(rule: format::Rule, lexers: &mut Vec<format::Rule>, parsers: &mut Vec<format::Rule>) {
    if upper(&rule.name) {
        lexers.push(rule);
        return;
    }
    parsers.push(rule);
}

fn upper(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
}

pub(crate) fn web(rules: &(Vec<format::Rule>, Vec<format::Rule>), source: &Source) -> Cst {
    let mut root = braces(source, rules);
    dialect(&mut root, &source.text);
    root
}

pub(crate) fn swatch(end: usize) -> Cst {
    Cst {
        kind: Kind::Root,
        span: Span { start: 0, end },
        kids: vec![Cst {
            kind: Kind::Style,
            span: Span { start: 0, end },
            kids: Vec::new(),
        }],
    }
}

fn dialect(node: &mut Cst, text: &str) {
    hook(node, text);
    sheet(node, text);
    for kid in &mut node.kids {
        dialect(kid, text);
    }
}
