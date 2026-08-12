use kernel::law::{LAWS, Law};
use std::fmt::Write;

pub(crate) fn render(name: Option<&str>) -> Result<String, String> {
    let Some(name) = name else {
        return Ok(ledger());
    };
    kernel::law::find(name)
        .map(entry)
        .ok_or_else(|| format!("unknown law `{name}`; available: {}", names()))
}

fn ledger() -> String {
    let mut out = String::new();
    for law in LAWS {
        let _ = out.write_str(&entry(law));
    }
    out
}

fn entry(law: &Law) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} refuses {}", law.name, law.note);
    if !law.exempt {
        let _ = writeln!(out, "  BOUNDARY: none; this law admits no exemption");
    }
    out
}

fn names() -> String {
    LAWS.iter()
        .map(|law| law.name)
        .collect::<Vec<_>>()
        .join(", ")
}
