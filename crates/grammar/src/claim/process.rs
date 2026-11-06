use super::Sweep;
use crate::{Cst, Kind, Span};

impl Sweep<'_> {
    pub(super) fn process(&self, root: &mut Cst) {
        let names = self.names();
        for at in 0..self.held.len() {
            if !self.leading(at) || (at > 0 && self.glyph(at - 1) == ":") {
                continue;
            }
            if let Some(end) = self.route(at) {
                self.put(root, at, end);
            }
            if names.iter().any(|name| name == self.glyph(at))
                && let Some(end) = self.bound(at)
            {
                self.put(root, at, end);
            }
        }
    }

    fn names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for at in 0..self.held.len() {
            if self.glyph(at) == "use" {
                self.intake(at + 1, &mut names);
            }
        }
        names
    }

    fn intake(&self, at: usize, names: &mut Vec<String>) {
        let Some(after) = self.module(at) else {
            return;
        };
        if self.glyph(after) != ":" || self.glyph(after + 1) != ":" {
            return;
        }
        let head = after + 2;
        match self.glyph(head) {
            "Command" => self.bind(head, names),
            "{" => self.group(head + 1, names),
            _ => {}
        }
    }

    fn module(&self, at: usize) -> Option<usize> {
        let root = self.glyph(at);
        if !matches!(root, "std" | "tokio") {
            return None;
        }
        let parts = [root, ":", ":", "process"];
        parts
            .iter()
            .enumerate()
            .all(|(offset, part)| self.glyph(at + offset) == *part)
            .then_some(at + parts.len())
    }

    fn group(&self, from: usize, names: &mut Vec<String>) {
        let mut depth = 1;
        let mut at = from;
        let mut fresh = true;
        while at < self.held.len() && depth > 0 {
            let text = self.glyph(at);
            match text {
                "{" => depth += 1,
                "}" => depth -= 1,
                "Command" if depth == 1 && fresh => self.bind(at, names),
                _ => {}
            }
            fresh = text == "{" || (text == "," && depth == 1);
            at += 1;
        }
    }

    fn bind(&self, at: usize, names: &mut Vec<String>) {
        let name = if self.glyph(at + 1) == "as" {
            self.glyph(at + 2)
        } else {
            "Command"
        };
        if !name.is_empty() && name != "_" {
            names.push(name.to_string());
        }
    }

    fn route(&self, at: usize) -> Option<usize> {
        let after = self.module(at)?;
        let parts = [":", ":", "Command", ":", ":"];
        if !parts
            .iter()
            .enumerate()
            .all(|(offset, part)| self.glyph(after + offset) == *part)
        {
            return None;
        }
        let end = after + parts.len();
        called(self.glyph(end)).then_some(end)
    }

    fn bound(&self, at: usize) -> Option<usize> {
        if self.glyph(at + 1) != ":" || self.glyph(at + 2) != ":" {
            return None;
        }
        let end = at + 3;
        called(self.glyph(end)).then_some(end)
    }

    fn leading(&self, at: usize) -> bool {
        at == 0 || self.glyph(at - 1) != "."
    }

    fn put(&self, root: &mut Cst, from: usize, to: usize) {
        root.kids.push(Cst {
            kind: Kind::Process,
            span: Span {
                start: self.held[from].start,
                end: self.held[to].end,
            },
            kids: Vec::new(),
        });
    }
}

fn called(name: &str) -> bool {
    matches!(name, "new" | "spawn" | "output" | "status")
}
