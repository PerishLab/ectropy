use crate::format;
use crate::lex::{self, Slot, Token};
use crate::rules;
use crate::{Cst, Kind, Source, Span};

pub(crate) fn rust(root: &mut Cst, source: &Source) {
    Sweep::load(source, &rules::rust().0).stds(root);
}

pub(crate) fn web(root: &mut Cst, source: &Source, rules: &(Vec<format::Rule>, Vec<format::Rule>)) {
    Sweep::load(source, &rules.0).members(root);
}

struct Sweep<'a> {
    source: &'a Source,
    held: Vec<Token>,
}

impl<'a> Sweep<'a> {
    fn load(source: &'a Source, lexers: &[format::Rule]) -> Self {
        let held = lex::lex(lexers, source.text.as_bytes())
            .into_iter()
            .filter(|token| matches!(token.kind, Slot::Emit))
            .collect();
        Sweep { source, held }
    }

    fn glyph(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }

    fn stds(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.rooted(at) {
                self.drawn(root, at);
            }
        }
    }

    fn rooted(&self, at: usize) -> bool {
        self.glyph(at) == "std" && self.glyph(at + 1) == ":" && self.glyph(at + 2) == ":"
    }

    fn drawn(&self, root: &mut Cst, at: usize) {
        match self.glyph(at + 3) {
            "env" => stamp(root, self.held[at].start, self.held[at + 3].end),
            "{" => self.grouped(root, at + 4),
            _ => {}
        }
    }

    fn grouped(&self, root: &mut Cst, from: usize) {
        let mut depth = 1;
        let mut fresh = true;
        let mut at = from;
        while at < self.held.len() && depth > 0 {
            let text = self.glyph(at);
            match text {
                "{" => depth += 1,
                "}" => depth -= 1,
                "env" => self.member(root, at, depth, fresh),
                _ => {}
            }
            fresh = text == "{" || (text == "," && depth == 1);
            at += 1;
        }
    }

    fn member(&self, root: &mut Cst, at: usize, depth: usize, fresh: bool) {
        if depth == 1 && fresh {
            stamp(root, self.held[at].start, self.held[at].end);
        }
    }

    fn members(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.handed(at) {
                stamp(root, self.held[at].start, self.held[at + 2].end);
            }
        }
    }

    fn handed(&self, at: usize) -> bool {
        let head = self.glyph(at);
        (head == "Deno" || head == "process")
            && self.glyph(at + 1) == "."
            && self.glyph(at + 2) == "env"
    }
}

fn stamp(root: &mut Cst, start: usize, end: usize) {
    root.kids.push(Cst {
        kind: Kind::Environment,
        span: Span { start, end },
        kids: Vec::new(),
    });
}
