use super::overlay::{self, Overlay};
use super::{Config, Error, File, shape, validate};
use plumb::depot::Rules;
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

const LIMITS: &str = "rules/atoms/limit.toml";

pub enum Source {
    Legacy(Config),
    Depot(Option<Overlay>),
}

#[derive(Deserialize)]
struct Atom {
    #[serde(default)]
    limit: overlay::Limit,
    #[serde(default)]
    comment: overlay::Comment,
    #[serde(default)]
    word: overlay::Word,
}

pub fn inspect(root: &Path) -> Result<Source, Error> {
    let path = root.join("ectropy.toml");
    let present = exists(&path)?;
    let overlay = radicula_config::section::<Overlay>(&root.join("plumb.toml"), "ectropy")
        .map_err(|source| Error::new(source.to_string()))?;
    match (present, overlay) {
        (true, Some(_)) => Err(Error::new(
            "both ectropy.toml and [ectropy] in plumb.toml exist; keep only one",
        )),
        (true, None) => Ok(Source::Legacy(legacy(&path)?)),
        (false, overlay) => Ok(Source::Depot(overlay)),
    }
}

pub(super) fn load(root: &Path) -> Result<Config, Error> {
    match inspect(root)? {
        Source::Legacy(config) => Ok(config),
        Source::Depot(None) => Ok(Config {
            file: File::default(),
        }),
        Source::Depot(Some(overlay)) => {
            let mut file = File::default();
            overlay.apply(&mut file)?;
            validate::run(&file)?;
            Ok(Config { file })
        }
    }
}

pub fn resolve(root: &Path, rules: &Rules, overlay: Option<Overlay>) -> Result<Config, Error> {
    let mut file = File::default();
    limits(rules, &mut file)?;
    if exists(&root.join("plumb.toml"))? {
        shape::apply(root, rules, &mut file)?;
    }
    if let Some(overlay) = overlay {
        overlay.apply(&mut file)?;
    }
    validate::run(&file)?;
    Ok(Config { file })
}

fn legacy(path: &Path) -> Result<Config, Error> {
    let file = radicula_config::load(path).map_err(|source| Error::new(source.to_string()))?;
    validate::run(&file)?;
    Ok(Config { file })
}

fn limits(rules: &Rules, file: &mut File) -> Result<(), Error> {
    let text = rules.read(LIMITS).map_err(Error::new)?;
    let atom: Atom = toml::from_str(&text)
        .map_err(|source| Error::new(format!("cannot parse {LIMITS}: {source}")))?;
    atom.limit.apply(&mut file.limit);
    if let Some(allow) = atom.comment.allow {
        file.comment.allow = allow;
    }
    if let Some(single) = atom.word.single {
        file.word.single = single;
    }
    Ok(())
}

fn exists(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(source) if source.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(Error::new(format!(
            "cannot inspect {}: {source}",
            path.display()
        ))),
    }
}
