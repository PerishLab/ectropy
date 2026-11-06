use super::*;

#[test]
fn rust() {
    let bare = config(&[], 4);
    for source in [
        "fn run() { std::process::Command::new(\"true\"); }",
        "fn run() { let _ = std::process::Command::new(\"true\").output(); }",
        "fn run() { let _ = std::process::Command::new(\"true\").status(); }",
        "fn run() { let _ = std::process::Command::new(\"true\").spawn(); }",
        "fn run() { tokio::process::Command::new(\"true\"); }",
        "fn run() { let _ = tokio::process::Command::new(\"true\").output(); }",
    ] {
        assert_eq!(hits("src/lib.rs", source, &bare), 0, "{source}");
    }
}

#[test]
fn banned() {
    let mut policy = config(&[], 4);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "process".into(),
        paths: vec!["tests/**".into()],
    });
    let source = "fn run() { let _ = std::process::Command::new(\"true\").output(); }";
    assert_eq!(
        count(
            findings("tests/probe.rs", source, &policy)
                .iter()
                .map(|f| f.law.clone())
                .collect(),
            "ban"
        ),
        1
    );
    assert_eq!(
        count(
            findings("src/lib.rs", source, &policy)
                .iter()
                .map(|f| f.law.clone())
                .collect(),
            "ban"
        ),
        0
    );
    assert_eq!(hits("src/lib.rs", source, &policy), 0);
}

#[test]
fn binding() {
    let mut policy = config(&[], 4);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "process".into(),
        paths: vec!["tests/**".into()],
    });
    for source in [
        "use std::process::Command; fn run() { Command::new(\"true\"); }",
        "use std::process::Command as Proc; fn run() { Proc::new(\"true\"); }",
        "use std::process::{Command, Stdio}; fn run() { Command::new(\"true\"); }",
        "use std::process::{Command as Proc}; fn run() { Proc::new(\"true\"); }",
        "use tokio::process::Command; fn run() { Command::new(\"true\"); }",
    ] {
        assert_eq!(
            count(
                findings("tests/probe.rs", source, &policy)
                    .iter()
                    .map(|f| f.law.clone())
                    .collect(),
                "ban"
            ),
            1,
            "{source}"
        );
    }
}

#[test]
fn mock() {
    let mut policy = config(&[], 4);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "process".into(),
        paths: vec!["tests/**".into()],
    });
    for source in [
        "struct Command; impl Command { fn new(_: &str) -> Self { Self } } fn run() { Command::new(\"x\"); }",
        "fn run() { let text = \"std::process::Command::new\"; }",
        "fn run() { other::std::process::Command::new(\"x\"); }",
        "fn run() { crate::process::Command::new(\"x\"); }",
    ] {
        assert_eq!(
            count(
                findings("tests/probe.rs", source, &policy)
                    .iter()
                    .map(|f| f.law.clone())
                    .collect(),
                "ban"
            ),
            0,
            "{source}"
        );
    }
}

#[test]
fn evidence() {
    let mut policy = config(&[], 4);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "process".into(),
        paths: vec!["tests/**".into()],
    });
    let source = "fn run() { std::process::Command::new(\"true\"); }";
    let found = findings("tests/probe.rs", source, &policy);
    let hit = found.iter().find(|finding| finding.law == "ban").unwrap();
    assert_eq!(hit.line, 1);
    assert_eq!(hit.col, 12);
    assert_eq!(hit.note, "process syntax banned in this path");
}

#[test]
fn limits() {
    let mut policy = config(&[], 4);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "process".into(),
        paths: vec!["tests/**".into()],
    });
    for source in [
        "fn run(cmd: &mut std::process::Command) { cmd.output(); }",
        "fn run() { ::std::process::Command::new(\"x\"); }",
        "mod bridge { pub use std::process::Command as Tool; } fn run() { bridge::Tool::new(\"x\"); }",
        "fn run() { type Tool = std::process::Command; Tool::new(\"x\"); }",
    ] {
        assert_eq!(
            count(
                findings("tests/probe.rs", source, &policy)
                    .iter()
                    .map(|f| f.law.clone())
                    .collect(),
                "ban"
            ),
            0,
            "{source}"
        );
    }
}
