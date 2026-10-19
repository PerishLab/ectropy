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

const BRANCHES: [&str; 10] = [
    "return { k: 1 };",
    "return f({ k: 1 });",
    "go(() => {});",
    "go((a, b, c, d, e) => { go(a); });",
    "go({ setDepth: () => go() });",
    "go(function () { const alphaBeta = 1; });",
    "go(class { run() {} });",
    "var { k } = pair;",
    "return pair as { k: number };",
    "go(`${a}`, { k: `${a}` });",
];

const HEADS: [&str; 5] = [
    "for (const alphaBeta of s)",
    "for (const alphaBeta in s)",
    "for (let alphaBeta = 0; alphaBeta < 1; alphaBeta++)",
    "for await (const alphaBeta of s)",
    "while (alphaBeta)",
];

const SHALLOW: &str = "if (a) { if (a) { go(); } }";

const NEXT: [&str; 10] = [
    "if (a) { SHALLOW }",
    "for (const z of s) { SHALLOW }",
    "while (a) { SHALLOW }",
    "do { SHALLOW } while (a);",
    "switch (a) { default: { if (a) { go(); } } }",
    "try { SHALLOW } catch { go(); }",
    "const run = () => { SHALLOW };",
    "let run = () => { SHALLOW };",
    "var run = () => { SHALLOW };",
    "go(() => { SHALLOW });",
];

const ENDS: [(&str, &str); 6] = [
    ("if (a === 1) go();", "if (a === 1) { go(); }"),
    ("if (a === 1) var k = 1;", "if (a === 1) { var k = 1; }"),
    (
        "if (a === 1) go({ k: 1 });",
        "if (a === 1) { go({ k: 1 }); }",
    ),
    ("for (const z of s) go(z);", "for (const z of s) { go(z); }"),
    (
        "for (const z of s) if (z) { go(); }",
        "for (const z of s) { if (z) { go(); } }",
    ),
    ("do go(); while (a);", "do { go(); } while (a);"),
];

fn within(body: &str) -> String {
    format!("export async function host(a: number, s: S) {{\n{body}\n}}\n")
}

fn ladder(first: &str) -> String {
    within(&format!(
        "if (a === 1) {first}\nif (a === 2) return 2;\nreturn 0;"
    ))
}

#[test]
fn extent() {
    let dispatch = ("dispatch".to_string(), "a".to_string());
    for first in BRANCHES {
        let want = read("host.ts", &ladder(&format!("{{ {first} }}")));
        assert!(want.contains(&dispatch), "{first}: {want:?}");
        agree(&ladder(first), &want);
    }
    let tail =
        "if (a === 1) return { k: 1 };\nelse if (a === 2) return { k: 2 };\nelse go(() => {});";
    agree(&within(tail), &vec![dispatch]);
    let braced = within("if (s) { go(); } else { go((a, b, c, d, e) => { go(a); }); }");
    assert_eq!(read("host.ts", &braced), over(1));
    agree(
        &within("if (s) { go(); } else go((a, b, c, d, e) => { go(a); });"),
        &over(1),
    );
}

#[test]
fn heads() {
    for head in HEADS {
        for body in [
            "go(alphaBeta);",
            "go({ k: alphaBeta });",
            "if (alphaBeta) go({ k: 1 });",
        ] {
            let braced = within(&format!("{head} {{ {body} }}"));
            assert_eq!(read("host.ts", &braced), Marks::new(), "{braced}");
            agree(&within(&format!("{head} {body}")), &Marks::new());
        }
        let inner = format!("{head} go(() => {{ const gammaDelta = 1; }});");
        let word = vec![("word".to_string(), "gammaDelta".to_string())];
        agree(&within(&inner), &word);
    }
}

#[test]
fn loops() {
    for head in HEADS {
        let braced = within(&format!(
            "{head} {{ if (a === 1) go(); }}\nif (a === 2) return 2;"
        ));
        assert_eq!(read("host.ts", &braced), Marks::new(), "{braced}");
        let bare = within(&format!(
            "{head} if (a === 1) go();\nif (a === 2) return 2;"
        ));
        agree(&bare, &Marks::new());
        let inside = within(&format!(
            "{head} if (a === 1) go(); else if (a === 2) go();"
        ));
        agree(&inside, &vec![("dispatch".to_string(), "a".to_string())]);
        let deep = format!("if (a) {{ if (a) {{ if (a) {{ {head} go(); }} }} }}");
        let braced = format!("if (a) {{ if (a) {{ if (a) {{ {head} {{ go(); }} }} }} }}");
        let want = read("host.ts", &within(&braced));
        assert_eq!(want.len(), 1, "{want:?}");
        agree(&within(&deep), &want);
    }
}

#[test]
fn ends() {
    let chain = within("if (a === 1) go()\nif (a === 2) go()");
    agree(&chain, &vec![("dispatch".to_string(), "a".to_string())]);
    for next in NEXT.map(|next| next.replace("SHALLOW", SHALLOW)) {
        let want = read("host.ts", &within(&format!("{{ {next} }}")));
        assert!(want.iter().any(|hit| hit.0 == "block"), "{next}: {want:?}");
        for (bare, braced) in ENDS {
            let want = read("host.ts", &within(&format!("{braced}\n{next}")));
            assert!(
                want.iter().all(|hit| hit.0 != "block"),
                "{braced} {next}: {want:?}"
            );
            agree(&within(&format!("{bare}\n{next}")), &want);
        }
        if next.starts_with("go(") {
            continue;
        }
        let want = read(
            "host.ts",
            &within(&format!("if (a === 1) {{ go() }}\n{next}")),
        );
        agree(&within(&format!("if (a === 1) go()\n{next}")), &want);
    }
    let otherwise = format!("if (a === 1) go()\nelse {{ {SHALLOW} }}");
    let want = read(
        "host.ts",
        &within(&format!("if (a === 1) {{ go() }}\nelse {{ {SHALLOW} }}")),
    );
    agree(&within(&otherwise), &want);
}
