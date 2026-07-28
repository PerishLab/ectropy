use kernel::config::{Config, File, Glob};
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

#[test]
fn defaults() {
    let file: File = toml::from_str("").expect("empty config should parse");
    assert_eq!(file.limit.block, 4);
    assert_eq!(file.limit.path, 4);
    assert!(file.module.roots.is_empty());
}

#[test]
fn module() {
    let file: File = toml::from_str(
        r#"
[module]
roots = ["app/src", "crates/*/src"]
"#,
    )
    .expect("module roots should parse");
    assert_eq!(file.module.roots, ["app/src", "crates/*/src"]);
    assert_eq!(
        Glob::root("crates/*/src", "crates/kernel/src/lib.rs"),
        Some(3)
    );
    assert_eq!(
        Glob::root("workspace/**/src", "workspace/a/b/src/lib.rs"),
        Some(4)
    );
}

#[test]
fn boundary() {
    let file: File = toml::from_str(
        r#"
[[boundary]]
paths = ["packages/*/vitest.config.ts"]
allow = ["path"]

[[boundary]]
paths = [".runseal/**"]
allow = ["comment"]
"#,
    )
    .expect("boundary edges should parse");
    let config = Config { file };
    assert!(config.exempt("packages/react/vitest.config.ts", "path"));
    assert!(!config.exempt("packages/react/vite.config.ts", "path"));
    assert!(!config.exempt("packages/react/vitest.config.ts", "comment"));
    assert!(config.exempt(".runseal", "comment"));
    assert!(config.exempt(".runseal/wrappers/land.ts", "comment"));
    assert!(!config.exempt(".runseallike/land.ts", "comment"));
}

#[test]
fn ban() {
    let file: File = toml::from_str(
        r#"
[[ban]]
syntax = "style"
paths = ["apps/*/src/lib/components/**"]
"#,
    )
    .expect("ban should parse");
    let config = Config { file };
    assert!(config.banned("apps/web/src/lib/components/card.tsx", "style"));
    assert!(!config.banned("apps/web/src/views/card.tsx", "style"));
    assert!(!config.banned("apps/web/src/lib/components/card.tsx", "test"));
}

#[test]
fn glob() {
    assert!(Glob::matches(".runseal/**", ".runseal/lib/cli.ts"));
    assert!(Glob::matches("app/src/**/*.rs", "app/src/lib.rs"));
    assert!(Glob::matches("app/src/**/*.rs", "app/src/core/mod.rs"));
    assert!(Glob::matches("**/target/**", "target/debug/app"));
    assert!(Glob::matches("**/target/**", "app/target/debug/app"));
    assert!(!Glob::matches("docs/**/*.md", "crates/kernel/src/lib.rs"));
}

#[test]
fn validation() {
    let cases = [
        "unknown = true",
        "[[boundary]]\npaths = [\"src/**\"]\nallow = [\"missing\"]\nnote = \"why\"\n",
        "[scan]\ninclude = [\"src/**x/*.rs\"]\n",
        "[[grant]]\nsyntax = \"missing\"\npaths = [\"src/**\"]\n",
        "[[ban]]\nsyntax = \"missing\"\npaths = [\"src/**\"]\n",
        "[[ban]]\nsyntax = \"style\"\npaths = []\n",
        "[[ban]]\nsyntax = \"style\"\npaths = [\"src/**x/*.tsx\"]\n",
        "[[vocabulary.term]]\nname = \"two_words\"\ndescription = \"\"\n",
        "[[vocabulary.term]]\nname = \"two_words\"\ndescription = \"one\"\n[[vocabulary.term]]\nname = \"two_words\"\ndescription = \"two\"\n",
    ];
    for (at, text) in cases.iter().enumerate() {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("ectropy-config-{id}-{at}"));
        fs::create_dir(&root).expect("create config root");
        fs::write(root.join("ectropy.toml"), text).expect("write config");
        let loaded = kernel::config::load(&root);
        fs::remove_dir_all(&root).expect("remove config root");
        assert!(loaded.is_err(), "case {at} should fail");
    }
}
