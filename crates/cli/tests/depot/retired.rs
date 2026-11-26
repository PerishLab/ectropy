use crate::fixture::{Env, LIMITS, Mock, bundle, channel, invoke, publish, put};
use std::collections::BTreeMap;

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn overlays() {
    let (listener, source) = Mock::open();
    let held = bundle("v0.164.0", LIMITS);
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &source, &held);
    channel(&mut routes, &pointer);
    let _server = Mock::start(listener, routes);

    let home = tempfile::tempdir().expect("home");
    let env = Env {
        home: home.path(),
        source: Some(&source),
        config: None,
        mark: None,
    };

    for (config, expected) in [
        ("[ectropy.word]\n", "unknown field"),
        ("[ectropy.word]\nsingle = false\n", "unknown field"),
        ("[ectropy.word]\nsingle = true\n", "unknown field"),
        ("[ectropy.vocabulary]\n", "unknown field"),
        (
            "[[ectropy.vocabulary.term]]\nname = \"two_words\"\ndescription = \"valid\"\n",
            "unknown field",
        ),
        (
            "[[ectropy.boundary]]\npaths = [\"crates/**\"]\nallow = [\"word\"]\nnote = \"probe\"\n",
            "admits no exemption",
        ),
    ] {
        let root = tempfile::tempdir().expect("root");
        put(root.path(), "plumb.toml", config);
        put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");
        let output = invoke(&env, &[root.path().to_str().expect("path")]);
        assert_eq!(output.status.code(), Some(2), "{config}");
        assert!(!text(&output.stdout).contains("clean"), "{config}");
        let stderr = text(&output.stderr);
        assert!(stderr.contains(expected), "{config}: {stderr}");
    }
}

#[test]
fn limits() {
    let (listener, source) = Mock::open();
    let omitted = LIMITS.replace("[word]\nsingle = true\n", "");
    let held = bundle("v0.164.0", &omitted);
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &source, &held);
    channel(&mut routes, &pointer);
    let _server = Mock::start(listener, routes);

    let home = tempfile::tempdir().expect("home");
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");
    let env = Env {
        home: home.path(),
        source: Some(&source),
        config: None,
        mark: None,
    };
    let ok = invoke(&env, &[root.path().to_str().expect("path")]);
    assert!(ok.status.success(), "{}", text(&ok.stderr));
    assert_eq!(text(&ok.stdout), "clean\n");

    put(
        root.path(),
        "crates/cli/src/main.rs",
        "fn main() {}\nfn bad_name() {}\n",
    );
    let hit = invoke(&env, &[root.path().to_str().expect("path")]);
    assert_eq!(hit.status.code(), Some(1), "{}", text(&hit.stderr));
    assert!(
        text(&hit.stdout).contains("word bad_name"),
        "{}",
        text(&hit.stdout)
    );
    put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");

    let (listener, bad) = Mock::open();
    let disabled = LIMITS.replace("single = true", "single = false");
    let broken = bundle("v0.164.0", &disabled);
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &bad, &broken);
    channel(&mut routes, &pointer);
    let _server = Mock::start(listener, routes);

    let fresh = tempfile::tempdir().expect("fresh");
    let probe = Env {
        home: fresh.path(),
        source: Some(&bad),
        config: None,
        mark: None,
    };
    let refused = invoke(&probe, &[root.path().to_str().expect("path")]);
    assert_eq!(refused.status.code(), Some(2));
    assert!(
        text(&refused.stderr).contains("cannot disable word.single"),
        "{}",
        text(&refused.stderr)
    );
}
