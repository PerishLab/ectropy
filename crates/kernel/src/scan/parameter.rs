use super::Scan;
use crate::{Class, Node};
use grammar::Kind;
use std::collections::HashMap;

struct Member {
    key: String,
    at: usize,
    seat: String,
    name: String,
}

impl Scan<'_> {
    pub(super) fn arity(&mut self, item: &Node) {
        if self.config.exempt(&self.source.path, "param") {
            return;
        }
        let count = item
            .kids
            .iter()
            .filter(|kid| matches!(kid.kind, Kind::Receiver | Kind::Param))
            .count();
        if count > self.config.file.limit.param {
            let seat = item
                .kids
                .iter()
                .find(|kid| matches!(kid.kind, Kind::Receiver | Kind::Param))
                .unwrap_or(item);
            let note = format!(
                "{count} parameters over limit {}",
                self.config.file.limit.param
            );
            self.mark(seat.span.start, "param", &note, Class::Debt);
        }
    }

    pub(super) fn burr(&mut self, node: &Node) {
        if !matches!(node.kind, Kind::Receiver | Kind::Param)
            || self.config.exempt(&self.source.path, "burr")
        {
            return;
        }
        let Some((offset, name)) = binding(self.word(node)) else {
            return;
        };
        if !named(name) {
            return;
        }
        let note = format!("{name} is accepted but unused; see: ectropy cookbook burr");
        self.mark(node.span.start + offset, "burr", &note, Class::Debt);
    }

    pub(super) fn receiver(&mut self, root: &Node) {
        if self.config.exempt(&self.source.path, "receiver") {
            return;
        }
        let members: Vec<Member> = root
            .kids
            .iter()
            .filter_map(|item| self.member(item))
            .collect();
        let mut counts: HashMap<String, usize> = HashMap::new();
        for member in &members {
            let count = counts.entry(member.key.clone()).or_insert(0);
            *count += 1;
            if *count > 3 {
                let note = group(member, &members);
                self.mark(member.at, "receiver", &note, Class::Debt);
            }
        }
    }

    fn member(&self, item: &Node) -> Option<Member> {
        let receiver = item.kids.iter().find(|kid| kid.kind == Kind::Receiver)?;
        let name = item.kids.iter().find(|kid| kid.kind == Kind::Word)?;
        let text = self.word(receiver);
        let (_, seat) = binding(text)?;
        Some(Member {
            key: key(text),
            at: receiver.span.start,
            seat: seat.to_string(),
            name: self.word(name).to_string(),
        })
    }
}

fn group(member: &Member, members: &[Member]) -> String {
    let group: Vec<&Member> = members
        .iter()
        .filter(|other| other.key == member.key)
        .collect();
    let names = group
        .iter()
        .map(|other| other.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} functions share {}: {names}", group.len(), member.seat)
}

fn key(text: &str) -> String {
    let mut out = String::new();
    let mut glyphs = text.chars().peekable();
    while let Some(glyph) = glyphs.next() {
        if glyph == '\'' {
            while glyphs
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
            {
                glyphs.next();
            }
            continue;
        }
        if !glyph.is_whitespace() {
            out.push(glyph);
        }
    }
    out
}

fn binding(text: &str) -> Option<(usize, &str)> {
    let (head, _) = text.split_once(':')?;
    let mut name = head.trim();
    if let Some(tail) = name.strip_prefix("mut")
        && tail.chars().next().is_some_and(char::is_whitespace)
    {
        name = tail.trim_start();
    }
    name = name.trim_end_matches('?').trim_end();
    text.find(name).map(|at| (at, name))
}

fn named(name: &str) -> bool {
    name.starts_with('_') && !name.trim_start_matches('_').is_empty()
}
