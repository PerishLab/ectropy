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

fn reaches(path: &str, text: &str) -> Vec<String> {
    findings(path, text, &config(&[], 4))
        .into_iter()
        .filter(|finding| finding.law == "reach")
        .map(|finding| finding.note)
        .collect()
}

#[test]
fn wire() {
    let found = reaches("tests/cli.rs", "#[path = \"../../src/x.rs\"]\nmod x;");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("climbs into a src tree"), "{found:?}");
    assert!(found[0].contains("structure.reach"), "{found:?}");
    assert_eq!(
        reaches("src/lib.rs", "#[path = \"../src/x.rs\"]\nmod x;").len(),
        1
    );
    for source in [
        "#[path = \"support/mod.rs\"]\nmod support;",
        "#[path = \"../support/mod.rs\"]\nmod support;",
        "#[path = \"control/doctor.rs\"]\nmod doctor;",
        "#[path = \"src/../x.rs\"]\nmod x;",
        "fn f() { let path = \"../src/x.rs\"; }",
    ] {
        assert_eq!(reaches("tests/cli.rs", source).len(), 0, "{source}");
    }
}

#[test]
fn script() {
    for path in ["tests/x.ts", "tests/x.tsx", "src/x.svelte"] {
        for source in [
            "import \"../src/x\";",
            "import { x } from \"../src/x\";",
            "import x from '../../src/x';",
            "import type { X } from \"../../packages/web/src/x\";",
            "export { x } from \"../src/x\";",
            "export * from '../src/x';",
            "const x = await import(\"../src/x\");",
            "const x = await import(`../src/x`);",
        ] {
            let text = wrap(path, source);
            assert_eq!(reaches(path, &text).len(), 1, "{path}: {source}");
        }
    }
}

#[test]
fn allowed() {
    for path in ["tests/x.ts", "tests/x.tsx", "src/x.svelte"] {
        for source in [
            "import \"./x\";",
            "import { x } from \"./src/x\";",
            "import { x } from \"../tests/helper\";",
            "import { x } from \"@noema/web/hooks\";",
            "import { x } from \"../source/x\";",
            "export { x } from \"./x\";",
            "const x = await import(\"./x\");",
            "const x = await import(`../${root}/src/x`);",
            "const note = \"../src/x\";",
            "const x = Array.from(\"../src\");",
            "vi.mock(\"../src/x\");",
        ] {
            let text = wrap(path, source);
            assert_eq!(reaches(path, &text).len(), 0, "{path}: {source}");
        }
    }
}

fn wrap(path: &str, source: &str) -> String {
    if path.ends_with(".svelte") {
        format!("<script lang=\"ts\">\n{source}\n</script>\n")
    } else {
        source.to_string()
    }
}

#[test]
fn abi() {
    for qualifier in [
        "extern",
        "extern \"system\"",
        "unsafe extern \"system\"",
        "unsafe extern \"C\"",
        "extern \"C-unwind\"",
    ] {
        let source = format!(
            "pub {qualifier} fn callback(handle: usize, message: u32, word: usize, parameter: isize) -> isize {{ 0 }}"
        );
        assert!(!laws(&source).contains(&"coverage".to_string()), "{source}");
    }
}

#[test]
fn callback() {
    let source = r#"pub unsafe extern "system" fn bad_name(a: usize, b: usize, c: usize, d: usize, e: usize) { if a { if b { if c { if d { return; } } } } }"#;
    let found = laws(source);
    assert!(!found.contains(&"coverage".to_string()));
    assert!(found.contains(&"word".to_string()));
    assert!(found.contains(&"param".to_string()));
    assert!(found.contains(&"block".to_string()));
}

#[test]
fn malformed() {
    for source in [
        r#"pub unsafe extern 123 fn callback() {}"#,
        r#"pub unsafe extern "system" "C" fn callback() {}"#,
    ] {
        assert!(laws(source).contains(&"coverage".to_string()), "{source}");
    }
}
