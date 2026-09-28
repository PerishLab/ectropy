use crate::config::Config;
use crate::recovery::{SCHEMA, code};
use crate::{Finding, Node};
use grammar::{Kind, Source};
use std::collections::BTreeMap;

pub struct Record {
    path: String,
    name: String,
    at: usize,
    line: usize,
    col: usize,
    fields: Vec<String>,
}

struct Projection<'a> {
    source: &'a Source,
    records: Vec<Record>,
}

pub fn project(source: &Source, root: &Node) -> Vec<Record> {
    let mut projection = Projection {
        source,
        records: Vec::new(),
    };
    projection.collect(root);
    projection.records
}

pub fn check(records: &[Record], config: &Config) -> Vec<Finding> {
    let mut groups: BTreeMap<Vec<String>, Vec<&Record>> = BTreeMap::new();
    for record in records {
        if record.fields.len() >= 4 && !config.exempt(&record.path, "schema") {
            groups
                .entry(record.fields.clone())
                .or_default()
                .push(record);
        }
    }
    let mut findings = Vec::new();
    for records in groups.values_mut() {
        records.sort_by_key(|record| (&record.path, record.at, &record.name));
        if records.len() > config.file.limit.schema {
            findings.push(finding(records, config.file.limit.schema));
        }
    }
    findings
}

impl Projection<'_> {
    fn collect(&mut self, node: &Node) {
        if node.kind == Kind::Record {
            self.records.push(self.record(node));
        }
        for kid in &node.kids {
            self.collect(kid);
        }
    }

    fn record(&self, node: &Node) -> Record {
        let name = node
            .kids
            .iter()
            .find(|kid| kid.kind == Kind::Label)
            .map(|kid| self.word(kid))
            .unwrap_or_default();
        let mut fields: Vec<String> = node
            .kids
            .iter()
            .filter(|kid| kid.kind == Kind::Field)
            .map(|kid| self.word(kid))
            .collect();
        fields.sort();
        fields.dedup();
        let (line, col) = place(&self.source.text, node.span.start);
        Record {
            path: self.source.path.clone(),
            name,
            at: node.span.start,
            line,
            col,
            fields,
        }
    }

    fn word(&self, node: &Node) -> String {
        self.source
            .text
            .get(node.span.start..node.span.end)
            .unwrap_or("")
            .to_string()
    }
}

fn finding(records: &[&Record], limit: usize) -> Finding {
    let first = records[0];
    let names = records
        .iter()
        .map(|record| record.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let count = records.len();
    let width = first.fields.len();
    let note = format!(
        "{count} records repeat {width} fields over limit {limit}: {names}; the schema needs one canonical model; see: ectropy cookbook {}",
        code(SCHEMA)
    );
    Finding {
        law: "schema".to_string(),
        path: first.path.clone(),
        line: first.line,
        col: first.col,
        note,
    }
}

fn place(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let head = &text[..at];
    let line = head.matches('\n').count() + 1;
    let start = head.rfind('\n').map(|nl| nl + 1).unwrap_or(0);
    (line, at - start + 1)
}
