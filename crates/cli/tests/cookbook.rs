use serde_json::Value;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .args(args)
        .output()
        .expect("ectropy")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("utf8")
}

fn json(args: &[&str]) -> Value {
    let output = run(args);
    assert!(output.status.success(), "{}", text(output.stderr));
    serde_json::from_slice(&output.stdout).expect("json")
}

#[test]
fn ledger() {
    let output = run(&["cookbook"]);
    assert!(output.status.success());
    let stdout = text(output.stdout);
    assert_eq!(
        stdout.matches("EXIT:").count(),
        kernel::recovery::CODES.len()
    );
    let codes = stdout
        .lines()
        .filter(|line| !line.starts_with("  "))
        .collect::<Vec<_>>();
    assert_eq!(codes, kernel::recovery::CODES);
}

#[test]
fn entries() {
    for code in kernel::recovery::CODES {
        let output = run(&["cookbook", code]);
        assert!(output.status.success());
        let human = text(output.stdout);
        let value = json(&["cookbook", code, "--json"]);
        assert_eq!(value["code"], *code);
        for (heading, field) in [
            ("Trigger", "trigger"),
            ("Solution", "solution"),
            ("Evidence", "evidence"),
            ("EXIT", "exit"),
        ] {
            assert!(human.contains(&format!("## {heading}")), "{human}");
            assert!(
                human.contains(value[field].as_str().expect(field)),
                "{human}"
            );
        }
    }
}

#[test]
fn ordered() {
    let value = json(&["cookbook", "--json"]);
    let codes = value["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|entry| entry["code"].as_str().expect("code"))
        .collect::<Vec<_>>();
    assert_eq!(codes, kernel::recovery::CODES);
}

#[test]
fn unknown() {
    let output = run(&["cookbook", "burr"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = text(output.stderr);
    assert!(stderr.contains("unknown Cookbook code `burr`"), "{stderr}");
    assert!(
        stderr.contains(&kernel::recovery::CODES.join(", ")),
        "{stderr}"
    );
}

#[test]
fn simple() {
    let output = run(&["missing-root"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = text(output.stderr);
    assert!(stderr.contains("cannot open missing-root"));
    assert!(!stderr.contains("cookbook"));
}
