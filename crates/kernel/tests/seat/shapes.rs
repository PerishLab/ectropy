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
    let text =
        "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(\"ectropy-x\");\n}\n";
    assert!(
        laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn counter() {
    let text = "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{id}\", id = 1));\n}\n";
    assert!(
        laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn process() {
    let text = "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{}\", std::process::id()));\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn uuid() {
    let text = "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(format!(\"ectropy-{}\", Uuid::new_v4()));\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn held() {
    let text = "#[test]\nfn probe() {\n    let root = tempfile::tempdir().unwrap();\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn name() {
    let text =
        "#[test]\nfn probe(name: String) {\n    let root = std::env::temp_dir().join(name);\n}\n";
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

#[test]
fn nonce() {
    let text = "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(format!(\"sidecar-{name}-{nonce}\"));\n}\n";
    assert!(
        !laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn shared() {
    let text = "#[test]\nfn probe() {\n    let root = std::env::temp_dir().join(format!(\"x-{}\", COUNTER.fetch_add(1, SeqCst)));\n}\n";
    assert!(
        laws(text).contains(&"tempdir".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn runtime() {
    let text = "fn log() {\n    let path = std::env::temp_dir().join(\"broker.log\");\n}\n";
    assert!(
        !scan("src/runtime.rs", text).contains(&"tempdir".to_string()),
        "{:?}",
        scan("src/runtime.rs", text)
    );
}

#[test]
fn private() {
    let text = "#[test]\nfn probe() {\n    fs::write(&key, text).unwrap();\n    fs::set_permissions(&key, Permissions::from_mode(0o600)).unwrap();\n}\n";
    assert!(
        !laws(text).contains(&"exec".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn readonly() {
    let text = "#[test]\nfn probe() {\n    fs::write(&path, b\"x\").unwrap();\n    let mut permissions = fs::metadata(&path).unwrap().permissions();\n    permissions.set_readonly(true);\n    fs::set_permissions(&path, permissions).unwrap();\n}\n";
    assert!(
        !laws(text).contains(&"exec".to_string()),
        "{:?}",
        laws(text)
    );
}

#[test]
fn copy() {
    let text = "#[test]\nfn probe() {\n    fs::write(&path, bytes).unwrap();\n    fs::set_permissions(&path, fs::metadata(source).unwrap().permissions()).unwrap();\n}\n";
    assert!(laws(text).contains(&"exec".to_string()), "{:?}", laws(text));
}
