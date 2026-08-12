pub mod config;
pub mod law;
pub mod path;
mod scan;
mod shadow;

use config::Config;
use grammar::{Cst, Kind, Source, Span};

pub struct Node {
    pub kind: Kind,
    pub depth: usize,
    pub span: Span,
    pub kids: Vec<Node>,
}

pub struct Finding {
    pub law: String,
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub note: String,
}

impl Finding {
    fn word(path: &str, name: &str) -> Finding {
        Finding {
            law: "word".to_string(),
            path: path.to_string(),
            line: 0,
            col: 0,
            note: name.to_string(),
        }
    }

    fn file(path: &str, note: &str) -> Finding {
        Finding {
            law: "file".to_string(),
            path: path.to_string(),
            line: 0,
            col: 0,
            note: note.to_string(),
        }
    }

    fn fanout(path: &str, note: &str) -> Finding {
        Finding {
            law: "fanout".to_string(),
            path: path.to_string(),
            line: 0,
            col: 0,
            note: note.to_string(),
        }
    }

    fn path(path: &str, note: &str) -> Finding {
        Finding {
            law: "path".to_string(),
            path: path.to_string(),
            line: 0,
            col: 0,
            note: note.to_string(),
        }
    }
}

pub fn structure(source: &Source) -> Node {
    lift(&grammar::parse(source), 0, 0)
}

fn lift(cst: &Cst, scopes: usize, markups: usize) -> Node {
    let blocks = scopes + usize::from(cst.kind == Kind::Scope);
    let layers = markups + usize::from(cst.kind == Kind::Markup);
    Node {
        kind: cst.kind,
        depth: axis(cst.kind, blocks, layers),
        span: Span {
            start: cst.span.start,
            end: cst.span.end,
        },
        kids: cst
            .kids
            .iter()
            .map(|kid| lift(kid, blocks, layers))
            .collect(),
    }
}

fn axis(kind: Kind, blocks: usize, layers: usize) -> usize {
    if kind == Kind::Markup { layers } else { blocks }
}

pub fn check(source: &Source, node: &Node, config: &Config) -> Vec<Finding> {
    scan::run(source, node, config)
}
