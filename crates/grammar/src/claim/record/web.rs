use super::Read;
use crate::Cst;

pub(super) fn scan(read: &Read) -> Vec<Cst> {
    let mut found = Vec::new();
    for start in 0..read.held.len() {
        let head = read.glyph(start);
        if !matches!(head, "interface" | "type") || !read.named(start + 1) {
            continue;
        }
        let Some(open) = opening(read, start + 2, head == "type") else {
            continue;
        };
        let Some(close) = read.mate(open) else {
            continue;
        };
        found.push(read.record(start, start + 1, close, fields(read, open, close)));
    }
    found
}

fn opening(read: &Read, mut at: usize, alias: bool) -> Option<usize> {
    let mut assigned = !alias;
    while at < read.held.len() {
        match read.glyph(at) {
            "=" => assigned = true,
            "{" if assigned => return Some(at),
            ";" => return None,
            "class" | "const" | "export" | "function" | "interface" | "let" | "type" | "var" => {
                return None;
            }
            _ => {}
        }
        at += 1;
    }
    None
}

fn fields(read: &Read, open: usize, close: usize) -> Vec<usize> {
    let mut found = Vec::new();
    let mut at = open + 1;
    while at < close {
        if let Some(end) = read.mate(at)
            && end > at
        {
            at = end + 1;
            continue;
        }
        let mark = usize::from(read.glyph(at + 1) == "?");
        if read.named(at) && read.glyph(at + 1 + mark) == ":" {
            found.push(at);
        }
        at += 1;
    }
    found
}
