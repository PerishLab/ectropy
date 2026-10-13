use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Output, Stdio};

const ENABLED: &str = "[locus]\nenabled = true\nendpoint = 'http://127.0.0.1:0'\n";

fn run(home: &Path, root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .current_dir(root)
        .env("ECTROPY_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

fn configure(home: &Path, text: &str) {
    std::fs::write(home.join("config.toml"), text).unwrap();
}

fn records(home: &Path) -> Vec<Value> {
    let mut records = Vec::new();
    for entry in std::fs::read_dir(home.join("state/locus")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name == "active.jsonl" || name.starts_with("sealed-") {
            let content = std::fs::read_to_string(path).unwrap();
            records.extend(
                content
                    .lines()
                    .map(|line| serde_json::from_str::<Value>(line).unwrap()),
            );
        }
    }
    records
}

#[test]
fn disabled() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let absent = run(home.path(), root.path(), &[]);
    configure(home.path(), "[locus]\nenabled = false\n");
    let disabled = run(home.path(), root.path(), &[]);
    assert_eq!(absent.status.code(), Some(0));
    assert_eq!(absent.status.code(), disabled.status.code());
    assert_eq!(absent.stdout, disabled.stdout);
    assert_eq!(absent.stderr, disabled.stderr);
    assert!(!home.path().join("state/locus").exists());
}

#[test]
fn buffered() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    configure(home.path(), ENABLED);
    let clean = run(home.path(), root.path(), &[]);
    assert_eq!(clean.status.code(), Some(0));
    assert_eq!(clean.stdout, b"clean\n");
    std::fs::write(
        root.path().join("bad.rs"),
        "fn read_file() { let value = \"private-source-content\"; }\n",
    )
    .unwrap();
    let finding = run(home.path(), root.path(), &[]);
    assert_eq!(finding.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&finding.stdout).contains("word read_file"));
    std::fs::write(root.path().join("ectropy.toml"), "[limit\n").unwrap();
    let operational = run(home.path(), root.path(), &[]);
    assert_eq!(operational.status.code(), Some(2));
    let atoms = records(home.path());
    assert!(atoms.iter().all(|atom| atom["producer"] == "ectropy"));
    assert!(
        atoms
            .iter()
            .all(|atom| atom["context"]["ectropy.command"] == "scan")
    );
    for code in [0, 1, 2] {
        assert!(atoms.iter().any(
            |atom| atom["payload"]["event"] == "cli.finish" && atom["payload"]["code"] == code
        ));
    }
    let enters = atoms
        .iter()
        .filter(|atom| atom["source"]["edge"] == "enter")
        .count();
    let returns = atoms
        .iter()
        .filter(|atom| atom["source"]["edge"] == "return")
        .count();
    assert_eq!(enters, returns);
    assert_eq!(enters, 10);
    let text = serde_json::to_string(&atoms).unwrap();
    assert!(!text.contains("private-source-content"));
    assert!(!text.contains("read_file"));
    assert!(!text.contains("bad.rs"));
}

#[test]
fn isolated() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let baseline = run(home.path(), root.path(), &[]);
    for config in [
        ENABLED,
        "[locus]\nenabled = 'private-token'\n",
        "[locus]\nenabled = true\nendpoint = ''\n",
    ] {
        configure(home.path(), config);
        let result = run(home.path(), root.path(), &[]);
        assert_eq!(result.status.code(), baseline.status.code());
        assert_eq!(result.stdout, baseline.stdout);
        assert!(!result.stderr.is_empty());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("private-token"));
    }
    let blocked = tempfile::tempdir().unwrap();
    std::fs::write(blocked.path().join("state"), "occupied").unwrap();
    configure(blocked.path(), ENABLED);
    let result = run(blocked.path(), root.path(), &[]);
    assert_eq!(result.status.code(), Some(0));
    assert_eq!(result.stdout, baseline.stdout);
    assert!(!result.stderr.is_empty());
}

#[test]
fn pipe() {
    let home = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    configure(home.path(), ENABLED);
    for at in 0..1000 {
        std::fs::write(
            root.path().join(format!("file{at}.rs")),
            format!("fn seat{at}() {{}}\n"),
        )
        .unwrap();
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .current_dir(root.path())
        .env("ECTROPY_HOME", home.path())
        .arg("shape")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    drop(reader);
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(0));
    assert!(!String::from_utf8_lossy(&result.stderr).contains("Broken pipe"));
    let atoms = records(home.path());
    assert!(
        atoms
            .iter()
            .all(|atom| atom["context"]["ectropy.command"] == "shape")
    );
    assert!(
        atoms
            .iter()
            .any(|atom| atom["payload"]["event"] == "cli.finish" && atom["payload"]["code"] == 0)
    );
    let enters = atoms
        .iter()
        .filter(|atom| atom["source"]["edge"] == "enter")
        .count();
    assert_eq!(
        enters,
        atoms
            .iter()
            .filter(|atom| atom["source"]["edge"] == "return")
            .count()
    );
}
