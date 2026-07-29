use plumb::config::Cascade;

const RELEASES: &str = "https://releases.ectropy.perish.uk";

#[derive(Debug, PartialEq, Cascade)]
pub struct Rig {
    pub home: String,
    pub releases: String,
}

impl Default for Rig {
    fn default() -> Self {
        Rig {
            home: plumb::config::data("ectropy")
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
            releases: RELEASES.to_string(),
        }
    }
}
