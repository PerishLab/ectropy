use crate::Error;
use kernel::config::{Config, Glob};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) struct Repo {
    base: PathBuf,
    config: Config,
}

impl Repo {
    pub(crate) fn open(root: &Path) -> Result<Self, Error> {
        let base = fs::canonicalize(root)
            .map_err(|source| Error::note(format!("cannot open {}: {source}", root.display())))?;
        let meta = fs::metadata(&base).map_err(|source| {
            Error::note(format!("cannot inspect {}: {source}", base.display()))
        })?;
        if !meta.is_dir() {
            return Err(Error::note(format!(
                "scan root is not a directory: {}",
                root.display()
            )));
        }
        let config =
            kernel::config::load(&base).map_err(|source| Error::note(source.to_string()))?;
        Ok(Self { base, config })
    }

    pub(crate) fn files(&self) -> Result<Vec<PathBuf>, Error> {
        let mut files = Vec::new();
        self.walk(&self.base, &mut files)?;
        files.sort();
        Ok(files)
    }

    fn walk(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
        let entries = fs::read_dir(dir).map_err(|source| {
            Error::note(format!("cannot read directory {}: {source}", dir.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| {
                Error::note(format!(
                    "cannot read directory entry in {}: {source}",
                    dir.display()
                ))
            })?;
            self.visit(&entry.path(), files)?;
        }
        Ok(())
    }

    fn visit(&self, path: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
        if skip(path) {
            return Ok(());
        }
        let kind = fs::symlink_metadata(path)
            .map_err(|source| Error::note(format!("cannot inspect {}: {source}", path.display())))?
            .file_type();
        if kind.is_symlink() {
            return Ok(());
        }
        if kind.is_dir() {
            return self.walk(path, files);
        }
        if !kind.is_file() {
            return Ok(());
        }
        let rel = self.relative(path)?;
        if self.wanted(&rel) {
            files.push(path.to_path_buf());
        }
        Ok(())
    }

    fn wanted(&self, path: &str) -> bool {
        source(path) && self.within(path) && !self.barred(path)
    }

    fn within(&self, path: &str) -> bool {
        let scan = &self.config.file.scan;
        scan.all || scan.include.iter().any(|pat| Glob::matches(pat, path))
    }

    fn barred(&self, path: &str) -> bool {
        self.config
            .file
            .scan
            .exclude
            .iter()
            .any(|pat| Glob::matches(pat, path))
    }

    pub(crate) fn vocabulary(&self, files: &[PathBuf], out: &mut impl Write) -> Result<(), Error> {
        let mut book: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for path in files {
            self.mine(path, &mut book)?;
        }
        for (root, atoms) in book {
            writeln!(out, "{root}")?;
            for (atom, count) in atoms {
                writeln!(out, "  {atom} {count}")?;
            }
        }
        Ok(())
    }

    fn mine(
        &self,
        path: &Path,
        book: &mut BTreeMap<String, BTreeMap<String, usize>>,
    ) -> Result<(), Error> {
        let rel = self.relative(path)?;
        let root = self.config.file.module.owner(&rel);
        let text = self.read(path)?;
        let source = grammar::Source { path: rel, text };
        let node = kernel::structure(&source);
        glean(&node, &source.text, book.entry(root).or_default());
        Ok(())
    }

    pub(crate) fn shape(&self, files: &[PathBuf], out: &mut impl Write) -> Result<(), Error> {
        for path in files {
            self.draw(path, out)?;
        }
        Ok(())
    }

    fn draw(&self, path: &Path, out: &mut impl Write) -> Result<(), Error> {
        let rel = self.relative(path)?;
        let text = self.read(path)?;
        let source = grammar::Source {
            path: rel.clone(),
            text,
        };
        let value = json!({"path": rel, "tree": tree(&kernel::structure(&source))});
        let encoded = serde_json::to_vec(&value)
            .map_err(|source| Error::note(format!("cannot encode shape JSON: {source}")))?;
        out.write_all(&encoded)?;
        writeln!(out)?;
        Ok(())
    }

    pub(crate) fn scan(&self, files: &[PathBuf]) -> Result<Vec<kernel::Finding>, Error> {
        let mut findings = Vec::new();
        for path in files {
            self.one(path, &mut findings)?;
        }
        let seats = files
            .iter()
            .map(|path| self.relative(path))
            .collect::<Result<Vec<_>, _>>()?;
        findings.extend(kernel::path::fanout(&seats, &self.config));
        Ok(findings)
    }

    fn one(&self, path: &Path, findings: &mut Vec<kernel::Finding>) -> Result<(), Error> {
        let rel = self.relative(path)?;
        if let Some(hit) = kernel::path::depth(&rel, &self.config) {
            findings.push(hit);
        }
        findings.extend(kernel::path::word(&rel, &self.config));
        let text = self.read(path)?;
        if let Some(hit) = kernel::path::length(&rel, text.lines().count(), &self.config) {
            findings.push(hit);
        }
        let source = grammar::Source { path: rel, text };
        let node = kernel::structure(&source);
        findings.extend(kernel::check(&source, &node, &self.config));
        Ok(())
    }

    fn relative(&self, path: &Path) -> Result<String, Error> {
        let rel = path.strip_prefix(&self.base).map_err(|source| {
            Error::note(format!(
                "cannot make {} relative to {}: {source}",
                path.display(),
                self.base.display()
            ))
        })?;
        rel.to_str()
            .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
            .ok_or_else(|| Error::note(format!("path is not valid UTF-8: {}", rel.display())))
    }

    fn read(&self, path: &Path) -> Result<String, Error> {
        fs::read_to_string(path)
            .map_err(|source| Error::note(format!("cannot read {}: {source}", path.display())))
    }
}

fn tree(node: &kernel::Node) -> Value {
    json!({
        "kind": node.kind.to_string(),
        "depth": node.depth,
        "start": node.span.start,
        "end": node.span.end,
        "kids": node.kids.iter().map(tree).collect::<Vec<_>>()
    })
}

fn glean(node: &kernel::Node, text: &str, atoms: &mut BTreeMap<String, usize>) {
    let span = text.get(node.span.start..node.span.end).unwrap_or("");
    match node.kind {
        grammar::Kind::Word if span != "_" => {
            *atoms.entry(span.to_string()).or_default() += 1;
        }
        grammar::Kind::Receiver | grammar::Kind::Param => {
            let name = span.split(':').next().unwrap_or("").trim();
            let name = name.strip_prefix("mut ").unwrap_or(name);
            *atoms.entry(name.to_string()).or_default() += 1;
        }
        _ => {}
    }
    for kid in &node.kids {
        glean(kid, text, atoms);
    }
}

fn skip(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|part| part.to_str())
        .unwrap_or("");
    matches!(
        name,
        "target" | ".git" | ".task" | "node_modules" | ".local"
    )
}

fn source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "css" | "md" | "rs" | "scss" | "ts" | "tsx"))
}
