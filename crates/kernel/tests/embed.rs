mod seat;
use seat::*;

fn granted() -> kernel::config::Config {
    let mut config = config(&[], 4);
    config.file.grant.push(kernel::config::Grant {
        syntax: "embed".to_string(),
        paths: vec!["src/**".to_string()],
    });
    config
}

fn notes(path: &str, text: &str, config: &kernel::config::Config) -> Vec<String> {
    findings(path, text, config)
        .into_iter()
        .filter(|finding| finding.law == "grant")
        .map(|finding| finding.note)
        .collect()
}

#[test]
fn rust() {
    let bare = config(&[], 4);
    let cases = [
        "const TEXT: &str = include_str!(\"../data/rules.txt\");",
        "const BYTES: &[u8] = include_bytes!(\"logo.png\");",
        "include!(\"generated.rs\");",
        "fn root() -> &'static str { env!(\"CARGO_MANIFEST_DIR\") }",
        "fn root() -> Option<&'static str> { option_env!(\"CARGO_MANIFEST_DIR\") }",
        "fn root() { let dir = env::var(\"CARGO_MANIFEST_DIR\"); }",
        "fn root() { let dir = concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/fixtures\"); }",
        "fn root() { let dir = CARGO_MANIFEST_DIR; }",
    ];
    for source in cases {
        let found = notes("src/lib.rs", source, &bare);
        assert_eq!(found.len(), 1, "{source}: {found:?}");
        assert!(found[0].contains("outside granted paths"), "{found:?}");
        assert!(found[0].contains("structure.embed"), "{found:?}");
        assert_eq!(hits("src/lib.rs", source, &granted()), 0, "{source}");
        assert_eq!(hits("tests/cli.rs", source, &granted()), 1, "{source}");
    }
}

#[test]
fn plain() {
    let bare = config(&[], 4);
    for source in [
        "fn f() { let include = 1; let same = include != 2; }",
        "fn f() { let note = \"CARGO_MANIFEST_DIR is unset\"; }",
        "fn f() { let bin = env!(\"CARGO_BIN_EXE_ectropy\"); }",
        "#[derive(Debug)]\nstruct Path { path: String }",
        "#[path = \"x.rs\"]\nmod x;",
    ] {
        assert_eq!(hits("src/lib.rs", source, &bare), 0, "{source}");
    }
}

#[test]
fn marker() {
    let config = granted();
    let module = "const RULES: &str = include_str!(\"rules.txt\");\n#[cfg(test)]\n#[allow(dead_code)]\nmod tests {\n    const FIXTURE: &str = include_str!(\"lib.rs\");\n}";
    let found = notes("src/lib.rs", module, &config);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("inside test code"), "{found:?}");
    let function = "fn load() -> &'static [u8] { include_bytes!(\"logo.png\") }\n#[test]\nfn probe() {\n    let root = env!(\"CARGO_MANIFEST_DIR\");\n}";
    assert_eq!(hits("src/lib.rs", function, &config), 1);
    let nested = "mod store {\n    #[test]\n    fn probe() { include!(\"case.rs\"); }\n    fn load() { include!(\"table.rs\"); }\n}";
    assert_eq!(hits("src/lib.rs", nested, &config), 1);
    let inner = "#![cfg(test)]\nconst FIXTURE: &str = include_str!(\"lib.rs\");";
    assert_eq!(hits("src/lib.rs", inner, &config), 1);
}

#[test]
fn ban() {
    let mut config = granted();
    config.file.ban.push(kernel::config::Ban {
        syntax: "embed".to_string(),
        paths: vec!["src/cli/**".to_string()],
    });
    let source = "const TEXT: &str = include_str!(\"help.txt\");";
    let laws: Vec<String> = findings("src/cli/help.rs", source, &config)
        .into_iter()
        .map(|finding| finding.law)
        .collect();
    assert_eq!(laws, ["ban"]);
    assert_eq!(hits("src/help.rs", source, &config), 0);
}

#[test]
fn web() {
    let bare = config(&[], 4);
    let cases = [
        "const here = new URL(\"./data.json\", import.meta.url);",
        "const dir = import.meta.dirname;",
        "const file = import.meta.filename;",
        "const pages = import.meta.glob(\"./pages/*.ts\");",
        "const dir = __dirname;",
        "const file = __filename;",
    ];
    for source in cases {
        for path in ["src/load.ts", "src/Load.tsx"] {
            assert_eq!(hits(path, source, &bare), 1, "{path}: {source}");
            assert_eq!(hits(path, source, &granted()), 0, "{path}: {source}");
        }
        let svelte = format!("<script lang=\"ts\">{source}</script><p>x</p>");
        assert_eq!(hits("src/Load.svelte", &svelte, &bare), 1, "{svelte}");
    }
    assert_eq!(
        hits("src/load.ts", "const mode = import.meta.env.MODE;", &bare),
        0
    );
}

#[test]
fn python() {
    let bare = config(&[], 4);
    let source = "import os\nroot = os.path.dirname(__file__)\n";
    assert_eq!(hits("tools/load.py", source, &bare), 1);
    assert_eq!(hits("src/load.py", source, &granted()), 0);
    assert_eq!(hits("tools/load.py", "name = \"__file__\"\n", &bare), 0);
}
