use super::{Error, File};
use std::collections::HashSet;

pub(super) fn run(file: &File) -> Result<(), Error> {
    for pattern in file
        .scan
        .include
        .iter()
        .chain(&file.scan.exclude)
        .chain(&file.module.roots)
        .chain(file.boundary.iter().flat_map(|edge| &edge.paths))
        .chain(file.grant.iter().flat_map(|grant| &grant.paths))
    {
        glob(pattern)?;
    }
    for edge in &file.boundary {
        if edge.paths.is_empty() {
            return Err(Error::new("boundary paths must not be empty"));
        }
        if edge.note.trim().is_empty() {
            return Err(Error::new("boundary note must not be empty"));
        }
        if edge.allow.is_empty() {
            return Err(Error::new("boundary allow must not be empty"));
        }
        for name in &edge.allow {
            if !law(name) {
                return Err(Error::new(format!("unknown boundary law `{name}`")));
            }
        }
    }
    for grant in &file.grant {
        if !matches!(grant.syntax.as_str(), "environment" | "style" | "test") {
            return Err(Error::new(format!(
                "unknown grant syntax `{}`",
                grant.syntax
            )));
        }
        if grant.paths.is_empty() {
            return Err(Error::new(format!(
                "grant `{}` paths must not be empty",
                grant.syntax
            )));
        }
    }
    let mut names = HashSet::new();
    for term in &file.vocabulary.term {
        let name = term.name.trim();
        if name.is_empty() {
            return Err(Error::new("vocabulary term name must not be empty"));
        }
        if term.description.trim().is_empty() {
            return Err(Error::new(format!(
                "vocabulary term `{name}` description must not be empty"
            )));
        }
        if !names.insert(name) {
            return Err(Error::new(format!("duplicate vocabulary term `{name}`")));
        }
    }
    Ok(())
}

fn law(law: &str) -> bool {
    matches!(
        law,
        "block"
            | "burr"
            | "comment"
            | "coverage"
            | "dispatch"
            | "fanout"
            | "file"
            | "grant"
            | "markup"
            | "param"
            | "path"
            | "receiver"
            | "shadow"
            | "word"
    )
}

fn glob(pattern: &str) -> Result<(), Error> {
    if pattern.trim().is_empty() {
        return Err(Error::new("glob must not be empty"));
    }
    if pattern.contains('\\') {
        return Err(Error::new(format!(
            "glob `{pattern}` must use forward slashes"
        )));
    }
    for part in pattern.trim_matches('/').split('/') {
        if part.is_empty() || matches!(part, "." | "..") {
            return Err(Error::new(format!("invalid glob segment in `{pattern}`")));
        }
        if part.contains("**") && part != "**" {
            return Err(Error::new(format!(
                "recursive wildcard must occupy a whole segment in `{pattern}`"
            )));
        }
        if part != "**" && part.matches('*').count() > 1 {
            return Err(Error::new(format!(
                "glob segment has multiple wildcards in `{pattern}`"
            )));
        }
    }
    Ok(())
}
