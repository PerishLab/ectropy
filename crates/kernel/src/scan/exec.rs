use std::ops::Range;

pub(super) const NOTE: &str = "executable written in-process and made executable in one test function; use plumb::testing::executable or plumb::testing::link";

pub(super) fn find(path: &str, text: &str, marked: &[Range<usize>]) -> Vec<usize> {
    if grammar::Reader::of(path) != Some(grammar::Reader::Rust) {
        return Vec::new();
    }
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
        if tested(path, marked, at) && writes(body) {
            found.push(at);
        }
    }
    found
}

pub(super) fn tested(path: &str, marked: &[Range<usize>], at: usize) -> bool {
    let suite = path.starts_with("tests/") || path.contains("/tests/");
    suite || marked.iter().any(|span| span.contains(&at))
}

fn writes(body: &str) -> bool {
    let write = body.contains("fs::write(") || body.contains("File::create(");
    write && (moded(body) || copied(body))
}

fn moded(body: &str) -> bool {
    ["from_mode(", "set_mode("].iter().any(|call| {
        body.match_indices(call).any(|(at, _)| {
            let digits: Vec<u32> = body[at + call.len()..]
                .trim_start()
                .trim_start_matches("0o")
                .chars()
                .map_while(|glyph| glyph.to_digit(8))
                .collect();
            digits.iter().rev().take(3).any(|digit| digit % 2 == 1)
        })
    })
}

fn copied(body: &str) -> bool {
    body.match_indices("set_permissions(").any(|(at, _)| {
        let call = body[at..].split(';').next().unwrap_or("");
        call.contains("metadata(") && call.contains(".permissions()")
    })
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
