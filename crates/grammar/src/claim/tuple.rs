use crate::lex::Token;
use crate::{Cst, Kind, Source, Span};

mod nest;
mod web;

#[derive(Clone, Copy, PartialEq)]
enum Dialect {
    Rust,
    Python,
    Web,
}

struct Read<'a> {
    source: &'a Source,
    held: &'a [Token],
    pairs: Vec<Option<usize>>,
    dialect: Dialect,
    found: Vec<Cst>,
}

pub(crate) fn scan(source: &Source, held: &[Token]) -> Vec<Cst> {
    let dialect = if source.path.ends_with(".py") {
        Dialect::Python
    } else if source.path.ends_with(".rs") {
        Dialect::Rust
    } else {
        Dialect::Web
    };
    Read::new(source, held, dialect).scan()
}

impl<'a> Read<'a> {
    fn new(source: &'a Source, held: &'a [Token], dialect: Dialect) -> Self {
        Self {
            source,
            held,
            pairs: vec![None; held.len()],
            dialect,
            found: Vec::new(),
        }
    }

    fn scan(mut self) -> Vec<Cst> {
        self.pair();
        self.angles();
        self.groups();
        if self.dialect == Dialect::Python {
            self.lines();
        }
        self.found.sort_by_key(|node| node.span.start);
        self.found
            .dedup_by_key(|node| (node.span.start, node.span.end));
        self.found
    }

    fn glyph(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }

    fn pair(&mut self) {
        let mut stack: Vec<(usize, char)> = Vec::new();
        for at in 0..self.held.len() {
            match self.glyph(at) {
                "(" => stack.push((at, '(')),
                "[" => stack.push((at, '[')),
                "{" => stack.push((at, '{')),
                ")" | "]" | "}" => {
                    let Some((open, glyph)) = stack.pop() else {
                        continue;
                    };
                    self.link(open, glyph, at);
                }
                _ => {}
            }
        }
    }

    fn link(&mut self, open: usize, glyph: char, close: usize) {
        if !closes(glyph, self.glyph(close)) {
            return;
        }
        self.pairs[open] = Some(close);
        self.pairs[close] = Some(open);
    }

    fn groups(&mut self) {
        for open in 0..self.held.len() {
            let Some(close) = self.pairs[open] else {
                continue;
            };
            let admitted = match self.glyph(open) {
                "(" => self.dialect != Dialect::Web && !self.invoked(open),
                "[" => match self.dialect {
                    Dialect::Web => web::typed(self, open, close),
                    Dialect::Python => self.unpacked(open, close),
                    Dialect::Rust => false,
                },
                _ => false,
            };
            if admitted {
                self.group(open, close);
            }
        }
    }

    fn invoked(&self, open: usize) -> bool {
        if open == 0 {
            return false;
        }
        let prior = self.glyph(open - 1);
        let generic = prior == ">" && self.glyph(open.saturating_sub(2)) != "-";
        let closed = matches!(prior, ")" | "]" | "}" | "!" | ".");
        let named = self.held[open - 1].name == "IDENT" && !keyword(prior);
        closed || named || generic
    }

    fn unpacked(&self, open: usize, close: usize) -> bool {
        matches!(self.glyph(close + 1), "=" | "in") || self.glyph(open.saturating_sub(1)) == "for"
    }

    fn group(&mut self, open: usize, close: usize) {
        let Some(parts) = self.parts(open + 1, close) else {
            return;
        };
        let span = Span {
            start: self.held[open].start,
            end: self.held[close].end,
        };
        self.emit(span, parts);
    }

    fn parts(&self, from: usize, to: usize) -> Option<Vec<Span>> {
        let mut parts = Vec::new();
        let mut start = from;
        let mut at = from;
        let mut comma = false;
        while at < to {
            if let Some(next) = self.hop(at, start, to) {
                at = next;
                continue;
            }
            if self.glyph(at) == "," {
                comma = true;
                parts.push(self.span(start, at)?);
                start = at + 1;
            }
            at += 1;
        }
        if start < to {
            parts.push(self.span(start, to)?);
        }
        (comma && parts.len() > 1).then_some(parts)
    }

    fn span(&self, from: usize, to: usize) -> Option<Span> {
        let first = self.held.get(from)?;
        let last = self.held.get(to.checked_sub(1)?)?;
        (first.start < last.end).then_some(Span {
            start: first.start,
            end: last.end,
        })
    }

    fn lines(&mut self) {
        let mut from = 0;
        for at in 0..=self.held.len() {
            if at == self.held.len() || self.held[at].name == "NEWLINE" {
                self.line(from, at);
                from = at + 1;
            }
        }
    }

    fn line(&mut self, from: usize, to: usize) {
        let from = self.layout(from, to);
        if from >= to || !self.linekind(from, to) {
            return;
        }
        let Some(parts) = self.parts(from, to) else {
            return;
        };
        let Some(span) = self.span(from, to) else {
            return;
        };
        self.emit(span, parts);
    }

    fn layout(&self, mut from: usize, to: usize) -> usize {
        while from < to && matches!(self.held[from].name.as_str(), "INDENT" | "DEDENT") {
            from += 1;
        }
        from
    }

    fn linekind(&self, from: usize, to: usize) -> bool {
        if matches!(self.glyph(from), "return" | "yield") {
            return true;
        }
        if (from..to).any(|at| self.glyph(at) == "lambda") {
            return false;
        }
        let mut depth = 0usize;
        for at in from..to {
            match self.glyph(at) {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                "=" if depth == 0 => return true,
                "in" if depth == 0 && self.glyph(from) == "for" => return true,
                _ => {}
            }
        }
        false
    }

    fn emit(&mut self, span: Span, parts: Vec<Span>) {
        self.found.push(Cst {
            kind: Kind::Tuple,
            span,
            kids: parts
                .into_iter()
                .map(|span| Cst {
                    kind: Kind::Position,
                    span,
                    kids: Vec::new(),
                })
                .collect(),
        });
    }
}

fn closes(open: char, close: &str) -> bool {
    matches!((open, close), ('(', ")") | ('[', "]") | ('{', "}"))
}

fn keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "case"
            | "else"
            | "for"
            | "if"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "move"
            | "return"
            | "while"
            | "yield"
    )
}
