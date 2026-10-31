mod claim;
mod decision;
mod format;
mod layout;
mod lex;
mod markdown;
mod parse;
mod peg;
mod rules;

use lex::{Slot, Token};

pub struct Source {
    pub path: String,
    pub text: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Root,
    Scope,
    Item,
    Literal,
    Comment,
    Loose,
    Word,
    Test,
    Probe,
    Receiver,
    Param,
    Markup,
    Style,
    Environment,
    Lock,
    Embed,
    Link,
    Decision,
    Atom,
    Tuple,
    Position,
    Record,
    Field,
    Label,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, out: &mut std::fmt::Formatter) -> std::fmt::Result {
        out.write_str(name(*self))
    }
}

fn name(kind: Kind) -> &'static str {
    match kind {
        Kind::Root => "root",
        Kind::Scope => "scope",
        Kind::Item => "item",
        Kind::Literal => "literal",
        Kind::Comment => "comment",
        Kind::Loose => "loose",
        Kind::Word => "word",
        Kind::Test => "test",
        Kind::Probe => "probe",
        Kind::Receiver => "receiver",
        Kind::Param => "param",
        Kind::Markup => "markup",
        Kind::Style => "style",
        Kind::Environment => "environment",
        Kind::Lock => "lock",
        Kind::Embed => "embed",
        Kind::Link => "link",
        Kind::Decision => "decision",
        Kind::Atom => "atom",
        Kind::Tuple => "tuple",
        Kind::Position => "position",
        Kind::Record => "record",
        Kind::Field => "field",
        Kind::Label => "label",
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Reader {
    Markdown,
    Rust,
    Python,
    Ts,
    Tsx,
    Svelte,
    Style,
}

pub const READERS: [(&str, Reader); 14] = [
    ("md", Reader::Markdown),
    ("rs", Reader::Rust),
    ("py", Reader::Python),
    ("ts", Reader::Ts),
    ("mts", Reader::Ts),
    ("cts", Reader::Ts),
    ("tsx", Reader::Tsx),
    ("js", Reader::Tsx),
    ("mjs", Reader::Tsx),
    ("cjs", Reader::Tsx),
    ("jsx", Reader::Tsx),
    ("svelte", Reader::Svelte),
    ("css", Reader::Style),
    ("scss", Reader::Style),
];

impl Reader {
    pub fn named(extension: &str) -> Option<Self> {
        READERS
            .iter()
            .find(|(name, _)| *name == extension)
            .map(|(_, reader)| *reader)
    }

    pub fn of(path: &str) -> Option<Self> {
        let (_, extension) = path.rsplit_once('.')?;
        Self::named(extension)
    }
}

pub struct Span {
    pub start: usize,
    pub end: usize,
}

pub struct Cst {
    pub kind: Kind,
    pub span: Span,
    pub kids: Vec<Cst>,
}

pub fn parse(source: &Source) -> Cst {
    let end = source.text.len();
    match Reader::of(&source.path) {
        Some(Reader::Markdown) => markdown::parse(source),
        Some(Reader::Rust) => {
            let mut root = braces(source, rules::rust());
            claim::rust(&mut root, source);
            root
        }
        Some(Reader::Python) => {
            let mut root = indented(source, rules::python());
            claim::python(&mut root, source);
            root
        }
        Some(Reader::Ts) => rules::web(rules::ts(), source),
        Some(Reader::Tsx) => rules::web(rules::tsx(), source),
        Some(Reader::Svelte) => rules::web(rules::svelte(), source),
        Some(Reader::Style) => rules::swatch(end),
        None => Cst {
            kind: Kind::Root,
            span: Span { start: 0, end },
            kids: Vec::new(),
        },
    }
}

pub(crate) fn hook(node: &mut Cst, text: &str) {
    if node.kind != Kind::Word {
        return;
    }
    let name = text.get(node.span.start..node.span.end).unwrap_or("");
    if hooked(name) {
        node.span.start += 3;
    }
}

fn hooked(name: &str) -> bool {
    name.len() > 3 && name.starts_with("use") && name.as_bytes()[3].is_ascii_uppercase()
}

pub(crate) fn sheet(node: &mut Cst, text: &str) {
    if node.kind != Kind::Item {
        return;
    }
    let body = text.get(node.span.start..node.span.end).unwrap_or("");
    if !body.starts_with("import") {
        return;
    }
    if let Some(span) = woven(body, node.span.start) {
        node.kids.push(Cst {
            kind: Kind::Style,
            span,
            kids: Vec::new(),
        });
    }
}

fn woven(body: &str, base: usize) -> Option<Span> {
    let shut = body.rfind(['"', '\''])?;
    let mark = body.as_bytes()[shut];
    let open = body[..shut].rfind(mark as char)?;
    if !styled(&body[open + 1..shut]) {
        return None;
    }
    Some(Span {
        start: base + open,
        end: base + shut + 1,
    })
}

fn styled(inner: &str) -> bool {
    Reader::of(inner) == Some(Reader::Style)
}

pub(crate) fn braces(source: &Source, rules: &(Vec<format::Rule>, Vec<format::Rule>)) -> Cst {
    let bytes = source.text.as_bytes();
    let (sig, comments) = sift(lex::web(rules, bytes));
    let mut root = parse::run(&rules.1, &sig, bytes, source.text.len());
    for span in comments {
        root.kids.push(note(span));
    }
    root
}

fn indented(source: &Source, rules: &(Vec<format::Rule>, Vec<format::Rule>)) -> Cst {
    let bytes = source.text.as_bytes();
    let (lexers, parsers) = rules;
    let (sig, comments) = sift(lex::lex(lexers, bytes));
    let sig = layout::apply(sig, bytes);
    let mut root = parse::run(parsers, &sig, bytes, source.text.len());
    for span in comments {
        root.kids.push(note(span));
    }
    root
}

fn sift(tokens: Vec<Token>) -> (Vec<Token>, Vec<Span>) {
    let mut sig = Vec::new();
    let mut comments = Vec::new();
    for token in tokens {
        sort(token, &mut sig, &mut comments);
    }
    (sig, comments)
}

fn sort(token: Token, sig: &mut Vec<Token>, comments: &mut Vec<Span>) {
    if matches!(token.kind, Slot::Comment) {
        comments.push(Span {
            start: token.start,
            end: token.end,
        });
        return;
    }
    sig.push(token);
}

fn note(span: Span) -> Cst {
    Cst {
        kind: Kind::Comment,
        span,
        kids: Vec::new(),
    }
}
