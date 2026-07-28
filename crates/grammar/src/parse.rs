use crate::format::Rule;
use crate::lex::Token;
use crate::peg::{Pat, Quant};
use crate::{Cst, Kind, Span};
use std::collections::HashMap;

struct State<'a> {
    rules: &'a [Rule],
    index: HashMap<&'a str, usize>,
    toks: &'a [Token],
    src: &'a [u8],
}

pub fn run(rules: &[Rule], toks: &[Token], src: &[u8], end: usize) -> Cst {
    let mut index = HashMap::new();
    for (slot, rule) in rules.iter().enumerate() {
        index.insert(rule.name.as_str(), slot);
    }
    let state = State {
        rules,
        index,
        toks,
        src,
    };
    let kids = state.start();
    Cst {
        kind: Kind::Root,
        span: Span { start: 0, end },
        kids,
    }
}

impl<'b> State<'b> {
    fn start(&self) -> Vec<Cst> {
        match self.index.get("unit") {
            Some(idx) => self.sweep(*idx),
            None => Vec::new(),
        }
    }

    fn sweep(&self, unit: usize) -> Vec<Cst> {
        let mut at = 0;
        let mut kids = Vec::new();
        let mut lost: Option<usize> = None;
        while at < self.toks.len() {
            at = self.tick(unit, at, &mut kids, &mut lost);
        }
        self.flush(&mut kids, lost, self.toks.len());
        kids
    }

    fn tick(&self, unit: usize, at: usize, kids: &mut Vec<Cst>, lost: &mut Option<usize>) -> usize {
        match self.rule(unit, at) {
            Some((next, mut got)) if next > at => {
                self.flush(kids, lost.take(), at);
                kids.append(&mut got);
                next
            }
            _ => {
                lost.get_or_insert(at);
                at + 1
            }
        }
    }

    fn flush(&self, kids: &mut Vec<Cst>, lost: Option<usize>, end: usize) {
        if let Some(lo) = lost {
            kids.push(self.loose(lo, end));
        }
    }

    fn loose(&self, lo: usize, end: usize) -> Cst {
        let start = self.toks[lo].start;
        let close = self.toks[end - 1].end;
        Cst {
            kind: Kind::Loose,
            span: Span { start, end: close },
            kids: Vec::new(),
        }
    }

    fn rule(&self, idx: usize, at: usize) -> Option<(usize, Vec<Cst>)> {
        let r = &self.rules[idx];
        let (next, kids) = self.expr(&r.pat, at)?;
        Some(self.wrap(r, at, next, kids))
    }

    fn wrap(&self, r: &Rule, at: usize, next: usize, kids: Vec<Cst>) -> (usize, Vec<Cst>) {
        match kind(r) {
            Some(kind) => (next, vec![node(kind, self.span(at, next), kids)]),
            None => (next, kids),
        }
    }

    fn span(&self, at: usize, next: usize) -> Span {
        let start = if at < self.toks.len() {
            self.toks[at].start
        } else {
            0
        };
        let end = if next > 0 && next <= self.toks.len() {
            self.toks[next - 1].end
        } else {
            start
        };
        Span { start, end }
    }

    fn expr(&self, pat: &Pat, at: usize) -> Option<(usize, Vec<Cst>)> {
        match pat {
            Pat::Lit(want) => self.lit(want, at),
            Pat::Name(name) => self.refer(name, at),
            Pat::Any => self.any(at),
            Pat::Seq(parts) => self.seq(parts, at),
            Pat::Alt(parts) => self.alt(parts, at),
            Pat::Rep(inner, quant) => self.rep(inner, quant, at),
            Pat::Not(inner) => self.not(inner, at),
            Pat::Class(..) => None,
        }
    }

    fn lit(&self, want: &[u8], at: usize) -> Option<(usize, Vec<Cst>)> {
        if at < self.toks.len() && self.text(at) == want {
            return Some((at + 1, Vec::new()));
        }
        None
    }

    fn text(&self, at: usize) -> &[u8] {
        let tok = &self.toks[at];
        &self.src[tok.start..tok.end]
    }

    fn refer(&self, name: &str, at: usize) -> Option<(usize, Vec<Cst>)> {
        match self.index.get(name) {
            Some(idx) => self.rule(*idx, at),
            None => self.term(name, at),
        }
    }

    fn term(&self, name: &str, at: usize) -> Option<(usize, Vec<Cst>)> {
        if at < self.toks.len() && self.toks[at].name == name {
            return Some((at + 1, Vec::new()));
        }
        None
    }

    fn any(&self, at: usize) -> Option<(usize, Vec<Cst>)> {
        if at < self.toks.len() {
            Some((at + 1, Vec::new()))
        } else {
            None
        }
    }

    fn seq(&self, parts: &[Pat], at: usize) -> Option<(usize, Vec<Cst>)> {
        let mut pos = at;
        let mut kids = Vec::new();
        for part in parts {
            let (next, mut got) = self.expr(part, pos)?;
            pos = next;
            kids.append(&mut got);
        }
        Some((pos, kids))
    }

    fn alt(&self, parts: &[Pat], at: usize) -> Option<(usize, Vec<Cst>)> {
        for part in parts {
            if let Some(hit) = self.expr(part, at) {
                return Some(hit);
            }
        }
        None
    }

    fn rep(&self, inner: &Pat, quant: &Quant, at: usize) -> Option<(usize, Vec<Cst>)> {
        match quant {
            Quant::Opt => Some(self.opt(inner, at)),
            Quant::Star => Some(self.star(inner, at)),
            Quant::Plus => self.plus(inner, at),
        }
    }

    fn opt(&self, inner: &Pat, at: usize) -> (usize, Vec<Cst>) {
        match self.expr(inner, at) {
            Some(hit) => hit,
            None => (at, Vec::new()),
        }
    }

    fn star(&self, inner: &Pat, at: usize) -> (usize, Vec<Cst>) {
        let mut pos = at;
        let mut kids = Vec::new();
        while let Some((next, mut got)) = self.expr(inner, pos) {
            if next == pos {
                break;
            }
            pos = next;
            kids.append(&mut got);
        }
        (pos, kids)
    }

    fn plus(&self, inner: &Pat, at: usize) -> Option<(usize, Vec<Cst>)> {
        let (end, kids) = self.star(inner, at);
        if end > at { Some((end, kids)) } else { None }
    }

    fn not(&self, inner: &Pat, at: usize) -> Option<(usize, Vec<Cst>)> {
        match self.expr(inner, at) {
            Some(_) => None,
            None => Some((at, Vec::new())),
        }
    }
}

fn kind(r: &Rule) -> Option<Kind> {
    match r.tag.as_deref() {
        Some("scope") => Some(Kind::Scope),
        Some("item") => Some(Kind::Item),
        Some("literal") => Some(Kind::Literal),
        Some("word") => Some(Kind::Word),
        Some("test") => Some(Kind::Test),
        Some("probe") => Some(Kind::Probe),
        Some("receiver") => Some(Kind::Receiver),
        Some("param") => Some(Kind::Param),
        Some("markup") => Some(Kind::Markup),
        Some("style") => Some(Kind::Style),
        _ => None,
    }
}

fn node(kind: Kind, span: Span, kids: Vec<Cst>) -> Cst {
    Cst { kind, span, kids }
}
