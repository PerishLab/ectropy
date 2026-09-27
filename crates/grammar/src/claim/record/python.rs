use super::Read;
use crate::Cst;

pub(super) fn scan(read: &Read) -> Vec<Cst> {
    read.python()
}

impl Read<'_> {
    fn python(&self) -> Vec<Cst> {
        let mut found = Vec::new();
        for start in 0..self.held.len() {
            if self.glyph(start) != "class" || !self.named(start + 1) {
                continue;
            }
            let Some(open) = self.indent(start + 2) else {
                continue;
            };
            let close = self.dedent(open + 1);
            let fields = self.pyfields(open + 1, close);
            if !fields.is_empty() {
                found.push(self.record(start, start + 1, close, fields));
            }
        }
        found
    }

    fn indent(&self, mut at: usize) -> Option<usize> {
        let mut colon = false;
        while at < self.held.len() {
            match self.held[at].name.as_str() {
                "INDENT" if colon => return Some(at),
                "NEWLINE" if colon => {}
                _ if self.glyph(at) == ":" => colon = true,
                _ if colon => return None,
                _ => {}
            }
            at += 1;
        }
        None
    }

    fn dedent(&self, from: usize) -> usize {
        let mut depth = 1usize;
        for at in from..self.held.len() {
            match self.held[at].name.as_str() {
                "INDENT" => depth += 1,
                "DEDENT" if depth == 1 => return at,
                "DEDENT" => depth -= 1,
                _ => {}
            }
        }
        self.held.len().saturating_sub(1)
    }

    fn pyfields(&self, from: usize, to: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut depth = 1usize;
        let mut fresh = true;
        for at in from..to {
            match self.held[at].name.as_str() {
                "INDENT" => depth += 1,
                "DEDENT" => depth = depth.saturating_sub(1),
                "NEWLINE" => fresh = depth == 1,
                _ => {
                    self.pyfield(at, depth, fresh, &mut found);
                    fresh = false;
                }
            }
        }
        found
    }

    fn pyfield(&self, at: usize, depth: usize, fresh: bool, found: &mut Vec<usize>) {
        let rooted = fresh && depth == 1;
        let field = self.named(at) && self.glyph(at + 1) == ":";
        if rooted && field {
            found.push(at);
        }
    }
}
