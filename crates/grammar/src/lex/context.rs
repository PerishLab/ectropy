use crate::format::Rule;

const LITERALS: &[&str] = &[
    "STRING",
    "SINGLE",
    "TEMPLATE",
    "THEAD",
    "LINE_CMT",
    "BLOCK_CMT",
    "REGEX",
];

pub(super) fn literal(name: &str) -> bool {
    LITERALS.contains(&name)
}

pub(super) struct Context {
    jsx: bool,
    stack: Vec<Frame>,
}

enum Frame {
    Interp(usize),
    Hole(usize),
    Tree(Tree),
}

struct Tree {
    depth: usize,
    tag: Tag,
    raw: Option<Raw>,
    root: bool,
    peek: bool,
}

#[derive(Clone, Copy)]
enum Tag {
    Off,
    Start,
    Open(Option<Raw>, bool),
    Lone,
    Shut(Option<Raw>),
}

#[derive(Clone, Copy, PartialEq)]
enum Raw {
    Script,
    Style,
}

enum Move {
    Stay,
    Hole,
    Pop,
}

impl Context {
    pub(super) fn code() -> Self {
        Context {
            jsx: false,
            stack: Vec::new(),
        }
    }

    pub(super) fn hole() -> Self {
        Context {
            jsx: false,
            stack: vec![Frame::Hole(0)],
        }
    }

    pub(super) fn closed(&self) -> bool {
        self.stack.is_empty()
    }

    pub(super) fn valued(&self) -> bool {
        match self.stack.last() {
            Some(Frame::Tree(tree)) => tree.root && matches!(tree.tag, Tag::Open(..)),
            _ => false,
        }
    }

    pub(super) fn of(parsers: &[Rule]) -> Self {
        let mut context = Self::code();
        if parsers.iter().any(|rule| rule.name == "jsx") {
            context.jsx = true;
        } else if parsers.iter().any(|rule| rule.name == "fragment") {
            context.stack.push(Frame::Tree(Tree::new(true)));
        }
        context
    }

    pub(super) fn text(&self) -> bool {
        match self.stack.last() {
            Some(Frame::Tree(tree)) => tree.text(),
            _ => false,
        }
    }

    pub(super) fn resume(&self, bytes: &[u8], at: usize) -> Option<(&'static str, usize)> {
        let Some(Frame::Interp(1)) = self.stack.last() else {
            return None;
        };
        if bytes.get(at) != Some(&b'}') {
            return None;
        }
        let mut pos = at + 1;
        while pos < bytes.len() {
            match (bytes[pos], bytes.get(pos + 1)) {
                (b'\\', Some(_)) => pos += 2,
                (b'`', _) => return Some(("TTAIL", pos + 1)),
                (b'$', Some(b'{')) => return Some(("TMID", pos + 2)),
                _ => pos += 1,
            }
        }
        None
    }

    pub(super) fn step(&mut self, name: &str, glyph: &str, open: bool) {
        match name {
            "THEAD" => return self.stack.push(Frame::Interp(1)),
            "TMID" => return,
            "TTAIL" => {
                self.stack.pop();
                return;
            }
            _ => {}
        }
        let Some(Frame::Tree(tree)) = self.stack.last_mut() else {
            return self.plain(glyph, open);
        };
        match tree.step(name, glyph) {
            Move::Stay => {}
            Move::Hole => self.stack.push(Frame::Hole(1)),
            Move::Pop => {
                self.stack.pop();
            }
        }
    }

    fn plain(&mut self, glyph: &str, open: bool) {
        match (glyph, self.stack.last_mut()) {
            ("{", Some(Frame::Interp(depth) | Frame::Hole(depth))) => *depth += 1,
            ("}", Some(Frame::Interp(1) | Frame::Hole(1))) => {
                self.stack.pop();
            }
            ("}", Some(Frame::Interp(depth) | Frame::Hole(depth))) => *depth -= 1,
            ("<", _) if open && self.jsx => self.stack.push(Frame::Tree(Tree::new(false))),
            _ => {}
        }
    }
}

impl Tree {
    fn new(root: bool) -> Self {
        Tree {
            depth: 0,
            tag: Tag::Off,
            raw: None,
            root,
            peek: false,
        }
        .started(root)
    }

    fn started(mut self, root: bool) -> Self {
        if !root {
            self.tag = Tag::Start;
        }
        self
    }

    fn text(&self) -> bool {
        let inside = self.depth > 0 || self.root;
        matches!(self.tag, Tag::Off) && self.raw.is_none() && inside
    }

    fn outer(&self) -> bool {
        self.depth == 0 && !self.root
    }

    fn step(&mut self, name: &str, glyph: &str) -> Move {
        if std::mem::take(&mut self.peek) && glyph == "(" {
            return Move::Pop;
        }
        if self.raw.is_some() {
            self.inside(name, glyph);
            return Move::Stay;
        }
        match self.tag {
            Tag::Off => self.within(glyph),
            Tag::Start => self.start(name, glyph),
            Tag::Open(raw, fresh) => self.open(raw, fresh, glyph),
            Tag::Lone => self.lone(glyph),
            Tag::Shut(_) => self.shut(glyph),
        }
    }

    fn within(&mut self, glyph: &str) -> Move {
        match glyph {
            "<" => self.tag = Tag::Start,
            "{" => return Move::Hole,
            _ => {}
        }
        Move::Stay
    }

    fn start(&mut self, name: &str, glyph: &str) -> Move {
        self.tag = match (name, glyph) {
            (_, "/") => Tag::Shut(None),
            (_, ">") => return self.enter(None, false),
            ("IDENT", _) => Tag::Open(classify(glyph), true),
            _ => Tag::Off,
        };
        self.settle()
    }

    fn open(&mut self, raw: Option<Raw>, fresh: bool, glyph: &str) -> Move {
        if fresh && self.outer() && matches!(glyph, "," | "=" | "extends") {
            return Move::Pop;
        }
        self.tag = Tag::Open(raw, false);
        match glyph {
            "{" => Move::Hole,
            "/" => {
                self.tag = Tag::Lone;
                Move::Stay
            }
            ">" => self.enter(raw, fresh),
            _ => Move::Stay,
        }
    }

    fn enter(&mut self, raw: Option<Raw>, fresh: bool) -> Move {
        self.peek = fresh && self.outer();
        self.depth += 1;
        self.tag = Tag::Off;
        if self.root {
            self.raw = raw;
        }
        Move::Stay
    }

    fn lone(&mut self, glyph: &str) -> Move {
        if glyph != ">" {
            self.tag = Tag::Open(None, false);
            return Move::Stay;
        }
        self.tag = Tag::Off;
        self.settle()
    }

    fn shut(&mut self, glyph: &str) -> Move {
        if glyph != ">" {
            return Move::Stay;
        }
        self.tag = Tag::Off;
        self.depth = self.depth.saturating_sub(1);
        self.settle()
    }

    fn settle(&self) -> Move {
        if matches!(self.tag, Tag::Off) && self.outer() {
            Move::Pop
        } else {
            Move::Stay
        }
    }

    fn inside(&mut self, name: &str, glyph: &str) {
        self.tag = match (self.tag, name, glyph) {
            (_, _, "<") => Tag::Start,
            (Tag::Start, _, "/") => Tag::Shut(None),
            (Tag::Shut(None), "IDENT", _) => Tag::Shut(classify(glyph)),
            (Tag::Shut(raw), _, ">") if raw == self.raw => {
                self.raw = None;
                self.depth = self.depth.saturating_sub(1);
                Tag::Off
            }
            _ => Tag::Off,
        };
    }
}

fn classify(name: &str) -> Option<Raw> {
    match name {
        "script" => Some(Raw::Script),
        "style" => Some(Raw::Style),
        _ => None,
    }
}
