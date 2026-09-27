use crate::lex::Token;
use crate::{Cst, Kind, Source, Span};

mod python;
mod rust;
mod web;

pub(super) struct Read<'a> {
    source: &'a Source,
    held: &'a [Token],
    pairs: Vec<Option<usize>>,
}

pub(crate) fn scan(source: &Source, held: &[Token]) -> Vec<Cst> {
    let mut read = Read {
        source,
        held,
        pairs: vec![None; held.len()],
    };
    read.pair();
    if source.path.ends_with(".rs") {
        return rust::scan(&read);
    }
    if source.path.ends_with(".py") {
        return python::scan(&read);
    }
    if source.path.ends_with(".ts") || source.path.ends_with(".tsx") {
        return web::scan(&read);
    }
    Vec::new()
}

impl Read<'_> {
    pub(super) fn glyph(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }

    pub(super) fn named(&self, at: usize) -> bool {
        self.held.get(at).is_some_and(|token| token.name == "IDENT")
    }

    pub(super) fn mate(&self, at: usize) -> Option<usize> {
        self.pairs.get(at).copied().flatten()
    }

    pub(super) fn record(
        &self,
        start: usize,
        name: usize,
        close: usize,
        fields: Vec<usize>,
    ) -> Cst {
        let mut kids = vec![self.node(Kind::Label, name)];
        kids.extend(fields.into_iter().map(|at| self.node(Kind::Field, at)));
        Cst {
            kind: Kind::Record,
            span: Span {
                start: self.held[start].start,
                end: self.held[close].end,
            },
            kids,
        }
    }

    fn node(&self, kind: Kind, at: usize) -> Cst {
        Cst {
            kind,
            span: Span {
                start: self.held[at].start,
                end: self.held[at].end,
            },
            kids: Vec::new(),
        }
    }

    fn pair(&mut self) {
        let mut stack: Vec<(usize, char)> = Vec::new();
        for at in 0..self.held.len() {
            match self.glyph(at) {
                "(" => stack.push((at, '(')),
                "[" => stack.push((at, '[')),
                "{" => stack.push((at, '{')),
                ")" | "]" | "}" => self.close(&mut stack, at),
                _ => {}
            }
        }
    }

    fn close(&mut self, stack: &mut Vec<(usize, char)>, at: usize) {
        let Some((open, glyph)) = stack.pop() else {
            return;
        };
        let paired = matches!(
            (glyph, self.glyph(at)),
            ('(', ")") | ('[', "]") | ('{', "}")
        );
        if paired {
            self.pairs[open] = Some(at);
            self.pairs[at] = Some(open);
        }
    }
}
