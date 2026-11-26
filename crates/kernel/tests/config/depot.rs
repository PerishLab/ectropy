use crate::fixture::{LIMITS, put, rules};
use kernel::config::{Source, inspect, resolve};

#[test]
fn derivation() {
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(root.path(), "crates/cli/src/main.rs", "fn main() {}\n");
    put(
        root.path(),
        "crates/cli/web/view.ts",
        "export const x = 1;\n",
    );
    put(
        root.path(),
        "apps/web/src/lib/components/card.svelte",
        "<p>x</p>\n",
    );
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    assert!(overlay.is_none());
    let config = resolve(root.path(), &rules(LIMITS), overlay).expect("resolve");
    assert!(!config.file.scan.all);
    assert!(config.file.scan.include.contains(&"crates/**/*.rs".into()));
    assert!(config.file.scan.include.contains(&"crates/**/*.ts".into()));
    assert!(
        config
            .file
            .scan
            .include
            .contains(&"apps/**/*.svelte".into())
    );
    assert!(!config.file.scan.include.contains(&"apps/**/*.tsx".into()));
    assert!(config.file.module.roots.contains(&"crates/cli/web".into()));
    assert!(config.granted("crates/cli/src/main.rs", "test"));
    assert!(!config.granted("crates/cli/src/main.rs", "environment"));
    assert!(config.granted("crates/cli/tests/a.rs", "environment"));
    assert!(config.banned("apps/web/src/lib/components/card.svelte", "style"));
}

#[test]
fn fallback() {
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(root.path(), "app/src/main.rs", "fn main() {}\n");
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    let config = resolve(root.path(), &rules(LIMITS), overlay).expect("resolve");
    assert!(config.granted("app/tests/smoke.rs", "test"));
    assert!(config.granted("app/tests/smoke.rs", "environment"));
    assert!(!config.granted("app/src/main.rs", "environment"));
}

#[test]
fn seats() {
    let root = tempfile::tempdir().expect("root");
    put(
        root.path(),
        "plumb.toml",
        "[[layout.seat]]\npath = 'extra'\nscan = ['extra/**/*.py']\n[[layout.seat]]\npath = 'old'\nkind = 'retired'\nscan = ['old/**/*.py']\n",
    );
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    let config = resolve(root.path(), &rules(LIMITS), overlay).expect("resolve");
    assert!(!config.file.scan.all);
    assert_eq!(config.file.scan.include, ["extra/**/*.py"]);
    assert_eq!(config.file.module.roots, ["extra"]);

    put(
        root.path(),
        "plumb.toml",
        "[[layout.seat]]\npath = 'extra'\nscan = ['other/**/*.py']\n",
    );
    let error = resolve(root.path(), &rules(LIMITS), None)
        .err()
        .expect("escape");
    assert!(error.to_string().contains("leaves its seat"), "{error}");

    put(
        root.path(),
        "plumb.toml",
        "[[layout.seat]]\npath = 'extra'\nscan = ['extra/../bad.py']\n",
    );
    let error = resolve(root.path(), &rules(LIMITS), None)
        .err()
        .expect("parent");
    assert!(error.to_string().contains("leaves its seat"), "{error}");
}

#[test]
fn overlay() {
    let root = tempfile::tempdir().expect("root");
    put(
        root.path(),
        "plumb.toml",
        r#"
[identity]
ns = "probe"

[ectropy.limit]
file = 250

[ectropy.scan]
include = ["extra/**/*.rs"]
exclude = ["extra/gen/**"]

[ectropy.module]
roots = ["extra"]

[ectropy.word]
single = true

[[ectropy.grant]]
syntax = "embed"
paths = ["crates/cli/src/rules.rs"]
"#,
    );
    put(root.path(), "crates/cli/src/rules.rs", "fn main() {}\n");
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    assert!(overlay.is_some());
    let config = resolve(root.path(), &rules(LIMITS), overlay).expect("resolve");
    assert_eq!(config.file.limit.file, 250);
    assert_eq!(config.file.limit.path, 3);
    assert!(config.file.scan.include.contains(&"crates/**/*.rs".into()));
    assert!(config.file.scan.include.contains(&"extra/**/*.rs".into()));
    assert!(config.file.scan.exclude.contains(&"extra/gen/**".into()));
    assert!(config.file.module.roots.contains(&"extra".into()));
    assert!(config.granted("crates/cli/src/rules.rs", "embed"));

    let empty = tempfile::tempdir().expect("empty");
    put(empty.path(), "plumb.toml", "[ectropy.scan]\ninclude = []\n");
    let Source::Depot(overlay) = inspect(empty.path()).expect("inspect") else {
        panic!("expected depot");
    };
    let config = resolve(empty.path(), &rules(LIMITS), overlay).expect("resolve");
    assert!(!config.file.scan.all);
    assert!(config.file.scan.include.is_empty());
}

#[test]
fn refusal() {
    let root = tempfile::tempdir().expect("root");
    put(
        root.path(),
        "plumb.toml",
        "[ectropy.word]\nsingle = false\n",
    );
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    let error = resolve(root.path(), &rules(LIMITS), overlay)
        .err()
        .expect("word.single = false");
    assert!(error.to_string().contains("word.single"), "{error}");

    put(root.path(), "ectropy.toml", "[limit]\nblock = 4\n");
    let error = inspect(root.path()).err().expect("conflict");
    assert!(
        error
            .to_string()
            .contains("both ectropy.toml and [ectropy] in plumb.toml exist"),
        "{error}"
    );

    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    assert!(matches!(
        inspect(root.path()).expect("legacy"),
        Source::Legacy(_)
    ));

    put(root.path(), "plumb.toml", "ectropy = 7\n");
    assert!(inspect(root.path()).is_err());

    put(root.path(), "plumb.toml", "[ectropy]\nunknown = true\n");
    assert!(inspect(root.path()).is_err());
}

#[test]
fn exempt() {
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "lib/tool.py", "def run():\n    return 1\n");
    put(root.path(), "tests/test.py", "def check():\n    return 1\n");
    let Source::Depot(overlay) = inspect(root.path()).expect("inspect") else {
        panic!("expected depot");
    };
    assert!(overlay.is_none());
    let custom = LIMITS.replace("file = 300", "file = 180");
    let config = resolve(root.path(), &rules(&custom), overlay).expect("resolve");
    assert!(config.file.scan.all);
    assert!(config.file.scan.include.is_empty());
    assert!(config.file.module.roots.is_empty());
    assert_eq!(config.file.limit.file, 180);
    assert_eq!(config.file.limit.path, 3);
}

#[test]
fn tracked() {
    let root = tempfile::tempdir().expect("root");
    put(root.path(), "plumb.toml", "[identity]\nns = 'probe'\n");
    put(
        root.path(),
        "crates/a/target/gen.ts",
        "export const x = 1;\n",
    );
    put(root.path(), "tools/run.js", "export const x = 1;\n");
    let before = resolve(root.path(), &rules(LIMITS), None).expect("before");
    assert!(!before.file.scan.include.contains(&"crates/**/*.ts".into()));
    assert!(!before.file.scan.include.contains(&"tools/**/*.js".into()));

    let init = plumb::config::detached("git")
        .arg("-C")
        .arg(root.path())
        .arg("init")
        .output()
        .expect("git init");
    assert!(init.status.success());
    let add = plumb::config::detached("git")
        .arg("-C")
        .arg(root.path())
        .args(["add", "tools/run.js"])
        .output()
        .expect("git add");
    assert!(add.status.success());

    let after = resolve(root.path(), &rules(LIMITS), None).expect("after");
    assert!(after.file.scan.include.contains(&"tools/**/*.js".into()));
}
