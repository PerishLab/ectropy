#![allow(dead_code)]

use grammar::Source;

pub fn findings(path: &str, text: &str, config: &kernel::config::Config) -> Vec<kernel::Finding> {
    let source = Source {
        path: path.to_string(),
        text: text.to_string(),
    };
    let node = kernel::structure(&source);
    kernel::check(&source, &node, config)
}

pub fn scan(path: &str, text: &str) -> Vec<String> {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    findings(path, text, &config)
        .iter()
        .map(|f| f.law.clone())
        .collect()
}

pub fn laws(text: &str) -> Vec<String> {
    scan("t.rs", text)
}

pub fn count(laws: Vec<String>, law: &str) -> usize {
    laws.iter().filter(|hit| hit.as_str() == law).count()
}

pub fn config(roots: &[&str], limit: usize) -> kernel::config::Config {
    let mut file = kernel::config::File::default();
    file.limit.path = limit;
    file.module.roots = roots.iter().map(|root| root.to_string()).collect();
    kernel::config::Config { file }
}

pub fn hits(path: &str, text: &str, config: &kernel::config::Config) -> usize {
    count(
        findings(path, text, config)
            .iter()
            .map(|f| f.law.clone())
            .collect(),
        "grant",
    )
}
