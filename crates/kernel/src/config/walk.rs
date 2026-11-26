use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(super) struct Tracked(Vec<String>);

impl Tracked {
    pub(super) fn list(root: &Path) -> Self {
        let output = plumb::config::detached("git")
            .arg("-C")
            .arg(root)
            .args(["ls-files", "-z"])
            .output();
        let paths = match output {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_string)
                .collect(),
            _ => Vec::new(),
        };
        Self(paths)
    }

    pub(super) fn holds(&self, seats: &[String], suffixes: &BTreeSet<String>) -> bool {
        self.0.iter().any(|path| {
            let seated = seats
                .iter()
                .any(|seat| path.starts_with(&format!("{seat}/")));
            let suffix = Path::new(path)
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| suffixes.contains(value));
            seated && suffix
        })
    }
}

pub(super) struct Tree<'a> {
    root: &'a Path,
}

impl<'a> Tree<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self { root }
    }

    pub(super) fn derive(
        &self,
        patterns: &[String],
        statics: &[String],
        suffixes: &BTreeSet<String>,
    ) -> BTreeSet<String> {
        patterns
            .iter()
            .flat_map(|pattern| self.expand(pattern))
            .flat_map(|parent| self.children(&parent))
            .filter(|child| !statics.iter().any(|glob| matched(glob, child)))
            .filter(|child| {
                suffixes
                    .iter()
                    .any(|suffix| extension(&self.root.join(child), suffix))
            })
            .collect()
    }

    fn expand(&self, pattern: &str) -> Vec<String> {
        pattern
            .split('/')
            .fold(vec![String::new()], |held, segment| {
                held.iter()
                    .flat_map(|prefix| self.step(prefix, segment))
                    .collect()
            })
    }

    fn step(&self, prefix: &str, segment: &str) -> Vec<String> {
        if segment == "*" {
            return self.children(prefix);
        }
        let path = join(prefix, segment);
        if dir(&self.root.join(&path)) {
            vec![path]
        } else {
            Vec::new()
        }
    }

    fn children(&self, parent: &str) -> Vec<String> {
        let Ok(entries) = fs::read_dir(self.root.join(parent)) else {
            return Vec::new();
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|entry| {
                let path = entry.path();
                !skip(&path) && dir(&path)
            })
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .map(|name| join(parent, &name))
            .collect();
        names.sort();
        names
    }
}

pub(super) fn dir(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .is_ok_and(|meta| !meta.file_type().is_symlink() && meta.file_type().is_dir())
}

pub(super) fn extension(root: &Path, suffix: &str) -> bool {
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if skip(&path) {
            return false;
        }
        let Ok(meta) = fs::symlink_metadata(&path) else {
            return false;
        };
        let kind = meta.file_type();
        if kind.is_symlink() {
            return false;
        }
        if kind.is_dir() {
            extension(&path, suffix)
        } else {
            kind.is_file() && path.extension().and_then(|value| value.to_str()) == Some(suffix)
        }
    })
}

fn skip(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or("");
    matches!(
        name,
        ".git" | "target" | ".task" | "node_modules" | ".local" | "dist"
    )
}

fn join(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    }
}

fn matched(glob: &str, path: &str) -> bool {
    let left: Vec<&str> = glob.split('/').collect();
    let right: Vec<&str> = path.split('/').collect();
    left.len() == right.len()
        && left
            .iter()
            .zip(&right)
            .all(|(want, have)| *want == "*" || want == have)
}
