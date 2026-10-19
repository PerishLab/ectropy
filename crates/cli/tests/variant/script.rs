use super::{Marks, read};

const PROBE: &str = "export function host(a, b, c, d, e) { const alphaBeta = 1; if (a === 1) { return 1; } else if (a === 2) { return 2; } if (a) { if (a) { if (a) { if (a) { return alphaBeta; } } } } return 0; }\n";

const PLAIN: [&str; 6] = [
    "export function host(a, b) { return a < b ? a >> 1 : b << 2; }",
    "export function host(a, b, c, d, e) { return a > b && c >= d || e <= a; }",
    "export const host = (a, b, c, d, e) => a ? b : c;",
    "export class Host { run(a, b, c, d, e) { if (a) { return b; } return c; } }",
    "register(function (a, b, c, d, e) { const fooBar = a; return fooBar; });",
    "export function host(a) { if (a === 1) { return 1; } else if (a === 2) { return 2; } return 0; }",
];

const JSX: &str =
    "export const v = <a><a><a><a><a><a><a><a><a>x</a></a></a></a></a></a></a></a></a>;\n";

fn count(path: &str, text: &str, kind: grammar::Kind) -> usize {
    fn walk(node: &kernel::Node, kind: grammar::Kind) -> usize {
        let own = usize::from(node.kind == kind);
        own + node.kids.iter().map(|kid| walk(kid, kind)).sum::<usize>()
    }
    let source = grammar::Source {
        path: path.to_string(),
        text: text.to_string(),
    };
    walk(&kernel::structure(&source), kind)
}

#[test]
fn probe() {
    let want = read("host.ts", PROBE);
    let laws: Vec<&str> = want.iter().map(|hit| hit.0.as_str()).collect();
    assert_eq!(laws, ["block", "dispatch", "param", "word"], "{want:?}");
    for extension in ["js", "mjs", "cjs", "jsx", "mts", "cts"] {
        assert_eq!(
            read(&format!("host.{extension}"), PROBE),
            want,
            "{extension}"
        );
    }
}

#[test]
fn plain() {
    for form in PLAIN {
        let want = read("host.ts", form);
        assert_eq!(read("host.tsx", form), want, "{form}");
        for extension in ["js", "mjs", "cjs", "jsx", "mts", "cts", "d.mts"] {
            assert_eq!(
                read(&format!("host.{extension}"), form),
                want,
                "{extension}: {form}"
            );
        }
    }
}

#[test]
fn typed() {
    let want = read("host.ts", JSX);
    assert_ne!(read("host.tsx", JSX), want);
    let shape = "export interface T { a: number; b?: string }\n";
    for extension in ["mts", "cts", "d.mts", "d.cts"] {
        assert_eq!(read(&format!("host.{extension}"), JSX), want, "{extension}");
        let path = format!("host.{extension}");
        assert_eq!(count(&path, shape, grammar::Kind::Field), 2, "{extension}");
    }
    assert_eq!(count("host.tsx", shape, grammar::Kind::Field), 2);
}

#[test]
fn markup() {
    let want = read("host.tsx", JSX);
    assert!(want.iter().any(|hit| hit.0 == "markup"), "{want:?}");
    for extension in ["js", "mjs", "cjs", "jsx"] {
        assert_eq!(read(&format!("host.{extension}"), JSX), want, "{extension}");
    }
}

#[test]
fn evidence() {
    let text = "import a from \"./a.js\";\nexport { b } from \"../b/src/x.js\";\nconst c = import(\"./c.js\");\nconst d = process.env.HOME;\nconst e = Deno.env.get(\"E\");\nconst f = import.meta.url;\nconst g = __dirname;\nconst h = require(\"./h.js\");\n";
    for (kind, times) in [
        (grammar::Kind::Link, 3),
        (grammar::Kind::Environment, 2),
        (grammar::Kind::Embed, 2),
    ] {
        assert_eq!(count("host.ts", text, kind), times, "{times}");
        for extension in ["js", "mjs", "cjs", "jsx", "mts", "cts"] {
            let path = format!("host.{extension}");
            assert_eq!(count(&path, text, kind), times, "{extension}");
        }
    }
    let reach = |path: &str| read(path, text).into_iter().filter(|hit| hit.0 == "reach");
    assert_eq!(
        reach("host.js").collect::<Marks>(),
        reach("host.ts").collect::<Marks>()
    );
    assert_eq!(reach("host.js").count(), 1);
}

#[test]
fn readers() {
    for (extension, _) in grammar::READERS {
        assert!(kernel::path::source(extension), "{extension}");
        let source = grammar::Source {
            path: format!("host.{extension}"),
            text: "# a\n\n<p>a</p>\nlet a = 1;\n".to_string(),
        };
        assert!(
            !grammar::parse(&source).kids.is_empty(),
            "{extension} has no reader"
        );
    }
    for extension in ["pyi", "toml", "json", "d", ""] {
        assert!(!kernel::path::source(extension), "{extension}");
        let source = grammar::Source {
            path: format!("host.{extension}"),
            text: "# a\n\n<p>a</p>\nlet a = 1;\n".to_string(),
        };
        assert!(grammar::parse(&source).kids.is_empty(), "{extension}");
    }
}
