use crate::rig::Rig;
use plumb::skill::Kit;
use plumb::skill::command::Command;
use std::path::PathBuf;

pub use plumb::skill::command::Deed;

const DEPOT: &str = "https://depot.ectropy.perish.uk";

#[locus::trace(with = plumb::observation::view())]
pub fn run(deed: Deed) -> i32 {
    let rig = match Rig::resolve(None) {
        Ok(rig) => rig,
        Err(error) => return sour(&error.to_string()),
    };
    if rig.home.is_empty() {
        return sour("no data home; set ECTROPY_HOME");
    }
    let kit = Kit {
        name: "ectropy".to_string(),
        home: plumb::config::home().unwrap_or_else(|| PathBuf::from(".")),
        state: PathBuf::from(&rig.home).join("state").join("skills.json"),
        url: rig.releases,
    };
    let depot = kit.depot(DEPOT, "ectropy", plumb::version!("ECTROPY"));
    Command("ectropy").run(&depot, deed)
}

fn sour(note: &str) -> i32 {
    eprintln!("ectropy skill: {note}");
    1
}
