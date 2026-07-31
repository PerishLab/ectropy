mod regex;
mod syntax;

use crate::lex::Token;
use crate::{Cst, Kind, Source, Span};

struct Model {
    span: Span,
    atoms: Vec<Span>,
}

#[derive(Default)]
struct Read {
    models: Vec<Model>,
    whole: Option<Model>,
}

impl Read {
    fn settle(&mut self, mut other: Read) {
        self.models.append(&mut other.models);
        if let Some(model) = other.whole {
            self.models.push(model);
        }
    }
}

struct Term {
    models: Vec<Model>,
    atoms: Vec<Span>,
}

struct Parse<'a> {
    source: &'a Source,
    held: &'a [Token],
    mates: Vec<Option<usize>>,
    scopes: Vec<usize>,
    marks: Vec<(usize, usize)>,
    veils: Vec<Option<usize>>,
}

pub(crate) fn scan(source: &Source, held: &[Token], root: &Cst) -> Vec<Cst> {
    let parse = Parse::load(source, held, root);
    let mut read = parse.range(0, held.len());
    if let Some(model) = read.whole {
        read.models.push(model);
    }
    read.models.retain(|model| parse.code(model.span.start));
    read.models.sort_by_key(|model| model.span.start);
    read.models.into_iter().map(node).collect()
}

impl<'a> Parse<'a> {
    fn load(source: &'a Source, held: &'a [Token], root: &Cst) -> Self {
        let mut scopes = Vec::new();
        let mut marks = Vec::new();
        collect(root, &mut scopes, &mut marks);
        let mut parse = Self {
            source,
            held,
            mates: vec![None; held.len()],
            scopes,
            marks,
            veils: regex::scan(source, held),
        };
        parse.pair();
        parse
    }

    fn range(&self, from: usize, to: usize) -> Read {
        if !self.cuts(from, to) {
            return self.segment(from, to);
        }
        let mut out = Read::default();
        let mut start = from;
        let mut at = from;
        while at < to {
            if self.veil(at).is_some() {
                at = self.step(at, to);
                continue;
            }
            if matches!(self.glyph(at), "(" | "[") || (self.glyph(at) == "{" && !self.boundary(at))
            {
                at = self.step(at, to);
                continue;
            }
            if self.glyph(at) == "{" && self.boundary(at) {
                out.settle(self.segment(start, at));
                let end = self.mate(at, to);
                if end > at {
                    out.settle(self.range(at + 1, end));
                }
                at = end.saturating_add(1);
                start = at;
                continue;
            }
            if self.hard(at) {
                out.settle(self.segment(start, at));
                at += 1;
                start = at;
                continue;
            }
            at += 1;
        }
        out.settle(self.segment(start, to));
        out
    }

    fn segment(&self, from: usize, to: usize) -> Read {
        if from >= to {
            return Read::default();
        }
        let ops = self.ops(from, to);
        if let Some(at) = ops.iter().copied().find(|at| self.prefix(from, *at)) {
            let mut out = Read::default();
            out.settle(self.range(from, at));
            out.settle(self.range(at + 2, to));
            return out;
        }
        if ops.is_empty() {
            return self.plain(from, to);
        }
        if !valid(from, to, &ops) {
            return self.plain(from, to);
        }
        let mut out = Read::default();
        let mut atoms = Vec::new();
        let mut start = from;
        for at in ops {
            let mut term = self.term(start, at);
            out.models.append(&mut term.models);
            atoms.append(&mut term.atoms);
            start = at + 2;
        }
        let mut term = self.term(start, to);
        out.models.append(&mut term.models);
        atoms.append(&mut term.atoms);
        out.whole = Some(Model {
            span: self.span(from, to),
            atoms,
        });
        out
    }

    fn plain(&self, from: usize, to: usize) -> Read {
        if let Some((start, end)) = self.wrapper(from, to) {
            let inner = self.range(start + 1, end);
            if inner.whole.is_some() {
                return inner;
            }
            let mut out = Read::default();
            out.settle(inner);
            return out;
        }
        self.nested(from, to)
    }

    fn term(&self, from: usize, to: usize) -> Term {
        if let Some((start, end)) = self.wrapper(from, to) {
            let mut inner = self.range(start + 1, end);
            if let Some(model) = inner.whole {
                return Term {
                    models: inner.models,
                    atoms: model.atoms,
                };
            }
            let mut models = Vec::new();
            models.append(&mut inner.models);
            return Term {
                models,
                atoms: vec![self.span(from, to)],
            };
        }
        let mut inner = self.nested(from, to);
        if let Some(model) = inner.whole {
            inner.models.push(model);
        }
        Term {
            models: inner.models,
            atoms: vec![self.span(from, to)],
        }
    }

    fn nested(&self, from: usize, to: usize) -> Read {
        let mut out = Read::default();
        let mut at = from;
        while at < to {
            if self.veil(at).is_some() {
                at = self.step(at, to);
                continue;
            }
            if matches!(self.glyph(at), "(" | "[" | "{") {
                let end = self.mate(at, to);
                if end > at {
                    out.settle(self.range(at + 1, end));
                }
                at = end.saturating_add(1);
                continue;
            }
            at += 1;
        }
        out
    }

    fn wrapper(&self, from: usize, to: usize) -> Option<(usize, usize)> {
        let mut at = from;
        while at < to && self.glyph(at) == "!" {
            at += 1;
        }
        if at >= to || self.glyph(at) != "(" {
            return None;
        }
        let end = self.mate(at, to);
        (end + 1 == to).then_some((at, end))
    }

    fn ops(&self, from: usize, to: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut at = from;
        while at + 1 < to {
            if self.veil(at).is_some() {
                at = self.step(at, to);
                continue;
            }
            if matches!(self.glyph(at), "(" | "[" | "{") {
                at = self.step(at, to);
                continue;
            }
            if self.double(at, "&") || self.double(at, "|") {
                found.push(at);
                at += 2;
                continue;
            }
            at += 1;
        }
        found
    }

    fn prefix(&self, from: usize, at: usize) -> bool {
        if from == at {
            return true;
        }
        if self.glyph(at) != "|" {
            return false;
        }
        (from..at).all(|held| matches!(self.glyph(held), "async" | "move" | "return" | "static"))
    }
}

fn valid(from: usize, to: usize, ops: &[usize]) -> bool {
    let mut start = from;
    for at in ops {
        if start >= *at {
            return false;
        }
        start = at + 2;
    }
    start < to
}

fn collect(node: &Cst, scopes: &mut Vec<usize>, marks: &mut Vec<(usize, usize)>) {
    match node.kind {
        Kind::Scope => scopes.push(node.span.start),
        Kind::Markup => marks.push((node.span.start, node.span.end)),
        _ => {}
    }
    for kid in &node.kids {
        collect(kid, scopes, marks);
    }
}

fn node(model: Model) -> Cst {
    Cst {
        kind: Kind::Decision,
        span: model.span,
        kids: model
            .atoms
            .into_iter()
            .map(|span| Cst {
                kind: Kind::Atom,
                span,
                kids: Vec::new(),
            })
            .collect(),
    }
}
