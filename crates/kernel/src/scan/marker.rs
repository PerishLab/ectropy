use crate::Node;
use grammar::Kind;
use std::ops::Range;

pub(super) fn spans(node: &Node, text: &str) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    walk(node, text, &mut found);
    found
}

fn walk(node: &Node, text: &str, found: &mut Vec<Range<usize>>) {
    for (at, kid) in node.kids.iter().enumerate() {
        if let Some(span) = marked(node, at, text) {
            found.push(span);
        }
        walk(kid, text, found);
    }
}

fn marked(node: &Node, at: usize, text: &str) -> Option<Range<usize>> {
    let sign = &node.kids[at];
    let head = text.get(sign.span.start..sign.span.end)?;
    if sign.kind != Kind::Test || !head.starts_with('#') {
        return None;
    }
    if head.starts_with("#!") {
        return Some(node.span.start..node.span.end);
    }
    let item = node.kids[at + 1..]
        .iter()
        .find(|next| next.kind != Kind::Test)?;
    (item.kind == Kind::Item).then_some(item.span.start..item.span.end)
}
