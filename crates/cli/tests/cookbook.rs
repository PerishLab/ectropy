use std::process::{Command, Output};

const ENTRIES: &[&str] = &["burr", "fanout", "shadow"];

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .args(args)
        .output()
        .expect("ectropy")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("utf8")
}

#[test]
fn ledger() {
    let output = run(&["cookbook"]);
    assert!(output.status.success());
    let stdout = text(output.stdout);
    assert_eq!(stdout.matches("EXIT:").count(), ENTRIES.len());
    for entry in ENTRIES {
        assert!(stdout.contains(entry));
    }
}

#[test]
fn entries() {
    for entry in ENTRIES {
        let output = run(&["cookbook", entry]);
        assert!(output.status.success());
        let stdout = text(output.stdout);
        assert!(stdout.contains("## Trigger"));
        assert!(stdout.contains("## Move"));
        assert!(stdout.contains("## Evidence"));
        assert!(stdout.contains("## EXIT"));
    }
}

#[test]
fn unknown() {
    let output = run(&["cookbook", "missing"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(output.stderr).contains("available: burr, fanout, shadow"));
}

#[test]
fn root() {
    let output = run(&["missing-root"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(output.stderr).contains("cannot open missing-root"));
}
