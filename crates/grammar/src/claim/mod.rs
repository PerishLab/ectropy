use crate::format;
use crate::lex::{self, Slot, Token};
use crate::rules;
use crate::{Cst, Kind, Source, Span};

mod embed;
mod link;
mod record;
mod tuple;

pub(crate) fn rust(root: &mut Cst, source: &Source) {
    let sweep = Sweep::load(source, &rules::rust().0);
    sweep.decisions(root);
    sweep.records(root);
    sweep.tuples(root);
    sweep.stds(root);
    sweep.anchors(root);
    sweep.wires(root);
}

pub(crate) fn web(root: &mut Cst, source: &Source, rules: &(Vec<format::Rule>, Vec<format::Rule>)) {
    let sweep = Sweep::load(source, &rules.0);
    sweep.decisions(root);
    sweep.records(root);
    sweep.tuples(root);
    sweep.members(root);
    sweep.metas(root);
    sweep.specifiers(root);
}

pub(crate) fn python(root: &mut Cst, source: &Source) {
    let sweep = Sweep::laid(source, &rules::python().0);
    sweep.decisions(root);
    sweep.records(root);
    sweep.tuples(root);
    sweep.docs(root);
    sweep.environs(root);
    sweep.files(root);
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

    fn laid(source: &'a Source, lexers: &[format::Rule]) -> Self {
        let held = lex::lex(lexers, source.text.as_bytes())
            .into_iter()
            .filter(|token| matches!(token.kind, Slot::Emit))
            .collect();
        let held = crate::layout::apply(held, source.text.as_bytes());
        Sweep { source, held }
    }

    fn glyph(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }

    fn decisions(&self, root: &mut Cst) {
        let found = crate::decision::scan(self.source, &self.held, root);
        root.kids.extend(found);
    }

    fn tuples(&self, root: &mut Cst) {
        root.kids.extend(tuple::scan(self.source, &self.held));
    }

    fn records(&self, root: &mut Cst) {
        root.kids.extend(record::scan(self.source, &self.held));
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
            "env" => self.stamp(root, self.held[at].start, self.held[at + 3].end),
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
            self.stamp(root, self.held[at].start, self.held[at].end);
        }
    }

    fn members(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.handed(at) {
                self.stamp(root, self.held[at].start, self.held[at + 2].end);
            }
        }
    }

    fn docs(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.doc(at) {
                let token = &self.held[at];
                root.kids.push(Cst {
                    kind: Kind::Comment,
                    span: Span {
                        start: token.start,
                        end: token.end,
                    },
                    kids: Vec::new(),
                });
            }
        }
    }

    fn doc(&self, at: usize) -> bool {
        let token = &self.held[at];
        let string = matches!(
            token.name.as_str(),
            "TRIPLE2" | "TRIPLE1" | "STRING2" | "STRING1"
        );
        let opens = at == 0 || self.held[at - 1].name == "INDENT";
        let closes = self
            .held
            .get(at + 1)
            .is_some_and(|next| next.name == "NEWLINE");
        string && opens && closes
    }

    fn environs(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.glyph(at) == "os"
                && self.glyph(at + 1) == "."
                && self.glyph(at + 2) == "environ"
            {
                self.stamp(root, self.held[at].start, self.held[at + 2].end);
            }
        }
    }

    fn handed(&self, at: usize) -> bool {
        let head = self.glyph(at);
        matches!(head, "Deno" | "process")
            && self.glyph(at + 1) == "."
            && self.glyph(at + 2) == "env"
    }

    fn stamp(&self, root: &mut Cst, start: usize, end: usize) {
        root.kids.push(Cst {
            kind: Kind::Environment,
            span: Span { start, end },
            kids: Vec::new(),
        });
    }
}
