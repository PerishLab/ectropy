use super::{BRANCHES, NEXT, SHALLOW, within};
use crate::{Marks, agree, read};

const IFS: [&str; 5] = [
    "if (s)",
    "if (!s.k)",
    "if (s.k === 1)",
    "if (a > 1 && s)",
    "if (go(a))",
];

const MORE: [&str; 3] = [
    "go();",
    "return { run(a, b, c, d, e) { return a; } };",
    "return { alphaBeta: () => 1 };",
];

fn nest(body: &str) -> String {
    format!("if (a) {{ if (a) {{ if (a) {{ {body} }} }} }}")
}

fn block(found: &Marks) -> usize {
    found.iter().filter(|hit| hit.0 == "block").count()
}

#[test]
fn evidence() {
    for head in IFS {
        for first in BRANCHES.iter().chain(&MORE) {
            let (bare, braced) = (format!("{head} {first}"), format!("{head} {{ {first} }}"));
            agree(&within(&bare), &read("host.ts", &within(&braced)));
            let want = read("host.ts", &within(&nest(&braced)));
            assert!(block(&want) > 0, "{braced}: {want:?}");
            agree(&within(&nest(&bare)), &want);
            let want = read("host.ts", &format!("if (a) {{ {} }}", nest(&braced)));
            assert!(block(&want) > 0, "{braced}: {want:?}");
            agree(&format!("if (a) {{ {} }}", nest(&bare)), &want);
        }
    }
}

#[test]
fn ladders() {
    let dispatch = vec![("dispatch".to_string(), "a".to_string())];
    for head in IFS {
        let braced = within(&format!(
            "{head} {{ if (a === 1) go(); }}\nif (a === 2) return 2;"
        ));
        assert_eq!(read("host.ts", &braced), Marks::new(), "{braced}");
        agree(
            &within(&format!(
                "{head} if (a === 1) go();\nif (a === 2) return 2;"
            )),
            &Marks::new(),
        );
        agree(
            &within(&format!(
                "{head} if (a === 1) go(); else if (a === 2) go();"
            )),
            &dispatch,
        );
        let rest = "else if (a === 1) go(); else if (a === 2) go();";
        let braced = within(&format!("{head} {{ go(); }} {rest}"));
        assert_eq!(read("host.ts", &braced), dispatch, "{braced}");
        agree(&within(&format!("{head} go(); {rest}")), &dispatch);
    }
}

#[test]
fn tails() {
    for head in IFS {
        let want = read(
            "host.ts",
            &within(&nest(&format!("{head} {{ go(); }} else {{ go(); }}"))),
        );
        assert_eq!(block(&want), 2, "{head}: {want:?}");
        agree(&within(&nest(&format!("{head} go(); else go();"))), &want);
        for next in NEXT {
            after(head, &next.replace("SHALLOW", SHALLOW));
        }
    }
}

fn after(head: &str, next: &str) {
    for end in [";", ""] {
        let want = read(
            "host.ts",
            &within(&format!("{head} {{ go(){end} }}\n{next}")),
        );
        assert_eq!(block(&want), 0, "{head} {next}: {want:?}");
        if end.is_empty() && next.starts_with("go(") {
            continue;
        }
        agree(&within(&format!("{head} go(){end}\n{next}")), &want);
    }
}
