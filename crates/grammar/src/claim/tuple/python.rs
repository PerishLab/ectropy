use super::Read;

impl Read<'_> {
    pub(super) fn lines(&mut self) {
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
        if self.glyph(from) == "for" {
            self.header(from + 1, to);
            return;
        }
        let mut start = from;
        for at in self.level(from, to) {
            if self.glyph(at) == "=" && self.assign(at) {
                self.side(start, at);
                start = at + 1;
            }
        }
        self.side(start, to);
    }

    fn header(&mut self, from: usize, to: usize) {
        let level = self.level(from, to);
        let Some(&split) = level.iter().find(|&&at| self.glyph(at) == "in") else {
            return;
        };
        let end = level
            .into_iter()
            .find(|&at| at > split && self.glyph(at) == ":")
            .unwrap_or(to);
        self.side(from, split);
        self.side(split + 1, end);
        self.line(end + 1, to);
    }

    fn side(&mut self, from: usize, to: usize) {
        let Some(parts) = self.parts(from, to) else {
            return;
        };
        let Some(span) = self.span(from, to) else {
            return;
        };
        self.emit(span, parts);
    }

    fn level(&self, from: usize, to: usize) -> Vec<usize> {
        let mut depth = 0usize;
        let mut found = Vec::new();
        for at in from..to {
            match self.glyph(at) {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth = depth.saturating_sub(1),
                _ if depth == 0 => found.push(at),
                _ => {}
            }
        }
        found
    }

    fn assign(&self, at: usize) -> bool {
        if self.touches(at, at + 1) && self.glyph(at + 1) == "=" {
            return false;
        }
        if at == 0 || !self.touches(at - 1, at) {
            return true;
        }
        match self.glyph(at - 1) {
            "=" | "!" | ":" => false,
            "<" | ">" => {
                at > 1 && self.touches(at - 2, at - 1) && self.glyph(at - 2) == self.glyph(at - 1)
            }
            _ => true,
        }
    }

    fn touches(&self, left: usize, right: usize) -> bool {
        match (self.held.get(left), self.held.get(right)) {
            (Some(first), Some(second)) => first.end == second.start,
            _ => false,
        }
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
}
