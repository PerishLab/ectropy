use crate::format::Rule;
use crate::peg::matches;

pub struct Token {
    pub name: String,
    pub kind: Slot,
    pub start: usize,
    pub end: usize,
}

pub enum Slot {
    Emit,
    Comment,
}

pub fn lex(rules: &[Rule], bytes: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        at = once(rules, bytes, at, &mut tokens);
    }
    tokens
}

fn once(rules: &[Rule], bytes: &[u8], at: usize, tokens: &mut Vec<Token>) -> usize {
    match longest(rules, bytes, at) {
        Some((rule, end)) => rule.step(at, end, tokens),
        None => at + 1,
    }
}

fn longest<'a>(rules: &'a [Rule], bytes: &[u8], at: usize) -> Option<(&'a Rule, usize)> {
    let mut best: Option<(&Rule, usize)> = None;
    for rule in rules {
        best = better(best, rule, matches(&rule.pat, bytes, at), at);
    }
    best
}

fn better<'a>(
    best: Option<(&'a Rule, usize)>,
    rule: &'a Rule,
    got: Option<usize>,
    at: usize,
) -> Option<(&'a Rule, usize)> {
    let end = match got {
        Some(end) if end > at => end,
        _ => return best,
    };
    match best {
        Some((_, prior)) if prior >= end => best,
        _ => Some((rule, end)),
    }
}

impl Rule {
    fn step(&self, at: usize, end: usize, tokens: &mut Vec<Token>) -> usize {
        if !self.tagged("skip") {
            tokens.push(self.make(at, end));
        }
        end
    }

    fn make(&self, at: usize, end: usize) -> Token {
        Token {
            name: self.name.clone(),
            kind: self.slot(),
            start: at,
            end,
        }
    }

    fn slot(&self) -> Slot {
        if self.tagged("comment") {
            Slot::Comment
        } else {
            Slot::Emit
        }
    }

    fn tagged(&self, name: &str) -> bool {
        self.tag.as_deref() == Some(name)
    }
}
