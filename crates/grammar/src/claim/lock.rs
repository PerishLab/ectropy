use super::Sweep;
use crate::{Cst, Kind, Span};

impl Sweep<'_> {
    pub(super) fn rust(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if !self.origin(at) || (at > 0 && self.glyph(at - 1) == ":") {
                continue;
            }
            let end = self
                .path(at, &["std", ":", ":", "fs", ":", ":", "File", ":", ":"])
                .or_else(|| self.path(at, &["fs2", ":", ":", "FileExt", ":", ":"]));
            if let Some(end) = end.filter(|&end| method(self.glyph(end))) {
                self.lock(root, at, end);
            }
            if let Some(end) = self.path(at, &["libc", ":", ":", "flock"]) {
                self.lock(root, at, end - 1);
            }
        }
    }

    pub(super) fn python(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if !self.origin(at) || self.glyph(at + 1) != "." {
                continue;
            }
            let found = match self.glyph(at) {
                "fcntl" => matches!(self.glyph(at + 2), "flock" | "lockf"),
                "msvcrt" => self.glyph(at + 2) == "locking",
                _ => false,
            };
            if found {
                self.lock(root, at, at + 2);
            }
        }
    }

    fn origin(&self, at: usize) -> bool {
        at == 0 || self.glyph(at - 1) != "."
    }

    fn path(&self, at: usize, parts: &[&str]) -> Option<usize> {
        parts
            .iter()
            .enumerate()
            .all(|(offset, part)| self.glyph(at + offset) == *part)
            .then_some(at + parts.len())
    }

    fn lock(&self, root: &mut Cst, from: usize, to: usize) {
        root.kids.push(Cst {
            kind: Kind::Lock,
            span: Span {
                start: self.held[from].start,
                end: self.held[to].end,
            },
            kids: Vec::new(),
        });
    }
}

fn method(name: &str) -> bool {
    matches!(
        name,
        "lock"
            | "lock_shared"
            | "lock_exclusive"
            | "try_lock"
            | "try_lock_shared"
            | "try_lock_exclusive"
            | "unlock"
    )
}
