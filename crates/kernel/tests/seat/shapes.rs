use super::*;

#[test]
fn bare() {
    let text = "#[test]\n#[ignore]\nfn skipped() {}\n";
    assert!(
        laws(text).contains(&"ignore".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn reason() {
    let text = "#[test]\n#[ignore = \"flake on CI\"]\nfn skipped() {}\n";
    assert!(
        !laws(text).contains(&"ignore".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn inner() {
    let text = "#![ignore]\nfn skipped() {}\n";
    assert!(
        laws(text).contains(&"ignore".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn literal() {
    let text = "fn probe() {\n    let root = std::env::temp_dir().join(\"ectropy-x\");\n}\n";
    assert!(
        laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn counter() {
    let text = "fn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{id}\", id = 1));\n}\n";
    assert!(
        laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn process() {
    let text = "fn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{}\", std::process::id()));\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn uuid() {
    let text = "fn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{}\", Uuid::new_v4()));\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn held() {
    let text = "fn probe() {\n    let root = tempfile::tempdir().unwrap();\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn name() {
    let text = "fn probe(name: String) {\n    let root = std::env::temp_dir().join(name);\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn exec() {
    let text = "#[test]\nfn probe() {\n    fs::write(&path, bytes).unwrap();\n    fs::set_permissions(&path, Permissions::from_mode(0o755)).unwrap();\n}\n";
    assert!(laws(text).contains(&"exec".to_string()), "{:?}", laws(text));
}

#[test]
fn support() {
    let text = "fn probe(path: &Path) {\n    let file = File::create(path).unwrap();\n    fs::set_permissions(path, Permissions::from_mode(0o755)).unwrap();\n}\n";
    assert!(
        scan("tests/support.rs", text).contains(&"exec".to_string()),
        "{:?}",
        scan("tests/support.rs", text)
    );
}

#[test]
fn helper() {
    let text = "#[test]\nfn probe() {\n    plumb::testing::executable(&path, \"#!/bin/sh\\n\");\n    fs::write(&data, bytes).unwrap();\n}\n";
    assert!(
        !laws(text).contains(&"exec".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn product() {
    let text = "fn install(path: &Path) {\n    fs::write(path, bytes).unwrap();\n    fs::set_permissions(path, Permissions::from_mode(0o755)).unwrap();\n}\n";
    assert!(
        !scan("src/install.rs", text).contains(&"exec".to_string()),
        "{:?}",
        scan("src/install.rs", text)
    );
}
