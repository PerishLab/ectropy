use crate::Source;
use crate::lex::Token;

struct Veil<'a> {
    source: &'a Source,
    held: &'a [Token],
}

pub(super) fn scan(source: &Source, held: &[Token]) -> Vec<Option<usize>> {
    Veil { source, held }.scan()
}

impl Veil<'_> {
    fn scan(&self) -> Vec<Option<usize>> {
        let mut veils = vec![None; self.held.len()];
        if !self.source.path.ends_with(".ts") && !self.source.path.ends_with(".tsx") {
            return veils;
        }
        let mut at = 0;
        while at < self.held.len() {
            if self.text(at) != "/" || !self.begins(at, &veils) {
                at += 1;
                continue;
            }
            let Some(end) = (at + 1..self.held.len())
                .find(|end| self.text(*end) == "/" && !self.escaped(&self.held[*end]))
            else {
                at += 1;
                continue;
            };
            veils[at] = Some(end);
            at = end + 1;
        }
        veils
    }

    fn begins(&self, at: usize, veils: &[Option<usize>]) -> bool {
        if at == 0 {
            return true;
        }
        if veils.contains(&Some(at - 1)) {
            return false;
        }
        !self.ends(at - 1)
    }

    fn ends(&self, at: usize) -> bool {
        let token = &self.held[at];
        if matches!(
            token.name.as_str(),
            "IDENT" | "NUMBER" | "SINGLE" | "STRING" | "TEMPLATE"
        ) {
            return !matches!(
                self.text(at),
                "await"
                    | "case"
                    | "delete"
                    | "do"
                    | "else"
                    | "in"
                    | "instanceof"
                    | "new"
                    | "of"
                    | "return"
                    | "throw"
                    | "typeof"
                    | "void"
                    | "yield"
            );
        }
        matches!(self.text(at), ")" | "]" | "}") || self.postfix(at)
    }

    fn postfix(&self, at: usize) -> bool {
        if at == 0 {
            return false;
        }
        let glyph = self.text(at);
        if !matches!(glyph, "+" | "-") {
            return false;
        }
        self.text(at - 1) == glyph && self.held[at - 1].end == self.held[at].start
    }

    fn escaped(&self, token: &Token) -> bool {
        let bytes = self.source.text.as_bytes();
        let mut at = token.start;
        let mut count = 0;
        while at > 0 && bytes[at - 1] == b'\\' {
            at -= 1;
            count += 1;
        }
        count % 2 == 1
    }

    fn text(&self, at: usize) -> &str {
        self.held
            .get(at)
            .and_then(|token| self.source.text.get(token.start..token.end))
            .unwrap_or("")
    }
}
