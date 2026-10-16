use std::ops::Range;

pub(super) const NOTE: &str = "executable written in-process and made executable in one test function; use plumb::testing::executable or plumb::testing::link";

pub(super) fn find(path: &str, text: &str, marked: &[Range<usize>]) -> Vec<usize> {
    if !path.ends_with(".rs") {
        return Vec::new();
    }
    let suite = path.starts_with("tests/") || path.contains("/tests/");
    let code = mask(text);
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(rel) = code[from..].find("fn ") {
        let at = from + rel;
        from = at + 3;
        if at > 0
            && (code.as_bytes()[at - 1].is_ascii_alphanumeric() || code.as_bytes()[at - 1] == b'_')
        {
            continue;
        }
        let Some(body) = body(&code, at) else {
            continue;
        };
        let tested = suite || marked.iter().any(|span| span.contains(&at));
        if tested && writes(body) {
            found.push(at);
        }
    }
    found
}

fn writes(body: &str) -> bool {
    let write = body.contains("fs::write(") || body.contains("File::create(");
    write && (body.contains("set_permissions(") || body.contains("from_mode("))
}

fn body(code: &str, at: usize) -> Option<&str> {
    let open = at + code[at..].find(['{', ';'])?;
    if code.as_bytes()[open] == b';' {
        return None;
    }
    let mut depth = 0usize;
    for (rel, glyph) in code[open..].char_indices() {
        depth = match glyph {
            '{' => depth + 1,
            '}' => depth - 1,
            _ => continue,
        };
        if depth == 0 {
            return Some(&code[open..open + rel + 1]);
        }
    }
    None
}

fn mask(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut quoted = false;
    let mut escaped = false;
    for glyph in text.chars() {
        let blank = quoted && (escaped || glyph != '"');
        match (quoted, escaped, glyph) {
            (true, false, '\\') => escaped = true,
            (true, true, _) => escaped = false,
            (_, false, '"') => quoted = !quoted,
            _ => {}
        }
        if blank && glyph != '\n' {
            out.extend(std::iter::repeat_n(' ', glyph.len_utf8()));
        } else {
            out.push(glyph);
        }
    }
    out
}
