use std::process::{Command, Output};

const SEALED: &[&str] = &["ban", "coverage", "grant"];

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
    let output = run(&["law"]);
    assert!(output.status.success());
    let stdout = text(output.stdout);
    assert_eq!(stdout.matches(" refuses ").count(), kernel::law::LAWS.len());
    for law in kernel::law::LAWS {
        assert!(stdout.contains(law.name), "{stdout}");
    }
}

#[test]
fn sealed() {
    let output = run(&["law"]);
    let stdout = text(output.stdout);
    assert_eq!(stdout.matches("BOUNDARY: none").count(), SEALED.len());
    for name in SEALED {
        let entry = run(&["law", name]);
        assert!(entry.status.success());
        assert!(text(entry.stdout).contains("BOUNDARY: none"), "{name}");
    }
}

#[test]
fn entries() {
    for law in kernel::law::LAWS {
        let output = run(&["law", law.name]);
        assert!(output.status.success());
        assert!(text(output.stdout).contains(law.note), "{}", law.name);
    }
}

#[test]
fn unknown() {
    let output = run(&["law", "missing"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(output.stderr).contains("unknown law `missing`"));
}
