use super::Read;
use crate::Cst;

pub(super) fn scan(read: &Read) -> Vec<Cst> {
    read.rust()
}

impl Read<'_> {
    fn rust(&self) -> Vec<Cst> {
        let mut found = Vec::new();
        for start in 0..self.held.len() {
            if self.glyph(start) != "struct" || !self.named(start + 1) {
                continue;
            }
            let Some(open) = self.opening(start + 2) else {
                continue;
            };
            let Some(close) = self.mate(open) else {
                continue;
            };
            found.push(self.record(start, start + 1, close, self.fields(open, close)));
        }
        found
    }

    fn opening(&self, mut at: usize) -> Option<usize> {
        while at < self.held.len() {
            match self.glyph(at) {
                "{" => return Some(at),
                "(" | ";" => return None,
                _ => at += 1,
            }
        }
        None
    }

    fn fields(&self, open: usize, close: usize) -> Vec<usize> {
        let mut found = Vec::new();
        let mut at = open + 1;
        while at < close {
            if let Some(end) = self.mate(at)
                && end > at
            {
                at = end + 1;
                continue;
            }
            if self.named(at) && self.glyph(at + 1) == ":" {
                found.push(at);
                at = self.comma(at + 2, close);
                continue;
            }
            at += 1;
        }
        found
    }

    fn comma(&self, mut at: usize, close: usize) -> usize {
        while at < close {
            if let Some(end) = self.mate(at)
                && end > at
            {
                at = end + 1;
                continue;
            }
            if self.glyph(at) == "," {
                return at + 1;
            }
            at += 1;
        }
        at
    }
}
