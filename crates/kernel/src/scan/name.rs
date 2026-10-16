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
