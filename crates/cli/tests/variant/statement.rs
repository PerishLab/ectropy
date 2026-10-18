use super::{Marks, agree, over, read};

const ARROWS: [&str; 9] = [
    "async page => { go(page); }",
    "page => { go(page); }",
    "(page) => { go(page); }",
    "async (page) => { go(page); }",
    "async () => { go(); }",
    "async page => go(page)",
    "page => go(page)",
    "<T,>(page: T) => go(page)",
    "(async page => { go(page); })",
];

const PHRASES: [&str; 19] = [
    "Test262Error;",
    "Test262Error.x;",
    "(Test262Error);",
    "\"use strict\";",
    "1;",
    "new Foo();",
    "x++;",
    "++x;",
    "void main();",
    "delete o.k;",
    "(async () => { await go(); })();",
    "!function () { go(); }();",
    "html`<p></p>`;",
    "x?.y();",
    "a, b;",
    "x && go();",
    "{ go(); }",
    "function* walk() { yield 1; }",
    "async function* walk() { yield 1; }",
];

const DEFAULTS: [&str; 6] = [
    "async page => { go(page); }",
    "(page) => page",
    "foo",
    "{ a: 1 }",
    "[1, 2]",
    "\"x\"",
];

const DEEP: &str = "if (a) { if (a) { if (a) { if (a) { go(); } } } }";

fn hosted(text: &str) -> Marks {
    read("host.ts", &format!("function host() {{\n{text}\n}}\n"))
}

fn script(text: &str) -> Marks {
    read(
        "host.svelte",
        &format!("<script lang=\"ts\">\n{text}\n</script>\n"),
    )
}

#[test]
fn statements() {
    for arrow in ARROWS {
        agree(&format!("{arrow};"), &Marks::new());
        assert_eq!(hosted(&format!("{arrow};")), Marks::new(), "{arrow}");
    }
    for text in PHRASES {
        agree(text, &Marks::new());
        assert_eq!(hosted(text), Marks::new(), "{text}");
    }
    for text in DEFAULTS {
        agree(&format!("export default {text};"), &Marks::new());
    }
    assert_eq!(
        read("view.tsx", "(p) => <a>x</a>;\n<b>y</b>;"),
        Marks::new()
    );
}

#[test]
fn open() {
    for text in ARROWS.iter().chain(&["Test262Error"]) {
        for path in ["host.ts", "host.tsx"] {
            assert_eq!(read(path, text), Marks::new(), "{text}");
        }
        assert_eq!(
            script(text),
            script(&format!("const run = {text}")),
            "{text}"
        );
    }
}

#[test]
fn readings() {
    let want = read(
        "host.ts",
        &format!("const run = async page => {{ {DEEP} }};"),
    );
    assert_eq!(
        want,
        vec![("block".to_string(), "depth over limit".to_string())]
    );
    for head in ["async page", "page", "(page)", "async (page)", "async ()"] {
        let body = format!("{head} => {{ {DEEP} }}");
        agree(&format!("{body};"), &want);
        agree(&format!("({body});"), &want);
        agree(&format!("export default {body};"), &want);
    }
    agree(&format!("{{ {DEEP} }}"), &hosted(DEEP));
    for text in [
        "(a, b, c, d, e) => a;",
        "async (a, b, c, d, e) => { go(); };",
        "export default (a, b, c, d, e) => a;",
        "function* walk(a, b, c, d, e) { yield a; }",
        "!function (a, b, c, d, e) { go(); }();",
    ] {
        agree(text, &over(1));
    }
    let word = "page => { const alphaBeta = 1; go(alphaBeta); };";
    agree(word, &hosted(word));
    assert_eq!(
        hosted(word),
        vec![("word".to_string(), "alphaBeta".to_string())]
    );
}

#[test]
fn refusals() {
    for text in [
        "= 1;",
        "declare const n: number;",
        "outer: for (const a of b) { go(a); }",
    ] {
        let found = read("host.ts", text);
        assert!(
            found.iter().any(|hit| hit.0 == "coverage"),
            "{text}: {found:?}"
        );
    }
}
