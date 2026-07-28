pub enum Pat {
    Lit(Vec<u8>),
    Class(Vec<Item>, bool),
    Any,
    Name(String),
    Seq(Vec<Pat>),
    Alt(Vec<Pat>),
    Rep(Box<Pat>, Quant),
    Not(Box<Pat>),
}

pub enum Item {
    One(u8),
    Span(u8, u8),
}

pub enum Quant {
    Star,
    Plus,
    Opt,
}

pub fn matches(pat: &Pat, bytes: &[u8], at: usize) -> Option<usize> {
    match pat {
        Pat::Lit(want) => lit(want, bytes, at),
        Pat::Class(items, neg) => class(items, *neg, bytes, at),
        Pat::Any => any(bytes, at),
        Pat::Name(_) => None,
        Pat::Seq(parts) => seq(parts, bytes, at),
        Pat::Alt(parts) => alt(parts, bytes, at),
        Pat::Rep(inner, quant) => inner.rep(quant, bytes, at),
        Pat::Not(inner) => inner.not(bytes, at),
    }
}

fn lit(want: &[u8], bytes: &[u8], at: usize) -> Option<usize> {
    let end = at + want.len();
    if end <= bytes.len() && &bytes[at..end] == want {
        return Some(end);
    }
    None
}

fn class(items: &[Item], neg: bool, bytes: &[u8], at: usize) -> Option<usize> {
    if at >= bytes.len() {
        return None;
    }
    let hit = items.iter().any(|item| holds(item, bytes[at]));
    if hit != neg {
        return Some(at + 1);
    }
    None
}

fn holds(item: &Item, byte: u8) -> bool {
    match item {
        Item::One(one) => *one == byte,
        Item::Span(lo, hi) => *lo <= byte && byte <= *hi,
    }
}

fn any(bytes: &[u8], at: usize) -> Option<usize> {
    if at < bytes.len() { Some(at + 1) } else { None }
}

fn seq(parts: &[Pat], bytes: &[u8], at: usize) -> Option<usize> {
    let mut pos = at;
    for part in parts {
        pos = matches(part, bytes, pos)?;
    }
    Some(pos)
}

fn alt(parts: &[Pat], bytes: &[u8], at: usize) -> Option<usize> {
    for part in parts {
        if let Some(pos) = matches(part, bytes, at) {
            return Some(pos);
        }
    }
    None
}

impl Pat {
    fn rep(&self, quant: &Quant, bytes: &[u8], at: usize) -> Option<usize> {
        match quant {
            Quant::Opt => Some(self.opt(bytes, at)),
            Quant::Star => Some(self.star(bytes, at)),
            Quant::Plus => self.plus(bytes, at),
        }
    }

    fn opt(&self, bytes: &[u8], at: usize) -> usize {
        matches(self, bytes, at).unwrap_or(at)
    }

    fn star(&self, bytes: &[u8], at: usize) -> usize {
        let mut pos = at;
        while let Some(next) = matches(self, bytes, pos) {
            if next == pos {
                break;
            }
            pos = next;
        }
        pos
    }

    fn plus(&self, bytes: &[u8], at: usize) -> Option<usize> {
        let end = self.star(bytes, at);
        if end > at { Some(end) } else { None }
    }

    fn not(&self, bytes: &[u8], at: usize) -> Option<usize> {
        match matches(self, bytes, at) {
            Some(_) => None,
            None => Some(at),
        }
    }
}
