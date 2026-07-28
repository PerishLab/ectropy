use super::Scan;
use serde::{Deserialize, Deserializer};

impl Default for Scan {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            all: true,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Raw {
    include: Option<Vec<String>>,
    exclude: Vec<String>,
}

impl<'de> Deserialize<'de> for Scan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = Raw::deserialize(deserializer)?;
        Ok(Self {
            all: raw.include.is_none(),
            include: raw.include.unwrap_or_default(),
            exclude: raw.exclude,
        })
    }
}
