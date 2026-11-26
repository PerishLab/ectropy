use crate::Error;
use plumb::config::Contract;
use plumb::depot::Rules;
use plumb::depot::v3::{Channel, Kind, Pinned};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const DEPOT: &str = "https://depot.plumb.perish.uk";
const CONFIGURATION: &str = "PLUMB_CONFIGURATION";
const MARK: &str = "PLUMB_DEPOT_MARK";
const SOURCE: &str = "PLUMB_RULES_SOURCE";

pub(crate) fn resolve() -> Result<Rules, Error> {
    let mut inherit = vec![CONFIGURATION.to_string(), MARK.to_string()];
    if cfg!(debug_assertions) && plumb::commit!("ECTROPY").is_none() {
        inherit.push(SOURCE.to_string());
    }
    let contract = Contract {
        inherit,
        managed: Vec::new(),
        reject: Vec::new(),
        bind: BTreeMap::new(),
    };
    let env = plumb::config::environment(&contract).map_err(Error::note)?;
    let source = env.get(SOURCE).unwrap_or(DEPOT);
    let home = home()?;
    let rules = match env.get(CONFIGURATION) {
        Some(raw) => pinned(raw, source, &home)?,
        None => floating(source, &home)?,
    };
    if let Some(expected) = env.get(MARK) {
        verify(expected, rules.mark())?;
    }
    Ok(rules)
}

fn home() -> Result<PathBuf, Error> {
    plumb::config::value("PLUMB_HOME")
        .map(PathBuf::from)
        .or_else(|| plumb::config::data("plumb"))
        .ok_or_else(|| Error::note("cannot locate Depot cache: no PLUMB_HOME"))
}

fn pinned(raw: &str, source: &str, home: &Path) -> Result<Rules, Error> {
    let pin = Pinned::parse(raw, source).map_err(Error::note)?;
    let bundle = pin.cached(home, source).map_err(Error::note)?;
    Rules::resolved(&bundle, pin).map_err(Error::note)
}

fn floating(source: &str, home: &Path) -> Result<Rules, Error> {
    let channel = Channel {
        source,
        product: "plumb",
        channel: "stable",
        kind: Kind::Configuration,
    };
    let discovered = channel.cached(home).map_err(Error::note)?;
    let pin = Pinned::pointer(&discovered.pointer).map_err(Error::note)?;
    Rules::resolved(&discovered.bundle, pin).map_err(Error::note)
}

fn verify(expected: &str, actual: &str) -> Result<(), Error> {
    let hex = expected.len() == 64
        && expected
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !hex {
        return Err(Error::note(format!(
            "{MARK} must be 64 lowercase hex characters"
        )));
    }
    if expected != actual {
        return Err(Error::note(format!(
            "{MARK} {expected} disagrees with resolved Depot mark {actual}"
        )));
    }
    Ok(())
}
