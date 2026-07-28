use crate::Finding;
use crate::config::Config;
use crate::scan::compound;

pub fn depth(path: &str, config: &Config) -> Option<Finding> {
    if config.exempt(path, "path") {
        return None;
    }
    let Some(depth) = config.file.module.depth(path) else {
        return Some(Finding::path(path, "outside declared module roots"));
    };
    if depth > config.file.limit.path {
        return Some(Finding::path(
            path,
            &format!("depth {depth} over limit {}", config.file.limit.path),
        ));
    }
    None
}

pub fn word(path: &str, config: &Config) -> Vec<Finding> {
    if !config.file.word.single || config.exempt(path, "word") {
        return Vec::new();
    }
    let Some(parts) = config.file.module.tail(path) else {
        return Vec::new();
    };
    parts
        .iter()
        .enumerate()
        .flat_map(|(index, part)| atoms(part, index + 1 == parts.len()))
        .filter(|name| compound(&flat(name)) && !config.registered(name))
        .map(|name| Finding::word(path, name))
        .collect()
}

fn atoms(part: &str, file: bool) -> Vec<&str> {
    let mut parts: Vec<&str> = part.split('.').filter(|atom| !atom.is_empty()).collect();
    if file && parts.len() > 1 && source(parts.last().copied().unwrap_or("")) {
        parts.pop();
    }
    parts
}

fn source(extension: &str) -> bool {
    matches!(extension, "md" | "rs" | "scss" | "ts" | "tsx")
}

fn flat(name: &str) -> String {
    name.replace('-', "_")
}

pub fn length(path: &str, lines: usize, config: &Config) -> Option<Finding> {
    if config.exempt(path, "file") {
        return None;
    }
    if lines > config.file.limit.file {
        return Some(Finding::file(
            path,
            &format!("length {lines} over limit {}", config.file.limit.file),
        ));
    }
    None
}

pub fn fanout(paths: &[String], config: &Config) -> Vec<Finding> {
    let mut kids: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for path in paths {
        let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
        for end in 1..parts.len() {
            let seat = parts[..end].join("/");
            kids.entry(seat).or_default().insert(parts[end].to_string());
        }
        if parts.len() > 1 {
            kids.entry(String::new())
                .or_default()
                .insert(parts[0].to_string());
        }
    }
    let mut findings = Vec::new();
    for (seat, names) in kids {
        if seat.is_empty() || config.exempt(&seat, "fanout") {
            continue;
        }
        if names.len() > config.file.limit.fanout {
            findings.push(Finding::fanout(
                &seat,
                &format!(
                    "fanout {} over limit {}; see: ectropy cookbook fanout",
                    names.len(),
                    config.file.limit.fanout
                ),
            ));
        }
    }
    findings
}
