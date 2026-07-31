use serde::Deserialize;
use std::fmt;
use std::path::Path;

mod scan;
mod validate;

pub struct Config {
    pub file: File,
}

#[derive(Debug)]
pub struct Error {
    note: String,
}

impl Error {
    pub(super) fn new(note: impl Into<String>) -> Self {
        Self { note: note.into() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.note)
    }
}

impl std::error::Error for Error {}

pub fn load(root: &Path) -> Result<Config, Error> {
    let path = root.join("ectropy.toml");
    let exists = path
        .try_exists()
        .map_err(|source| Error::new(format!("cannot inspect {}: {source}", path.display())))?;
    if !exists {
        return Ok(Config {
            file: File::default(),
        });
    }
    let file = plumb::config::load(&path).map_err(|source| Error::new(source.to_string()))?;
    validate::run(&file)?;
    Ok(Config { file })
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct File {
    pub scan: Scan,
    pub module: Module,
    pub limit: Limit,
    pub comment: Comment,
    pub word: Word,
    pub boundary: Vec<Boundary>,
    pub grant: Vec<Grant>,
    pub ban: Vec<Ban>,
    pub vocabulary: Vocabulary,
}

pub struct Scan {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub all: bool,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Module {
    pub roots: Vec<String>,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limit {
    pub block: usize,
    pub path: usize,
    pub param: usize,
    pub combination: usize,
    pub markup: usize,
    pub file: usize,
    pub fanout: usize,
}

impl Default for Limit {
    fn default() -> Self {
        Self {
            block: 4,
            path: 4,
            param: 4,
            combination: 3,
            markup: 8,
            file: 300,
            fanout: 10,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Comment {
    pub allow: bool,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Word {
    pub single: bool,
}

impl Default for Word {
    fn default() -> Self {
        Self { single: true }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Boundary {
    pub paths: Vec<String>,
    pub allow: Vec<String>,
    pub note: String,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Grant {
    pub syntax: String,
    pub paths: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Ban {
    pub syntax: String,
    pub paths: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Vocabulary {
    pub term: Vec<Term>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    pub name: String,
    pub description: String,
}

impl Config {
    pub fn exempt(&self, path: &str, law: &str) -> bool {
        self.file
            .boundary
            .iter()
            .any(|edge| covers(edge, path, law))
    }

    pub fn registered(&self, name: &str) -> bool {
        self.file
            .vocabulary
            .term
            .iter()
            .any(|entry| entry.name == name)
    }

    pub fn granted(&self, path: &str, syntax: &str) -> bool {
        let mut declared = false;
        for grant in &self.file.grant {
            if grant.syntax != syntax {
                continue;
            }
            declared = true;
            if grant.paths.iter().any(|pat| Glob::matches(pat, path)) {
                return true;
            }
        }
        !declared && !sealed(syntax)
    }

    pub fn banned(&self, path: &str, syntax: &str) -> bool {
        self.file
            .ban
            .iter()
            .any(|ban| ban.syntax == syntax && ban.paths.iter().any(|pat| Glob::matches(pat, path)))
    }
}

fn sealed(syntax: &str) -> bool {
    syntax == "environment"
}

impl Module {
    pub fn depth(&self, path: &str) -> Option<usize> {
        let size = path.split('/').count();
        if self.roots.is_empty() {
            return Some(size.saturating_sub(1));
        }
        self.roots
            .iter()
            .filter_map(|pat| Glob::root(pat, path))
            .max()
            .map(|root| size.saturating_sub(root + 1))
    }

    pub fn owner(&self, path: &str) -> String {
        if self.roots.is_empty() {
            return ".".to_string();
        }
        let parts = Glob::split(path);
        self.roots
            .iter()
            .filter_map(|pat| Glob::root(pat, path))
            .max()
            .map(|end| parts[..end].join("/"))
            .unwrap_or_else(|| ".".to_string())
    }

    pub fn tail<'a>(&self, path: &'a str) -> Option<Vec<&'a str>> {
        let parts = Glob::split(path);
        if self.roots.is_empty() {
            return Some(parts);
        }
        self.roots
            .iter()
            .filter_map(|pat| Glob::root(pat, path))
            .max()
            .map(|root| parts[root.min(parts.len())..].to_vec())
    }
}

pub struct Glob;

impl Glob {
    pub fn matches(pat: &str, path: &str) -> bool {
        let want = Self::split(pat);
        let have = Self::split(path);
        Self::segs(&want, &have)
    }

    pub fn root(pat: &str, path: &str) -> Option<usize> {
        let want = Self::split(pat);
        let have = Self::split(path);
        (0..have.len())
            .rev()
            .find(|end| Self::segs(&want, &have[..*end]))
    }

    fn split(value: &str) -> Vec<&str> {
        value
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .collect()
    }

    fn segs(want: &[&str], have: &[&str]) -> bool {
        if want.is_empty() {
            return have.is_empty();
        }
        if want[0] == "**" {
            return Self::segs(&want[1..], have)
                || (!have.is_empty() && Self::segs(want, &have[1..]));
        }
        !have.is_empty() && Self::part(want[0], have[0]) && Self::segs(&want[1..], &have[1..])
    }

    fn part(pat: &str, text: &str) -> bool {
        match pat.split_once('*') {
            Some((head, tail)) => text.starts_with(head) && text.ends_with(tail),
            None => pat == text,
        }
    }
}

fn covers(edge: &Boundary, path: &str, law: &str) -> bool {
    edge.allow.iter().any(|name| name == law)
        && edge.paths.iter().any(|pat| Glob::matches(pat, path))
}
