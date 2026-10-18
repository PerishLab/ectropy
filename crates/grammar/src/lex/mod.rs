use crate::format::Rule;
use crate::peg::matches;

mod context;

use context::Context;

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
    Lexer::new(rules, bytes, Context::code()).run()
}

pub fn web(rules: &(Vec<Rule>, Vec<Rule>), bytes: &[u8]) -> Vec<Token> {
    Lexer::new(&rules.0, bytes, Context::of(&rules.1)).run()
}

struct Lexer<'a> {
    rules: &'a [Rule],
    bytes: &'a [u8],
    tokens: Vec<Token>,
    context: Context,
    shut: Option<usize>,
}

impl<'a> Lexer<'a> {
    fn new(rules: &'a [Rule], bytes: &'a [u8], context: Context) -> Self {
        Lexer {
            rules,
            bytes,
            tokens: Vec::new(),
            context,
            shut: None,
        }
    }

    fn run(mut self) -> Vec<Token> {
        let mut at = 0;
        while at < self.bytes.len() {
            at = self.once(at);
        }
        self.tokens
    }

    fn once(&mut self, at: usize) -> usize {
        if self.quoted(at) {
            self.emit("QUOTE", Slot::Emit, at, at + 1);
            return at + 1;
        }
        if let Some((name, end)) = self.context.resume(self.bytes, at) {
            self.emit(name, Slot::Emit, at, end);
            return end;
        }
        let open = opens(&self.tokens, self.bytes);
        let text = self.context.text();
        let Some((rule, end)) = self.longest(at, open, text) else {
            return at + 1;
        };
        if !rule.tagged("skip") {
            self.emit(&rule.name, rule.slot(), at, end);
        }
        end
    }

    fn quoted(&mut self, at: usize) -> bool {
        if self.shut == Some(at) {
            self.shut = None;
            return true;
        }
        let quote = self.bytes[at];
        if !matches!(quote, b'"' | b'\'') || !self.context.valued() {
            return false;
        }
        if self.bytes.get(at + 1) != Some(&b'{') {
            return false;
        }
        let close = self.hole(at + 1);
        self.shut = close.filter(|&end| self.bytes.get(end) == Some(&quote));
        self.shut.is_some()
    }

    fn hole(&self, from: usize) -> Option<usize> {
        let mut sub = Lexer::new(self.rules, self.bytes, Context::hole());
        let mut at = from;
        while at < self.bytes.len() {
            at = sub.once(at);
            if sub.context.closed() {
                return Some(at);
            }
        }
        None
    }

    fn emit(&mut self, name: &str, kind: Slot, start: usize, end: usize) {
        if matches!(kind, Slot::Emit) {
            let open = opens(&self.tokens, self.bytes);
            let glyph = std::str::from_utf8(&self.bytes[start..end]).unwrap_or("");
            self.context.step(name, glyph, open);
        }
        self.tokens.push(Token {
            name: name.to_string(),
            kind,
            start,
            end,
        });
    }

    fn longest(&self, at: usize, open: bool, text: bool) -> Option<(&'a Rule, usize)> {
        let mut best: Option<(&Rule, usize)> = None;
        for rule in self
            .rules
            .iter()
            .filter(|rule| open || !rule.tagged("regex"))
            .filter(|rule| !text || !context::literal(&rule.name))
        {
            best = better(best, rule, matches(&rule.pat, self.bytes, at), at);
        }
        best
    }
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
fn opens(tokens: &[Token], bytes: &[u8]) -> bool {
    let mut held = tokens
        .iter()
        .rev()
        .filter(|token| matches!(token.kind, Slot::Emit));
    let Some(last) = held.next() else {
        return true;
    };
    if postfix(last, held.next(), bytes) {
        return false;
    }
    matches!(
        text(bytes, last),
        "(" | ","
            | "="
            | ":"
            | "["
            | "!"
            | "&"
            | "|"
            | "?"
            | ";"
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "~"
            | "^"
            | "=>"
            | "await"
            | "case"
            | "delete"
            | "do"
            | "else"
            | "in"
            | "instanceof"
            | "new"
            | "of"
            | "return"
            | "throw"
            | "typeof"
            | "void"
            | "yield"
    )
}

fn postfix(last: &Token, prior: Option<&Token>, bytes: &[u8]) -> bool {
    let glyph = text(bytes, last);
    if !matches!(glyph, "+" | "-") {
        return false;
    }
    prior.is_some_and(|prior| text(bytes, prior) == glyph && prior.end == last.start)
}

fn text<'a>(bytes: &'a [u8], token: &Token) -> &'a str {
    bytes
        .get(token.start..token.end)
        .and_then(|slice| std::str::from_utf8(slice).ok())
        .unwrap_or("")
}
