use crate::{Cst, Kind, Source, Span};

struct Head {
    level: usize,
    start: usize,
    kids: Vec<Cst>,
}

struct Rows<'a> {
    bytes: &'a [u8],
    stack: Vec<Head>,
}

pub(crate) fn parse(source: &Source) -> Cst {
    let mut rows = Rows {
        bytes: source.text.as_bytes(),
        stack: vec![Head {
            level: 0,
            start: 0,
            kids: Vec::new(),
        }],
    };
    let mut at = 0;
    while at < rows.bytes.len() {
        at = rows.row(at);
    }
    rows.settle(1, source.text.len());
    let root = rows.stack.pop().unwrap();
    Cst {
        kind: Kind::Root,
        span: Span {
            start: 0,
            end: source.text.len(),
        },
        kids: root.kids,
    }
}

impl Rows<'_> {
    fn row(&mut self, at: usize) -> usize {
        let end = self.eol(at);
        let level = self.hashes(at, end);
        if level > 0 {
            self.raise(level, at);
        }
        self.step(end)
    }

    fn raise(&mut self, level: usize, start: usize) {
        self.settle(level, start);
        self.stack.push(Head {
            level,
            start,
            kids: Vec::new(),
        });
    }

    fn settle(&mut self, level: usize, at: usize) {
        while self.deep(level) {
            self.close(at);
        }
    }

    fn deep(&self, level: usize) -> bool {
        self.stack.len() > 1 && self.stack.last().unwrap().level >= level
    }

    fn close(&mut self, at: usize) {
        let done = self.stack.pop().unwrap();
        let node = Cst {
            kind: Kind::Scope,
            span: Span {
                start: done.start,
                end: at,
            },
            kids: done.kids,
        };
        self.stack.last_mut().unwrap().kids.push(node);
    }

    fn hashes(&self, at: usize, end: usize) -> usize {
        let mut run = at;
        while run < end && self.bytes[run] == b'#' {
            run += 1;
        }
        self.marked(run, end, run - at)
    }

    fn marked(&self, run: usize, end: usize, count: usize) -> usize {
        if count > 0 && run < end && self.bytes[run] == b' ' {
            count
        } else {
            0
        }
    }

    fn eol(&self, at: usize) -> usize {
        let mut end = at;
        while end < self.bytes.len() && self.bytes[end] != b'\n' {
            end += 1;
        }
        end
    }

    fn step(&self, end: usize) -> usize {
        if end < self.bytes.len() { end + 1 } else { end }
    }
}
