mod seat;
use seat::*;

#[test]
fn path() {
    let config = config(&[], 4);
    assert!(kernel::path::depth("a/b/c/d/e.rs", &config).is_none());
    assert!(kernel::path::depth("a/b/c/d/e/f.rs", &config).is_some());
}

#[test]
fn module() {
    let config = config(&["app/src"], 1);
    assert!(kernel::path::depth("app/src/core/profile.rs", &config).is_none());
    assert!(kernel::path::depth("app/src/core/profile/load.rs", &config).is_some());
}

#[test]
fn wildcard() {
    let config = config(&["crates/*/src"], 0);
    assert!(kernel::path::depth("crates/kernel/src/lib.rs", &config).is_none());
    assert!(kernel::path::depth("crates/kernel/src/config/lib.rs", &config).is_some());
}

#[test]
fn unmatched() {
    let config = config(&["app/src"], 4);
    let finding =
        kernel::path::depth("docs/principles.md", &config).expect("unmatched path should fail");
    assert_eq!(finding.note, "outside declared module roots");
}

#[test]
fn specific() {
    let config = config(&["app", "app/src/core"], 0);
    assert!(kernel::path::depth("app/src/core/lib.rs", &config).is_none());
}

#[test]
fn word() {
    let config = config(&["app/src"], 4);
    assert_eq!(
        kernel::path::word("app/src/core/internal_help.rs", &config).len(),
        1
    );
    assert!(kernel::path::word("app/src/core/profile.rs", &config).is_empty());
    assert_eq!(
        kernel::path::word("app/src/help_pages/page.rs", &config).len(),
        1
    );
    assert!(kernel::path::word("docs/adapter-contract.md", &config).is_empty());
    assert!(kernel::path::word("app/src/APIBase.rs", &config).len() == 1);
    assert!(kernel::path::word("app/src/vite.config.ts", &config).is_empty());
    assert!(kernel::path::word("app/src/Card.svelte", &config).is_empty());
    assert_eq!(
        kernel::path::word("app/src/UserCard.svelte", &config).len(),
        1
    );
    assert_eq!(
        kernel::path::word("app/src/good.bad_name.ts", &config).len(),
        1
    );
}

#[test]
fn kebab() {
    let config = config(&[], 4);
    assert_eq!(
        kernel::path::word("docs/adapter-contract.md", &config).len(),
        1
    );
}

#[test]
fn registered() {
    let mut config = config(&["app/src"], 4);
    config.file.vocabulary.term.push(kernel::config::Term {
        name: "internal_help".to_string(),
        description: "Established local compound.".to_string(),
    });
    assert!(kernel::path::word("app/src/core/internal_help.rs", &config).is_empty());
}

#[test]
fn grant() {
    let source = "#[cfg(test)]\nmod tests {}\n#[test]\nfn probe() {}";
    assert_eq!(count(scan("src/lib.rs", source), "grant"), 0);
    let mut config = config(&[], 4);
    config.file.grant.push(kernel::config::Grant {
        syntax: "test".to_string(),
        paths: vec!["tests/**/*.rs".to_string()],
    });
    assert_eq!(hits("src/lib.rs", source, &config), 2);
    assert_eq!(hits("tests/laws.rs", source, &config), 0);
    assert_eq!(
        hits("src/gated.rs", "#[cfg(not(test))]\nfn probe() {}", &config),
        0
    );
}

#[test]
fn shadow() {
    let bridge = "fn load() { let held = Config { listen: file.listen, store: file.store, identity: file.identity, cache: file.cache, root }; }";
    assert_eq!(count(laws(bridge), "shadow"), 1);
    let source = grammar::Source {
        path: "t.rs".to_string(),
        text: bridge.to_string(),
    };
    let node = kernel::structure(&source);
    let config = config(&[], 4);
    let hit = kernel::check(&source, &node, &config)
        .into_iter()
        .find(|finding| finding.law == "shadow")
        .expect("shadow");
    assert_eq!(
        hit.note,
        "4 of 5 fields copied bare from one root, the twin is redundant; see: ectropy cookbook shadow"
    );
    let whole = "fn hold() { let held = Trio { a: other.a, b: other.b, c: other.c, }; }";
    assert_eq!(count(laws(whole), "shadow"), 1);
    let decode = "fn read() { let held = Row { id: row.get(0), prefix: row.get(1), digest: row.get(2), created: row.get(3) }; }";
    assert_eq!(count(laws(decode), "shadow"), 0);
    let pair = "fn tiny() { let held = Pair { a: other.a, b: other.b }; }";
    assert_eq!(count(laws(pair), "shadow"), 0);
    let sparse =
        "fn sparse() { let held = Wide { a: other.a, b: other.b, c: other.c, d: other.d, e, f }; }";
    assert_eq!(count(laws(sparse), "shadow"), 0);
    let mixed = "fn mix() { let held = Meta { provider: seat.provider, model: seat.model.clone(), budget: seat.budget, source: label }; }";
    assert_eq!(count(laws(mixed), "shadow"), 0);
    let split = "fn far() { let held = Wide { a: left.a, b: right.b, c: left.c, d: right.d }; }";
    assert_eq!(count(laws(split), "shadow"), 0);
}

#[test]
fn environment() {
    let bare = config(&[], 4);
    assert_eq!(
        hits(
            "src/lib.rs",
            "fn f() { let port = std::env::var(\"PORT\"); }",
            &bare
        ),
        1
    );
    assert_eq!(hits("src/lib.rs", "use std::env;\nfn f() {}", &bare), 1);
    assert_eq!(
        hits("src/lib.rs", "use std::{fmt, env};\nfn f() {}", &bare),
        1
    );
    assert_eq!(
        hits("src/lib.rs", "use std::{fmt, option};\nfn f() {}", &bare),
        0
    );
    assert_eq!(
        hits("src/lib.rs", "fn f() { let text = \"std::env\"; }", &bare),
        0
    );
    let mut granted = config(&[], 4);
    granted.file.grant.push(kernel::config::Grant {
        syntax: "environment".to_string(),
        paths: vec!["tests/**/*.rs".to_string()],
    });
    assert_eq!(
        hits(
            "tests/laws.rs",
            "fn f() { std::env::set_var(\"A\", \"b\"); }",
            &granted
        ),
        0
    );
    assert_eq!(
        hits("src/lib.rs", "fn f() { std::env::var(\"A\"); }", &granted),
        1
    );
}

#[test]
fn surface() {
    let bare = config(&[], 4);
    assert_eq!(
        hits("lib/cmd.ts", "const home = Deno.env.get(\"HOME\");", &bare),
        1
    );
    assert_eq!(
        hits("lib/cmd.ts", "const home = process.env.HOME;", &bare),
        1
    );
    let mut granted = config(&[], 4);
    granted.file.grant.push(kernel::config::Grant {
        syntax: "environment".to_string(),
        paths: vec!["**/*.test.ts".to_string()],
    });
    assert_eq!(
        hits(
            "lib/cmd.test.ts",
            "const home = Deno.env.get(\"HOME\");",
            &granted
        ),
        0
    );
}

#[test]
fn deno() {
    let mut config = config(&[], 4);
    config.file.grant.push(kernel::config::Grant {
        syntax: "test".to_string(),
        paths: vec!["**/*.test.ts".to_string()],
    });
    let source = "Deno.test(\"probe\", () => {});";
    assert_eq!(hits("lib/cmd.ts", source, &config), 1);
    assert_eq!(hits("lib/cmd.test.ts", source, &config), 0);
}

#[test]
fn length() {
    let config = config(&[], 4);
    assert!(kernel::path::length("src/mod.rs", 300, &config).is_none());
    let hit = kernel::path::length("src/mod.rs", 301, &config).expect("over limit");
    assert_eq!(hit.law, "file");
    assert_eq!(hit.note, "length 301 over limit 300");
}

#[test]
fn fanout() {
    let config = config(&[], 4);
    let calm: Vec<String> = (0..10).map(|at| format!("src/calm{at}.rs")).collect();
    assert!(kernel::path::fanout(&calm, &config).is_empty());
    let mut wide: Vec<String> = (0..11).map(|at| format!("src/wide{at}.rs")).collect();
    let findings = kernel::path::fanout(&wide, &config);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].law, "fanout");
    assert_eq!(findings[0].path, "src");
    assert_eq!(
        findings[0].note,
        "fanout 11 over limit 10; see: ectropy cookbook fanout"
    );
    wide.truncate(6);
    wide.extend((0..5).map(|at| format!("src/deep{at}/one.rs")));
    assert_eq!(kernel::path::fanout(&wide, &config).len(), 1);
}
