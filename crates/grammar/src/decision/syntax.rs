use super::Parse;
use crate::Span;

impl Parse<'_> {
    pub(super) fn pair(&mut self) {
        let mut stack = Vec::new();
        let mut at = 0;
        while at < self.held.len() {
            if self.veil(at).is_some() {
                at = self.step(at, self.held.len());
                continue;
            }
            if matches!(self.glyph(at), "(" | "[" | "{") {
                stack.push(at);
                at += 1;
                continue;
            }
            let Some(want) = open(self.glyph(at)) else {
                at += 1;
                continue;
            };
            let Some(slot) = stack.iter().rposition(|held| self.glyph(*held) == want) else {
                at += 1;
                continue;
            };
            let start = stack.remove(slot);
            self.mates[start] = Some(at);
            self.mates[at] = Some(start);
            at += 1;
        }
    }

    pub(super) fn cuts(&self, from: usize, to: usize) -> bool {
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
            if (self.glyph(at) == "{" && self.boundary(at)) || self.hard(at) {
                return true;
            }
            at += 1;
        }
        false
    }

    pub(super) fn hard(&self, at: usize) -> bool {
        match self.glyph(at) {
            ";" | "," | "=>" => true,
            ":" => !self.double(at, ":") && !self.before(at, ":"),
            "?" => !self.double(at, "?") && !self.after(at, "."),
            "=" => self.assign(at),
            _ => false,
        }
    }

    fn assign(&self, at: usize) -> bool {
        if self.comparison(at) {
            return false;
        }
        !self.after(at, "=")
    }

    fn comparison(&self, at: usize) -> bool {
        at > 0
            && matches!(self.glyph(at - 1), "=" | "!" | "<" | ">")
            && self.held[at - 1].end == self.held[at].start
    }

    pub(super) fn double(&self, at: usize, glyph: &str) -> bool {
        self.glyph(at) == glyph && self.after(at, glyph)
    }

    fn before(&self, at: usize, glyph: &str) -> bool {
        at > 0 && self.glyph(at - 1) == glyph && self.held[at - 1].end == self.held[at].start
    }

    fn after(&self, at: usize, glyph: &str) -> bool {
        at + 1 < self.held.len()
            && self.glyph(at + 1) == glyph
            && self.held[at].end == self.held[at + 1].start
    }

    pub(super) fn step(&self, at: usize, to: usize) -> usize {
        if let Some(end) = self.veil(at).filter(|end| *end < to) {
            return end + 1;
        }
        let end = self.mate(at, to);
        if end > at { end + 1 } else { at + 1 }
    }

    pub(super) fn mate(&self, at: usize, to: usize) -> usize {
        self.mates
            .get(at)
            .and_then(|mate| *mate)
            .filter(|mate| *mate < to)
            .unwrap_or(at)
    }

    pub(super) fn glyph(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }

    pub(super) fn veil(&self, at: usize) -> Option<usize> {
        self.veils.get(at).and_then(|end| *end)
    }

    pub(super) fn boundary(&self, at: usize) -> bool {
        self.scopes.contains(&self.held[at].start) && !self.operand(at)
    }

    fn operand(&self, at: usize) -> bool {
        self.leading(at) || self.trailing(self.mate(at, self.held.len()))
    }

    fn leading(&self, at: usize) -> bool {
        let mut before = at;
        while before > 0 && matches!(self.glyph(before - 1), "async" | "move" | "unsafe") {
            before -= 1;
        }
        before >= 2 && (self.double(before - 2, "&") || self.double(before - 2, "|"))
    }

    fn trailing(&self, at: usize) -> bool {
        self.double(at + 1, "&") || self.double(at + 1, "|")
    }

    pub(super) fn code(&self, at: usize) -> bool {
        let Some(mark) = self
            .marks
            .iter()
            .filter(|(start, end)| *start <= at && at < *end)
            .min_by_key(|(start, end)| end - start)
        else {
            return true;
        };
        self.held
            .iter()
            .enumerate()
            .any(|(slot, _)| self.embraces(slot, at, *mark))
    }

    fn embraces(&self, slot: usize, at: usize, mark: (usize, usize)) -> bool {
        let token = &self.held[slot];
        if self.glyph(slot) != "{" || token.start < mark.0 || token.start >= at {
            return false;
        }
        let Some(end) = self.mates.get(slot).and_then(|end| *end) else {
            return false;
        };
        at < self.held[end].end && self.held[end].end <= mark.1
    }

    pub(super) fn span(&self, from: usize, to: usize) -> Span {
        Span {
            start: self.held[from].start,
            end: self.held[to - 1].end,
        }
    }
}

fn open(glyph: &str) -> Option<&str> {
    match glyph {
        ")" => Some("("),
        "]" => Some("["),
        "}" => Some("{"),
        _ => None,
    }
}
