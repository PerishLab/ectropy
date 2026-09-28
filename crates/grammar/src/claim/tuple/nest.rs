use super::{Dialect, Read};

impl Read<'_> {
    pub(super) fn angles(&mut self) {
        if self.dialect == Dialect::Python {
            return;
        }
        for open in 0..self.held.len() {
            let Some(close) = self.generic(open).then(|| self.shut(open)).flatten() else {
                continue;
            };
            self.pairs[open] = Some(close);
            self.pairs[close] = Some(open);
        }
    }

    fn generic(&self, open: usize) -> bool {
        if open == 0 || self.glyph(open) != "<" {
            return false;
        }
        let prior = self.glyph(open - 1);
        let path = prior == ":" && self.glyph(open.saturating_sub(2)) == ":";
        let named = self.held[open - 1].name == "IDENT" && prior.starts_with(char::is_uppercase);
        path || named
    }

    fn shut(&self, open: usize) -> Option<usize> {
        let mut depth = 0usize;
        let mut at = open;
        while at < self.held.len() {
            match self.glyph(at) {
                "<" => depth += 1,
                ">" if !self.arrow(at) => depth -= 1,
                ";" | "{" | "}" | ")" | "]" => return None,
                _ => {}
            }
            if depth == 0 {
                return Some(at);
            }
            at = self.over(at);
        }
        None
    }

    fn arrow(&self, at: usize) -> bool {
        let prior = &self.held[at - 1];
        matches!(self.glyph(at - 1), "-" | "=") && prior.end == self.held[at].start
    }

    fn over(&self, at: usize) -> usize {
        match self.pairs[at] {
            Some(close) if close > at => close + 1,
            _ => at + 1,
        }
    }

    pub(super) fn hop(&self, at: usize, start: usize, to: usize) -> Option<usize> {
        if let Some(close) = self.pairs[at]
            && close > at
            && close < to
        {
            return Some(close + 1);
        }
        let stop = match (self.dialect, self.glyph(at)) {
            (Dialect::Python, "lambda") => ":",
            (Dialect::Rust, "|") if at == start || self.glyph(at - 1) == "move" => "|",
            _ => return None,
        };
        self.tail(at + 1, to, stop)
    }

    fn tail(&self, mut at: usize, to: usize, stop: &str) -> Option<usize> {
        while at < to {
            if self.glyph(at) == stop {
                return Some(at + 1);
            }
            at = self.over(at);
        }
        None
    }
}
