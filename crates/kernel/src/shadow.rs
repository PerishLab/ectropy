use std::collections::HashMap;

pub(crate) fn mirrored(text: &str) -> (usize, usize) {
    let Some(open) = text.find('{') else {
        return (0, 0);
    };
    let Some(shut) = text.rfind('}') else {
        return (0, 0);
    };
    if open + 1 >= shut {
        return (0, 0);
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total = 0;
    for element in pieces(&text[open + 1..shut]) {
        if element.trim().is_empty() {
            continue;
        }
        total += 1;
        if let Some(root) = bare(&element) {
            *counts.entry(root).or_insert(0) += 1;
        }
    }
    (counts.into_values().max().unwrap_or(0), total)
}

fn pieces(inner: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut held = String::new();
    let mut depth = 0usize;
    let mut quoted = false;
    for glyph in inner.chars() {
        step(glyph, &mut depth, &mut quoted);
        if glyph == ',' && depth == 0 && !quoted {
            out.push(std::mem::take(&mut held));
            continue;
        }
        held.push(glyph);
    }
    out.push(held);
    out
}

fn step(glyph: char, depth: &mut usize, quoted: &mut bool) {
    if *quoted {
        *quoted = glyph != '"';
        return;
    }
    match glyph {
        '"' => *quoted = true,
        '{' | '(' | '[' => *depth += 1,
        '}' | ')' | ']' => *depth = depth.saturating_sub(1),
        _ => {}
    }
}

fn bare(element: &str) -> Option<String> {
    let mut lot = Vec::new();
    let mut rest = element.trim();
    while !rest.is_empty() {
        let (token, tail) = chunk(rest)?;
        lot.push(token);
        rest = tail.trim_start();
    }
    let [name, colon, root, dot, field] = lot.as_slice() else {
        return None;
    };
    if colon != ":" || dot != "." || name != field {
        return None;
    }
    if !named(name) || !named(root) {
        return None;
    }
    Some(root.clone())
}

fn chunk(rest: &str) -> Option<(String, &str)> {
    let first = rest.chars().next()?;
    if first == ':' || first == '.' {
        return Some((first.to_string(), &rest[first.len_utf8()..]));
    }
    if !named(rest) {
        return None;
    }
    let end = rest
        .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    Some((rest[..end].to_string(), &rest[end..]))
}

fn named(token: &str) -> bool {
    token
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
}
