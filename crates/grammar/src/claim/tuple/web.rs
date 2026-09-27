use super::Read;

pub(super) fn typed(read: &Read, open: usize, close: usize) -> bool {
    let prior = open.checked_sub(1).map(|at| read.glyph(at)).unwrap_or("");
    if matches!(prior, "const" | "let" | "var" | "readonly") {
        return true;
    }
    if matches!(read.glyph(close + 1), "=" | ":") {
        return true;
    }
    if prior == ":" && annotation(read, open) {
        return true;
    }
    let mut at = open;
    while at > 0 {
        at -= 1;
        if matches!(read.glyph(at), ";" | "{" | "}") {
            break;
        }
        if read.glyph(at) == "type" {
            return true;
        }
    }
    false
}

fn annotation(read: &Read, open: usize) -> bool {
    let mut at = open.saturating_sub(1);
    while at > 0 {
        at -= 1;
        match read.glyph(at) {
            "const" | "function" | "interface" | "let" | "type" | "var" => return true,
            "{" => return typebrace(read, at),
            ";" | "}" => return false,
            _ => {}
        }
    }
    false
}

fn typebrace(read: &Read, brace: usize) -> bool {
    let mut at = brace;
    while at > 0 {
        at -= 1;
        match read.glyph(at) {
            "interface" | "type" => return true,
            ";" | "{" | "}" => return false,
            _ => {}
        }
    }
    false
}
