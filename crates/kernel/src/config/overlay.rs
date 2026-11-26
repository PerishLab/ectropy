use super::{Ban, Boundary, File, Grant};
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Overlay {
    pub scan: Scan,
    pub module: Module,
    pub limit: Limit,
    pub comment: Comment,
    pub boundary: Vec<Boundary>,
    pub grant: Vec<Grant>,
    pub ban: Vec<Ban>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Scan {
    pub include: Option<Vec<String>>,
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Module {
    pub roots: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Limit {
    pub block: Option<usize>,
    pub path: Option<usize>,
    pub param: Option<usize>,
    pub combination: Option<usize>,
    pub markup: Option<usize>,
    pub file: Option<usize>,
    pub fanout: Option<usize>,
    pub tuple: Option<usize>,
    pub schema: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Comment {
    pub allow: Option<bool>,
}

impl Limit {
    pub(super) fn apply(&self, target: &mut super::Limit) {
        if let Some(value) = self.block {
            target.block = value;
        }
        if let Some(value) = self.path {
            target.path = value;
        }
        if let Some(value) = self.param {
            target.param = value;
        }
        if let Some(value) = self.combination {
            target.combination = value;
        }
        if let Some(value) = self.markup {
            target.markup = value;
        }
        if let Some(value) = self.file {
            target.file = value;
        }
        if let Some(value) = self.fanout {
            target.fanout = value;
        }
        if let Some(value) = self.tuple {
            target.tuple = value;
        }
        if let Some(value) = self.schema {
            target.schema = value;
        }
    }
}

impl Overlay {
    pub(super) fn apply(self, file: &mut File) {
        if let Some(include) = self.scan.include {
            union(&mut file.scan.include, include);
            file.scan.all = false;
        }
        union(&mut file.scan.exclude, self.scan.exclude);
        union(&mut file.module.roots, self.module.roots);
        self.limit.apply(&mut file.limit);
        if let Some(allow) = self.comment.allow {
            file.comment.allow = allow;
        }
        file.boundary.extend(self.boundary);
        file.grant.extend(self.grant);
        file.ban.extend(self.ban);
    }
}

fn union(target: &mut Vec<String>, added: Vec<String>) {
    for item in added {
        if !target.contains(&item) {
            target.push(item);
        }
    }
}
