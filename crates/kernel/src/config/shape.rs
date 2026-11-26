use super::walk::{Tracked, Tree, dir, extension};
use super::{Ban, Error, File, Grant};
use plumb::depot::Rules;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const SCAN: &str = "rules/atoms/scan.toml";
const SHAPE: &str = "rules/suites/shape.toml";
const PREFIX: &str = "rule://scan/";

#[derive(Deserialize)]
struct Catalog {
    #[serde(default)]
    set: Vec<Set>,
}

#[derive(Deserialize)]
struct Set {
    name: String,
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    roots: Vec<String>,
    #[serde(default)]
    derive: Vec<String>,
    #[serde(default)]
    tests: Vec<String>,
    #[serde(default)]
    environment: Option<Vec<String>>,
    #[serde(default)]
    bans: Vec<String>,
}

#[derive(Deserialize)]
struct Suite {
    #[serde(default)]
    shape: Vec<Shape>,
    #[serde(default)]
    web: Vec<Web>,
}

#[derive(Deserialize)]
struct Shape {
    when: Vec<String>,
    #[serde(default, rename = "use")]
    base: Vec<String>,
    #[serde(default)]
    tsx: Vec<String>,
    #[serde(default)]
    js: Vec<String>,
}

#[derive(Deserialize)]
struct Web {
    seat: String,
    #[serde(default, rename = "use")]
    base: Vec<String>,
    #[serde(default)]
    svelte: Vec<String>,
    #[serde(default)]
    tsx: Vec<String>,
    #[serde(default)]
    js: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Layout {
    seat: Vec<Seat>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Seat {
    path: String,
    scan: Vec<String>,
    kind: Option<String>,
}

struct Context<'a> {
    root: &'a Path,
    sets: BTreeMap<String, Set>,
    tracked: Tracked,
}

#[derive(Default)]
struct Held {
    include: BTreeSet<String>,
    exclude: BTreeSet<String>,
    roots: BTreeSet<String>,
    tests: BTreeSet<String>,
    environment: BTreeSet<String>,
    bans: BTreeSet<String>,
}

pub(super) fn apply(root: &Path, rules: &Rules, file: &mut File) -> Result<(), Error> {
    let context = Context {
        root,
        sets: sets(rules)?,
        tracked: Tracked::list(root),
    };
    let suite = suite(rules)?;
    let mut held = Held::default();
    for row in &suite.shape {
        held.shape(row, &context)?;
    }
    for row in &suite.web {
        held.web(row, &context)?;
    }
    held.layout(root)?;
    held.write(file);
    Ok(())
}

fn sets(rules: &Rules) -> Result<BTreeMap<String, Set>, Error> {
    let text = rules.read(SCAN).map_err(Error::new)?;
    let catalog: Catalog = toml::from_str(&text)
        .map_err(|source| Error::new(format!("cannot parse {SCAN}: {source}")))?;
    let mut map = BTreeMap::new();
    for set in catalog.set {
        if map.insert(set.name.clone(), set).is_some() {
            return Err(Error::new(format!("duplicate scan set in {SCAN}")));
        }
    }
    Ok(map)
}

fn suite(rules: &Rules) -> Result<Suite, Error> {
    let text = rules.read(SHAPE).map_err(Error::new)?;
    toml::from_str(&text).map_err(|source| Error::new(format!("cannot parse {SHAPE}: {source}")))
}

impl Held {
    fn shape(&mut self, row: &Shape, context: &Context<'_>) -> Result<(), Error> {
        if row.when.is_empty() || !row.when.iter().all(|path| dir(&context.root.join(path))) {
            return Ok(());
        }
        self.select(&row.base, context)?;
        if !row.tsx.is_empty() && scripts(context.root, &row.when) {
            self.select(&row.tsx, context)?;
        }
        if !row.js.is_empty()
            && context
                .tracked
                .holds(&row.when, &earning(&row.js, &context.sets)?)
        {
            self.select(&row.js, context)?;
        }
        Ok(())
    }

    fn web(&mut self, row: &Web, context: &Context<'_>) -> Result<(), Error> {
        let base = context.root.join(&row.seat);
        if !dir(&base) {
            return Ok(());
        }
        self.select(&row.base, context)?;
        let svelte = extension(&base, "svelte");
        if svelte {
            self.select(&row.svelte, context)?;
        }
        if !svelte || extension(&base, "tsx") {
            self.select(&row.tsx, context)?;
        }
        let seats = [row.seat.clone()];
        if !row.js.is_empty()
            && context
                .tracked
                .holds(&seats, &earning(&row.js, &context.sets)?)
        {
            self.select(&row.js, context)?;
        }
        Ok(())
    }

    fn select(&mut self, refs: &[String], context: &Context<'_>) -> Result<(), Error> {
        let tree = Tree::new(context.root);
        for name in refs {
            let set = lookup(name, &context.sets)?;
            self.include.extend(set.include.iter().cloned());
            self.exclude.extend(set.exclude.iter().cloned());
            self.roots.extend(set.roots.iter().cloned());
            self.roots
                .extend(tree.derive(&set.derive, &set.roots, &suffixes(set)));
            self.tests.extend(set.tests.iter().cloned());
            let env = set.environment.as_deref().unwrap_or(&set.tests);
            self.environment.extend(env.iter().cloned());
            self.bans.extend(set.bans.iter().cloned());
        }
        Ok(())
    }

    fn layout(&mut self, root: &Path) -> Result<(), Error> {
        let Some(layout) = radicula_config::section::<Layout>(&root.join("plumb.toml"), "layout")
            .map_err(|source| Error::new(source.to_string()))?
        else {
            return Ok(());
        };
        for seat in layout.seat {
            if seat.kind.as_deref() == Some("retired") || seat.scan.is_empty() {
                continue;
            }
            within(&seat.path, &seat.scan)?;
            self.roots.insert(seat.path);
            self.include.extend(seat.scan);
        }
        Ok(())
    }

    fn write(self, file: &mut File) {
        file.scan.include = self.include.into_iter().collect();
        file.scan.all = file.scan.include.is_empty();
        file.scan.exclude = self.exclude.into_iter().collect();
        file.module.roots = self.roots.into_iter().collect();
        if !self.tests.is_empty() {
            file.grant.push(Grant {
                syntax: "test".to_string(),
                paths: self.tests.into_iter().collect(),
            });
        }
        if !self.environment.is_empty() {
            file.grant.push(Grant {
                syntax: "environment".to_string(),
                paths: self.environment.into_iter().collect(),
            });
        }
        if !self.bans.is_empty() {
            file.ban.push(Ban {
                syntax: "style".to_string(),
                paths: self.bans.into_iter().collect(),
            });
        }
    }
}

fn scripts(root: &Path, seats: &[String]) -> bool {
    seats.iter().any(|path| {
        let base = root.join(path);
        extension(&base, "ts") || extension(&base, "tsx")
    })
}

fn lookup<'a>(name: &str, sets: &'a BTreeMap<String, Set>) -> Result<&'a Set, Error> {
    let slug = name
        .strip_prefix(PREFIX)
        .filter(|slug| !slug.is_empty())
        .ok_or_else(|| Error::new(format!("{SHAPE} names invalid scan reference `{name}`")))?;
    sets.get(slug)
        .ok_or_else(|| Error::new(format!("{SHAPE} names unknown scan set `{name}`")))
}

fn earning(refs: &[String], sets: &BTreeMap<String, Set>) -> Result<BTreeSet<String>, Error> {
    let mut found = BTreeSet::new();
    for name in refs {
        found.extend(suffixes(lookup(name, sets)?));
    }
    Ok(found)
}

fn suffixes(set: &Set) -> BTreeSet<String> {
    set.include
        .iter()
        .filter_map(|glob| glob.rsplit_once("*.").map(|(_, tail)| tail.to_string()))
        .collect()
}

fn within(path: &str, globs: &[String]) -> Result<(), Error> {
    if path.is_empty() || path.starts_with('/') || path.contains("//") {
        return Err(Error::new(format!(
            "layout seat `{path}` is not a valid seat path"
        )));
    }
    let prefix = format!("{path}/");
    for glob in globs {
        if !glob.starts_with(&prefix) || glob.split('/').any(|part| part == "..") {
            return Err(Error::new(format!(
                "layout seat `{path}` scan `{glob}` leaves its seat"
            )));
        }
    }
    Ok(())
}
