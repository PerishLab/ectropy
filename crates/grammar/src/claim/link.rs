use super::Sweep;
use crate::{Cst, Kind, Span};

const QUOTES: &[&str] = &["SINGLE", "STRING", "TEMPLATE"];

impl Sweep<'_> {
    pub(super) fn wires(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.spelled(at, &["#", "[", "path", "="]) {
                self.link(root, at + 4);
            }
        }
    }

    pub(super) fn specifiers(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            match self.glyph(at) {
                "from" => self.link(root, at + 1),
                "import" if self.glyph(at + 1) == "(" => self.link(root, at + 2),
                "import" => self.link(root, at + 1),
                _ => {}
            }
        }
    }

    fn link(&self, root: &mut Cst, at: usize) {
        let Some(token) = self.held.get(at) else {
            return;
        };
        if !QUOTES.contains(&token.name.as_str()) || self.glyph(at).contains("${") {
            return;
        }
        root.kids.push(Cst {
            kind: Kind::Link,
            span: Span {
                start: token.start + 1,
                end: token.end - 1,
            },
            kids: Vec::new(),
        });
    }
}
