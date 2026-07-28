use std::fmt::Write;

struct Entry {
    name: &'static str,
    body: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        name: "burr",
        body: include_str!("../cookbook/burr.md"),
    },
    Entry {
        name: "fanout",
        body: include_str!("../cookbook/fanout.md"),
    },
    Entry {
        name: "shadow",
        body: include_str!("../cookbook/shadow.md"),
    },
];

pub(crate) fn render(name: Option<&str>) -> Result<String, String> {
    let Some(name) = name else {
        return Ok(ledger());
    };
    ENTRIES
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.body.to_string())
        .ok_or_else(|| format!("unknown cookbook entry `{name}`; available: {}", names()))
}

fn ledger() -> String {
    let mut out = String::new();
    for entry in ENTRIES {
        let _ = writeln!(out, "{}\n  EXIT: {}", entry.name, exit(entry.body));
    }
    out
}

fn exit(body: &str) -> &str {
    body.split_once("## EXIT\n")
        .map(|(_, clause)| clause.trim())
        .unwrap_or("")
}

fn names() -> String {
    ENTRIES
        .iter()
        .map(|entry| entry.name)
        .collect::<Vec<_>>()
        .join(", ")
}
