mod fixture;
mod retired;

use fixture::{Env, LIMITS, Mock, bundle, channel, dead, invoke, publish, put};
use plumb::depot::v3;
use std::collections::BTreeMap;

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn floating() {
    let (listener, source) = Mock::open();
    let held = bundle("v0.164.0", LIMITS);
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &source, &held);
    channel(&mut routes, &pointer);
    let server = Mock::start(listener, routes);

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
    let output = invoke(&env, &[root.path().to_str().expect("path")]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), "clean\n");
    let cached = home
        .path()
        .join("depot/generations")
        .join(&pointer.generation)
        .join(v3::LEAF);
    assert!(cached.is_file());

    drop(server);
    let offline = invoke(&env, &[root.path().to_str().expect("path")]);
    assert_eq!(offline.status.code(), Some(2));

    let (listener, bad) = Mock::open();
    let mut broken = BTreeMap::new();
    let pointer = publish(&mut broken, &bad, &held);
    channel(&mut broken, &pointer);
    let path = pointer
        .manifest
        .url
        .strip_prefix(&bad)
        .expect("path")
        .to_string();
    broken.insert(path, b"{}".to_vec());
    let _server = Mock::start(listener, broken);
    let fresh = tempfile::tempdir().expect("fresh");
    let probe = Env {
        home: fresh.path(),
        source: Some(&bad),
        config: None,
        mark: None,
    };
    let tampered = invoke(&probe, &[root.path().to_str().expect("path")]);
    assert_eq!(tampered.status.code(), Some(2));
}

#[test]
fn pinned() {
    let (listener, source) = Mock::open();
    let first = bundle("v0.163.0", LIMITS);
    let second = bundle("v0.164.0", &LIMITS.replace("file = 300", "file = 5"));
    let mut routes = BTreeMap::new();
    let kept = publish(&mut routes, &source, &first);
    let drifted = publish(&mut routes, &source, &second);
    channel(&mut routes, &drifted);
    let server = Mock::start(listener, routes);

    let pin = v3::Pinned::pointer(&kept)
        .expect("pin")
        .encode()
        .expect("encode");
    let home = tempfile::tempdir().expect("home");
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(
        root.path(),
        "crates/cli/src/main.rs",
        "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\nfn e() {}\nfn f() {}\n",
    );

    let env = Env {
        home: home.path(),
        source: Some(&source),
        config: Some(&pin),
        mark: None,
    };
    let arg = root.path().to_str().expect("path");
    let output = invoke(&env, &[arg]);
    assert!(output.status.success(), "{}", text(&output.stderr));
    let generations = home.path().join("depot/generations");
    assert!(generations.join(&kept.generation).join(v3::LEAF).is_file());
    assert!(!generations.join(&drifted.generation).exists());

    drop(server);
    let hit = invoke(&env, &[arg]);
    assert!(hit.status.success(), "{}", text(&hit.stderr));

    let foreign = pin.replace(&source, "https://depot.other.perish.uk");
    let unknown = pin.replacen('{', "{\"extra\":1,", 1);
    let badsha = pin.replace(&kept.manifest.sha256, &"0".repeat(64));
    for bad in ["", "{}", &foreign, &unknown, &badsha] {
        let probe = Env {
            home: home.path(),
            source: Some(&source),
            config: Some(bad),
            mark: None,
        };
        let refused = invoke(&probe, &[arg]);
        assert_eq!(refused.status.code(), Some(2), "expected refusal for {bad}");
    }
}

#[test]
fn mark() {
    let (listener, source) = Mock::open();
    let held = bundle("v0.164.0", LIMITS);
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &source, &held);
    channel(&mut routes, &pointer);
    let _server = Mock::start(listener, routes);

    let home = tempfile::tempdir().expect("home");
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");
    let arg = root.path().to_str().expect("path");

    let ok = Env {
        home: home.path(),
        source: Some(&source),
        config: None,
        mark: Some(&pointer.generation),
    };
    let matched = invoke(&ok, &[arg]);
    assert!(matched.status.success(), "{}", text(&matched.stderr));

    let wrong = "0".repeat(64);
    let upper = pointer.generation.to_ascii_uppercase();
    for bad in [wrong.as_str(), upper.as_str(), "short", ""] {
        let probe = Env {
            home: home.path(),
            source: Some(&source),
            config: None,
            mark: Some(bad),
        };
        let refused = invoke(&probe, &[arg]);
        assert_eq!(refused.status.code(), Some(2), "expected refusal for {bad}");
    }
}

#[test]
fn overlay() {
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

    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");
    put(root.path(), "tools/extra.ts", "const count = 1;\n");
    let gap = invoke(&env, &[root.path().to_str().expect("path")]);
    assert_eq!(gap.status.code(), Some(1));
    assert!(text(&gap.stdout).contains("tools/extra.ts coverage"));

    put(
        root.path(),
        "plumb.toml",
        "[identity]\nns = 'probe'\n[ectropy.scan]\nexclude = ['tools/extra.ts']\n[[ectropy.grant]]\nsyntax = 'embed'\npaths = ['crates/cli/src/main.rs']\n",
    );
    put(
        root.path(),
        "crates/cli/src/main.rs",
        "const TEXT: &str = include_str!(\"main.rs\");\nfn main() { let _ = TEXT; }\n",
    );
    let clean = invoke(&env, &[root.path().to_str().expect("path")]);
    assert!(clean.status.success(), "{}", text(&clean.stderr));
    assert_eq!(text(&clean.stdout), "clean\n");

    put(root.path(), "ectropy.toml", "[limit]\nblock = 4\n");
    let conflict = invoke(&env, &[root.path().to_str().expect("path")]);
    assert_eq!(conflict.status.code(), Some(2));
    assert!(text(&conflict.stderr).contains("both ectropy.toml and [ectropy] in plumb.toml exist"));

    let bad = tempfile::tempdir().expect("bad");
    put(bad.path(), "plumb.toml", "ectropy = 7\n");
    let refused = invoke(&env, &[bad.path().to_str().expect("path")]);
    assert_eq!(refused.status.code(), Some(2));
}

#[test]
fn exempt() {
    let (listener, source) = Mock::open();
    let held = bundle("v0.164.0", &LIMITS.replace("file = 300", "file = 2"));
    let mut routes = BTreeMap::new();
    let pointer = publish(&mut routes, &source, &held);
    channel(&mut routes, &pointer);
    let _server = Mock::start(listener, routes);

    let home = tempfile::tempdir().expect("home");
    let root = tempfile::tempdir().expect("root");
    put(
        root.path(),
        "lib/tool.py",
        "def one():\n    return 1\n\ndef two():\n    return 2\n",
    );
    put(root.path(), "tests/check.py", "def ok():\n    return 1\n");
    let arg = root.path().to_str().expect("path");

    let live = Env {
        home: home.path(),
        source: Some(&source),
        config: None,
        mark: None,
    };
    let output = invoke(&live, &[arg]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("lib/tool.py file length 5 over limit 2"),
        "{stdout}"
    );
    assert!(!stdout.contains("coverage"), "{stdout}");

    let unreachable = dead();
    let offline = Env {
        home: home.path(),
        source: Some(&unreachable),
        config: None,
        mark: None,
    };
    let refused = invoke(&offline, &[arg]);
    assert_eq!(refused.status.code(), Some(2));
}

#[test]
fn offline() {
    let home = tempfile::tempdir().expect("home");
    let unreachable = dead();
    let env = Env {
        home: home.path(),
        source: Some(&unreachable),
        config: None,
        mark: None,
    };
    for args in [&["law"][..], &["cookbook"][..], &["skill", "list"][..]] {
        let output = invoke(&env, args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            text(&output.stderr)
        );
    }
}
