mod claim;
mod decision;
mod format;
mod lex;
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
    Decision,
    Atom,
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
        Kind::Decision => "decision",
        Kind::Atom => "atom",
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
    if source.path.ends_with(".md") {
        return heads(source);
    }
    if source.path.ends_with(".rs") {
        let mut root = braces(source, rules::rust());
        claim::rust(&mut root, source);
        return root;
    }
    if source.path.ends_with(".ts") {
        let mut root = rules::web(rules::ts(), source);
        claim::web(&mut root, source, rules::ts());
        return root;
    }
    if source.path.ends_with(".tsx") {
        let mut root = rules::web(rules::tsx(), source);
        claim::web(&mut root, source, rules::tsx());
        return root;
    }
    if source.path.ends_with(".scss") || source.path.ends_with(".css") {
        return rules::swatch(source.text.len());
    }
    Cst {
        kind: Kind::Root,
        span: Span {
            start: 0,
            end: source.text.len(),
        },
        kids: Vec::new(),
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
    inner.ends_with(".scss") || inner.ends_with(".css")
}

pub(crate) fn braces(source: &Source, rules: &(Vec<format::Rule>, Vec<format::Rule>)) -> Cst {
    let bytes = source.text.as_bytes();
    let (lexers, parsers) = rules;
    let (sig, comments) = sift(lex::lex(lexers, bytes));
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

struct Head {
    level: usize,
    start: usize,
    kids: Vec<Cst>,
}

fn heads(source: &Source) -> Cst {
    let mut rows = Rows {
        bytes: source.text.as_bytes(),
        stack: vec![Head {
            level: 0,
            start: 0,
            kids: Vec::new(),
        }],
    };
    let mut at = 0;
    while at < rows.bytes.len() {
        at = rows.row(at);
    }
    rows.settle(1, source.text.len());
    let root = rows.stack.pop().unwrap();
    Cst {
        kind: Kind::Root,
        span: Span {
            start: 0,
            end: source.text.len(),
        },
        kids: root.kids,
    }
}

struct Rows<'a> {
    bytes: &'a [u8],
    stack: Vec<Head>,
}

impl Rows<'_> {
    fn row(&mut self, at: usize) -> usize {
        let end = self.eol(at);
        let level = self.hashes(at, end);
        if level > 0 {
            self.raise(level, at);
        }
        self.step(end)
    }

    fn raise(&mut self, level: usize, start: usize) {
        self.settle(level, start);
        self.stack.push(Head {
            level,
            start,
            kids: Vec::new(),
        });
    }

    fn settle(&mut self, level: usize, at: usize) {
        while self.deep(level) {
            self.close(at);
        }
    }

    fn deep(&self, level: usize) -> bool {
        self.stack.len() > 1 && self.stack.last().unwrap().level >= level
    }

    fn close(&mut self, at: usize) {
        let done = self.stack.pop().unwrap();
        let node = Cst {
            kind: Kind::Scope,
            span: Span {
                start: done.start,
                end: at,
            },
            kids: done.kids,
        };
        self.stack.last_mut().unwrap().kids.push(node);
    }

    fn hashes(&self, at: usize, end: usize) -> usize {
        let mut run = at;
        while run < end && self.bytes[run] == b'#' {
            run += 1;
        }
        self.marked(run, end, run - at)
    }

    fn marked(&self, run: usize, end: usize, count: usize) -> usize {
        if count > 0 && run < end && self.bytes[run] == b' ' {
            count
        } else {
            0
        }
    }

    fn eol(&self, at: usize) -> usize {
        let mut end = at;
        while end < self.bytes.len() && self.bytes[end] != b'\n' {
            end += 1;
        }
        end
    }

    fn step(&self, end: usize) -> usize {
        if end < self.bytes.len() { end + 1 } else { end }
    }
}
