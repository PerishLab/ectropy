use crate::lex::{Slot, Token};

const NEWLINE: &str = "NEWLINE";
const INDENT: &str = "INDENT";
const DEDENT: &str = "DEDENT";
const ERROR: &str = "LAYOUT_ERROR";

#[derive(Clone, Copy)]
struct Level {
    column: usize,
    alternate: usize,
}

struct Layout<'a> {
    bytes: &'a [u8],
    levels: Vec<Level>,
    depth: usize,
    prior: Option<usize>,
    tokens: Vec<Token>,
}

pub(crate) fn apply(tokens: Vec<Token>, bytes: &[u8]) -> Vec<Token> {
    let mut layout = Layout {
        bytes,
        levels: vec![Level {
            column: 0,
            alternate: 0,
        }],
        depth: 0,
        prior: None,
        tokens: Vec::new(),
    };
    for token in tokens {
        layout.push(token);
    }
    layout.finish()
}

impl Layout<'_> {
    fn push(&mut self, token: Token) {
        match self.prior {
            Some(prior) => self.separate(prior, token.start),
            None => self.margin(token.start),
        }
        self.depth(&token);
        self.prior = Some(token.end);
        self.tokens.push(token);
    }

    fn separate(&mut self, prior: usize, next: usize) {
        let Some(at) = self.newline(prior, next) else {
            return;
        };
        if self.depth > 0 || self.continued(prior, next) {
            return;
        }
        self.tokens.push(mark(NEWLINE, at, at));
        self.margin(next);
    }

    fn margin(&mut self, at: usize) {
        let start = self.line(at);
        let Some(level) = self.level(start, at) else {
            self.tokens.push(mark(ERROR, start, at));
            return;
        };
        let current = *self.levels.last().unwrap();
        if level.column > current.column {
            self.levels.push(level);
            self.tokens.push(mark(INDENT, start, at));
            return;
        }
        if level.column == current.column {
            if level.alternate != current.alternate {
                self.tokens.push(mark(ERROR, start, at));
            }
            return;
        }
        let Some(found) = self
            .levels
            .iter()
            .rposition(|held| held.column == level.column)
        else {
            self.tokens.push(mark(ERROR, start, at));
            return;
        };
        if self.levels[found].alternate != level.alternate {
            self.tokens.push(mark(ERROR, start, at));
            return;
        }
        while self.levels.len() > found + 1 {
            self.levels.pop();
            self.tokens.push(mark(DEDENT, at, at));
        }
    }

    fn depth(&mut self, token: &Token) {
        let text = self.bytes.get(token.start..token.end).unwrap_or_default();
        if matches!(text, b"(" | b"[" | b"{") {
            self.depth += 1;
        } else if matches!(text, b")" | b"]" | b"}") {
            self.depth = self.depth.saturating_sub(1);
        }
    }

    fn finish(mut self) -> Vec<Token> {
        if let Some(prior) = self.prior {
            let at = self.newline(prior, self.bytes.len()).unwrap_or(prior);
            self.tokens.push(mark(NEWLINE, at, at));
        }
        while self.levels.len() > 1 {
            self.levels.pop();
            self.tokens
                .push(mark(DEDENT, self.bytes.len(), self.bytes.len()));
        }
        self.tokens
    }

    fn newline(&self, from: usize, to: usize) -> Option<usize> {
        self.bytes
            .get(from..to)?
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|at| from + at)
    }

    fn continued(&self, from: usize, to: usize) -> bool {
        let Some(gap) = self.bytes.get(from..to) else {
            return false;
        };
        let mut rows = gap.split(|byte| *byte == b'\n');
        let Some(first) = rows.next() else {
            return false;
        };
        if rows.clone().count() != 1 {
            return false;
        }
        first.strip_suffix(b"\r").unwrap_or(first).ends_with(b"\\")
    }

    fn line(&self, at: usize) -> usize {
        self.bytes
            .get(..at)
            .and_then(|head| head.iter().rposition(|byte| *byte == b'\n'))
            .map(|found| found + 1)
            .unwrap_or(0)
    }

    fn level(&self, from: usize, to: usize) -> Option<Level> {
        let mut column = 0usize;
        let mut alternate = 0usize;
        for byte in self.bytes.get(from..to)? {
            match byte {
                b' ' => {
                    column += 1;
                    alternate += 1;
                }
                b'\t' => {
                    column = (column / 8 + 1) * 8;
                    alternate += 1;
                }
                0x0c => {
                    column = 0;
                    alternate = 0;
                }
                b'\r' => {}
                _ => return None,
            }
        }
        Some(Level { column, alternate })
    }
}

fn mark(name: &str, start: usize, end: usize) -> Token {
    Token {
        name: name.to_string(),
        kind: Slot::Emit,
        start,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn laid(text: &str, words: &[&str]) -> Vec<String> {
        let mut at = 0usize;
        let tokens = words
            .iter()
            .map(|word| {
                let found = text[at..].find(word).unwrap() + at;
                at = found + word.len();
                mark("TOKEN", found, at)
            })
            .collect();
        apply(tokens, text.as_bytes())
            .into_iter()
            .map(|token| token.name)
            .collect()
    }

    #[test]
    fn nested() {
        let text = "root\n    child\n        leaf\n    peer\nend\n";
        let got = laid(text, &["root", "child", "leaf", "peer", "end"]);
        assert_eq!(
            got,
            [
                "TOKEN", "NEWLINE", "INDENT", "TOKEN", "NEWLINE", "INDENT", "TOKEN", "NEWLINE",
                "DEDENT", "TOKEN", "NEWLINE", "DEDENT", "TOKEN", "NEWLINE"
            ]
        );
    }

    #[test]
    fn paired() {
        let text = "call(\n    one,\n    two,\n)\nafter";
        let got = laid(text, &["call", "(", "one", ",", "two", ",", ")", "after"]);
        assert_eq!(
            got,
            [
                "TOKEN", "TOKEN", "TOKEN", "TOKEN", "TOKEN", "TOKEN", "TOKEN", "NEWLINE", "TOKEN",
                "NEWLINE"
            ]
        );
    }

    #[test]
    fn continued() {
        let text = "value = \\\n    first + \\\n    second\nafter";
        let got = laid(text, &["value", "=", "first", "+", "second", "after"]);
        assert_eq!(
            got,
            [
                "TOKEN", "TOKEN", "TOKEN", "TOKEN", "TOKEN", "NEWLINE", "TOKEN", "NEWLINE"
            ]
        );
    }

    #[test]
    fn blank() {
        let text = "root\n    child\n\n    # held\n    peer\nend";
        let got = laid(text, &["root", "child", "peer", "end"]);
        assert_eq!(
            got,
            [
                "TOKEN", "NEWLINE", "INDENT", "TOKEN", "NEWLINE", "TOKEN", "NEWLINE", "DEDENT",
                "TOKEN", "NEWLINE"
            ]
        );
    }

    #[test]
    fn quoted() {
        let text = "text = \"\"\"one\n    two\"\"\"\nafter";
        let got = laid(text, &["text", "=", "\"\"\"one\n    two\"\"\"", "after"]);
        assert_eq!(
            got,
            ["TOKEN", "TOKEN", "TOKEN", "NEWLINE", "TOKEN", "NEWLINE"]
        );
    }

    #[test]
    fn tabs() {
        let text = "root\n\tchild\n\tpeer\nend";
        let got = laid(text, &["root", "child", "peer", "end"]);
        assert!(!got.contains(&ERROR.to_string()));
    }

    #[test]
    fn mixed() {
        let text = "root\n\tchild\n        peer";
        let got = laid(text, &["root", "child", "peer"]);
        assert!(got.contains(&ERROR.to_string()));
    }

    #[test]
    fn uneven() {
        let text = "root\n    child\n  peer";
        let got = laid(text, &["root", "child", "peer"]);
        assert!(got.contains(&ERROR.to_string()));
    }
}
