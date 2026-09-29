use super::Sweep;
use crate::{Cst, Kind, Span};

const MACROS: &[&str] = &["include", "include_bytes", "include_str"];
const MANIFEST: &str = "\"CARGO_MANIFEST_DIR\"";
const METAS: &[&str] = &["dirname", "filename", "glob", "url"];

impl Sweep<'_> {
    pub(super) fn anchors(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if let Some(end) = self.anchor(at) {
                self.embed(root, at, end);
            }
        }
    }

    fn anchor(&self, at: usize) -> Option<usize> {
        let head = self.glyph(at);
        if head == MANIFEST || head == MANIFEST.trim_matches('"') {
            return Some(at);
        }
        let invoked = self.glyph(at + 1) == "!" && self.opens(at + 2);
        (MACROS.contains(&head) && invoked).then_some(at + 1)
    }

    pub(super) fn spelled(&self, at: usize, glyphs: &[&str]) -> bool {
        glyphs
            .iter()
            .enumerate()
            .all(|(step, glyph)| self.glyph(at + step) == *glyph)
    }

    fn opens(&self, at: usize) -> bool {
        matches!(self.glyph(at), "(" | "[" | "{")
    }

    pub(super) fn metas(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            match self.glyph(at) {
                "__dirname" | "__filename" => self.embed(root, at, at),
                _ if self.meta(at) => self.embed(root, at, at + 4),
                _ => {}
            }
        }
    }

    fn meta(&self, at: usize) -> bool {
        self.spelled(at, &["import", ".", "meta", "."]) && METAS.contains(&self.glyph(at + 4))
    }

    pub(super) fn files(&self, root: &mut Cst) {
        for at in 0..self.held.len() {
            if self.glyph(at) == "__file__" {
                self.embed(root, at, at);
            }
        }
    }

    fn embed(&self, root: &mut Cst, first: usize, last: usize) {
        root.kids.push(Cst {
            kind: Kind::Embed,
            span: Span {
                start: self.held[first].start,
                end: self.held[last].end,
            },
            kids: Vec::new(),
        });
    }
}
