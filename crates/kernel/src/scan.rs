mod parameter;

use crate::config::Config;
use crate::{Class, Finding, Node};
use grammar::{Kind, Source};

pub(crate) fn run(source: &Source, node: &Node, config: &Config) -> Vec<Finding> {
    let mut scan = Scan {
        source,
        config,
        findings: Vec::new(),
    };
    scan.laws(node);
    scan.receiver(node);
    scan.findings
}

struct Scan<'a> {
    source: &'a Source,
    config: &'a Config,
    findings: Vec<Finding>,
}

impl<'a> Scan<'a> {
    fn laws(&mut self, node: &Node) {
        self.block(node);
        self.markup(node);
        self.comment(node);
        self.coverage(node);
        self.single(node);
        self.grant(node);
        self.dispatch(node);
        self.arity(node);
        self.burr(node);
        self.shadow(node);
        for kid in &node.kids {
            self.laws(kid);
        }
    }

    fn shadow(&mut self, node: &Node) {
        if node.kind != Kind::Literal || self.config.exempt(&self.source.path, "shadow") {
            return;
        }
        let (copied, total) = crate::shadow::mirrored(self.word(node));
        let dense = copied >= 4 && copied * 5 >= total * 4;
        let whole = copied >= 3 && copied == total;
        if dense || whole {
            let note = format!(
                "{copied} of {total} fields copied bare from one root, the twin is redundant; see: ectropy cookbook shadow"
            );
            self.mark(node.span.start, "shadow", &note, Class::Debt);
        }
    }

    fn dispatch(&mut self, node: &Node) {
        if self.config.exempt(&self.source.path, "dispatch") {
            return;
        }
        for at in 2..node.kids.len() {
            if self.table(&node.kids[at - 2..=at]) {
                let name = self.word(&node.kids[at]);
                self.mark(node.kids[at].span.start, "dispatch", name, Class::Debt);
            }
        }
    }

    fn table(&self, run: &[Node]) -> bool {
        run[0].kind == Kind::Probe
            && run[1].kind == Kind::Scope
            && run[2].kind == Kind::Probe
            && self.word(&run[0]) == self.word(&run[2])
            && self.linked(run[1].span.end, run[2].span.start)
    }

    fn linked(&self, from: usize, to: usize) -> bool {
        let gap: String = self
            .source
            .text
            .get(from..to)
            .unwrap_or("")
            .chars()
            .filter(|glyph| !glyph.is_whitespace() && *glyph != '(')
            .collect();
        gap == "if" || gap == "elseif"
    }

    fn grant(&mut self, node: &Node) {
        self.claim(node, Kind::Test, "test");
        self.claim(node, Kind::Style, "style");
        self.claim(node, Kind::Environment, "environment");
    }

    fn claim(&mut self, node: &Node, kind: Kind, syntax: &str) {
        if node.kind != kind {
            return;
        }
        if self.config.banned(&self.source.path, syntax) {
            let note = format!("{syntax} syntax banned in this path");
            self.mark(node.span.start, "ban", &note, Class::Fault);
            return;
        }
        if !self.config.granted(&self.source.path, syntax) {
            let note = match syntax {
                "environment" => "environment syntax outside granted paths, route through the config cascade (plumb docs/config.md)".to_string(),
                _ => format!("{syntax} syntax outside granted paths"),
            };
            self.mark(node.span.start, "grant", &note, Class::Fault);
        }
    }

    fn coverage(&mut self, node: &Node) {
        if node.kind == Kind::Loose && !self.noise(node) {
            self.mark(node.span.start, "coverage", "unparsed region", Class::Blind);
        }
    }

    fn noise(&self, node: &Node) -> bool {
        self.word(node)
            .chars()
            .all(|glyph| glyph.is_whitespace() || matches!(glyph, ';' | ','))
    }

    fn single(&mut self, node: &Node) {
        let name = self.word(node);
        if node.kind == Kind::Word
            && self.config.file.word.single
            && compound(name)
            && !self.config.registered(name)
            && !self.config.exempt(&self.source.path, "word")
        {
            self.mark(node.span.start, "word", name, Class::Debt);
        }
    }

    fn block(&mut self, node: &Node) {
        if node.depth > self.config.file.limit.block
            && node.kind == Kind::Scope
            && !self.config.exempt(&self.source.path, "block")
        {
            self.mark(node.span.start, "block", "depth over limit", Class::Fault);
        }
    }

    fn markup(&mut self, node: &Node) {
        if node.depth > self.config.file.limit.markup
            && node.kind == Kind::Markup
            && !self.config.exempt(&self.source.path, "markup")
        {
            self.mark(node.span.start, "markup", "depth over limit", Class::Fault);
        }
    }

    fn comment(&mut self, node: &Node) {
        if node.kind == Kind::Comment
            && !self.config.file.comment.allow
            && !self.config.exempt(&self.source.path, "comment")
        {
            self.mark(
                node.span.start,
                "comment",
                "denied by default",
                Class::Fault,
            );
        }
    }

    fn word(&self, node: &Node) -> &'a str {
        self.source
            .text
            .get(node.span.start..node.span.end)
            .unwrap_or("")
    }

    fn mark(&mut self, at: usize, law: &str, note: &str, class: Class) {
        let (line, col) = place(&self.source.text, at);
        self.findings.push(Finding {
            law: law.to_string(),
            path: self.source.path.clone(),
            line,
            col,
            note: note.to_string(),
            class,
        });
    }
}

fn place(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let head = &text[..at];
    let line = head.matches('\n').count() + 1;
    let start = head.rfind('\n').map(|nl| nl + 1).unwrap_or(0);
    (line, at - start + 1)
}

pub(crate) fn compound(name: &str) -> bool {
    segments(name) > 1
}

fn segments(name: &str) -> usize {
    let glyphs: Vec<char> = name.chars().collect();
    let mut count = 0;
    let mut prev: Option<char> = None;
    for (at, glyph) in glyphs.iter().copied().enumerate() {
        count += grow(prev, glyph, glyphs.get(at + 1).copied());
        prev = seat(glyph);
    }
    count
}

fn grow(prev: Option<char>, glyph: char, next: Option<char>) -> usize {
    if glyph == '_' {
        return 0;
    }
    match prev {
        None => 1,
        Some(before) => usize::from(
            glyph.is_uppercase()
                && (!before.is_uppercase() || next.is_some_and(char::is_lowercase)),
        ),
    }
}

fn seat(glyph: char) -> Option<char> {
    if glyph == '_' { None } else { Some(glyph) }
}
