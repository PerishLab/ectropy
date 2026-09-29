use plumb::cookbook::Code;

pub const BURR: &str = "structure.burr";
pub const COMBINATION: &str = "structure.combination";
pub const EMBED: &str = "structure.embed";
pub const FANOUT: &str = "structure.fanout";
pub const RECEIVER: &str = "structure.receiver";
pub const SCHEMA: &str = "structure.schema";
pub const SHADOW: &str = "structure.shadow";
pub const TUPLE: &str = "structure.tuple";

pub const CODES: &[&str] = &[
    BURR,
    COMBINATION,
    EMBED,
    FANOUT,
    RECEIVER,
    SCHEMA,
    SHADOW,
    TUPLE,
];

pub(crate) fn code(raw: &'static str) -> Code {
    assert!(CODES.contains(&raw), "undeclared recovery code");
    Code::new(raw).expect("valid recovery code")
}
