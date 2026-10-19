use crate::Node;
use grammar::Kind;

pub(super) const IGNORE: &str = "#[ignore] without a reason; write #[ignore = \"...\"]";
pub(super) const TEMPDIR: &str =
    "temporary directory named only by a literal or in-process counter; use tempfile::tempdir()";

pub(super) fn ignore(node: &Node, word: &str, exempt: bool) -> Option<usize> {
    if node.kind != Kind::Test || exempt {
        return None;
    }
    let compact: String = word
        .chars()
        .filter(|glyph| !glyph.is_whitespace())
        .collect();
    (compact == "#[ignore]" || compact == "#![ignore]").then_some(node.span.start)
}

pub(super) fn temps(path: &str, source: &str) -> Vec<usize> {
    if grammar::Reader::of(path) != Some(grammar::Reader::Rust) {
        return Vec::new();
    }
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at + 8 <= bytes.len() {
        if &bytes[at..at + 8] != b"temp_dir" {
            at += 1;
            continue;
        }
        let left = at == 0 || !ident(bytes[at - 1]);
        let right = at + 8 == bytes.len() || !ident(bytes[at + 8]);
        if !(left && right) {
            at += 1;
            continue;
        }
        let Some(end) = call(source, at + 8) else {
            at += 1;
            continue;
        };
        if !unique(source, at + 8, end) {
            found.push(at);
        }
        at = end;
    }
    found
}

fn ident(glyph: u8) -> bool {
    glyph.is_ascii_alphanumeric() || glyph == b'_'
}

fn call(source: &str, from: usize) -> Option<usize> {
    let rest = source.get(from..)?;
    let mut chars = rest.char_indices().peekable();
    space(&mut chars)?;
    if chars.next()?.1 != '(' {
        return None;
    }
    space(&mut chars)?;
    if chars.next()?.1 != ')' {
        return None;
    }
    space(&mut chars)?;
    if chars.next()?.1 != '.' {
        return None;
    }
    space(&mut chars)?;
    let start = chars.peek()?.0;
    if !rest[start..].starts_with("join") {
        return None;
    }
    for _ in 0..4 {
        chars.next()?;
    }
    space(&mut chars)?;
    let open = chars.peek()?.0;
    if chars.next()?.1 != '(' {
        return None;
    }
    Some(from + closed(rest, open)?)
}

fn space(chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>) -> Option<()> {
    while matches!(chars.peek().map(|(_, glyph)| glyph), Some(glyph) if glyph.is_whitespace()) {
        chars.next();
    }
    Some(())
}

fn closed(source: &str, open: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if open >= bytes.len() || bytes[open] != b'(' {
        return None;
    }
    let mut depth = 0usize;
    let mut at = open;
    let mut quote = None::<u8>;
    while at < bytes.len() {
        let glyph = bytes[at];
        if let Some(mark) = quote {
            if glyph == 92 {
                at += 2;
                continue;
            }
            if glyph == mark {
                quote = None;
            }
            at += 1;
            continue;
        }
        if glyph == b'"' || glyph == 39 {
            quote = Some(glyph);
            at += 1;
            continue;
        }
        if glyph == b'(' || glyph == b'[' || glyph == b'{' {
            depth += 1;
            at += 1;
            continue;
        }
        if (glyph == b')' || glyph == b']' || glyph == b'}') && depth > 0 {
            depth -= 1;
            at += 1;
            if depth == 0 {
                return Some(at);
            }
            continue;
        }
        at += 1;
    }
    None
}

fn unique(source: &str, from: usize, end: usize) -> bool {
    let slice = source.get(from..end).unwrap_or("");
    let Some(rel) = slice.find("join(") else {
        return true;
    };
    let body = source
        .get(from + rel + 5..end.saturating_sub(1))
        .unwrap_or("");
    let trimmed = body.trim_start();
    if literal(trimmed) {
        return false;
    }
    if !trimmed.starts_with("format!") {
        return true;
    }
    marked(body) || variable(body)
}

fn variable(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    if lower.contains("fetch_add") || lower.contains("counter") {
        return false;
    }
    let Some(open) = body.find('"') else {
        return false;
    };
    let Some(len) = body[open + 1..].find('"') else {
        return false;
    };
    let template = &body[open + 1..open + 1 + len];
    let args: Vec<&str> = body[open + 2 + len..]
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty() && *arg != ")")
        .collect();
    let bound = |name: &str| {
        args.iter().any(|arg| {
            arg.split_once('=')
                .is_some_and(|(key, value)| key.trim() == name && constant(value))
        })
    };
    let inline = template.split('{').skip(1).any(|piece| {
        let name: String = piece
            .chars()
            .take_while(|glyph| glyph.is_alphanumeric() || *glyph == '_')
            .collect();
        !name.is_empty() && !name.chars().all(|glyph| glyph.is_ascii_digit()) && !bound(&name)
    });
    inline || args.iter().any(|arg| !arg.contains('=') && !constant(arg))
}

fn constant(value: &str) -> bool {
    let value = value.trim().trim_end_matches(')').trim();
    value.starts_with('"')
        || value
            .chars()
            .all(|glyph| glyph.is_ascii_digit() || glyph == '_')
}

fn literal(trimmed: &str) -> bool {
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        return true;
    }
    trimmed.starts_with("r\"") || trimmed.starts_with("r#")
}

fn marked(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    if lower.contains("uuid") || lower.contains("systemtime") {
        return true;
    }
    if lower.contains("instant") || lower.contains("timestamp") {
        return true;
    }
    if lower.contains("utc::now") || lower.contains("local::now") {
        return true;
    }
    lower.contains("process") && lower.contains("id")
}
