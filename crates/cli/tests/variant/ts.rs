use super::{Marks, agree, over, read};

#[test]
fn asi() {
    let with = "import { x } from \"./x\";\nconst epsilonZeta = x;\nclass Holder { gammaDelta(a: number, b: number, c: number, d: number, e: number): number { const alphaBeta = 1; if (a === 1) { return 1; } else if (a === 2) { return 2; } if (a) { if (a) { if (a) { if (a) { return alphaBeta; } } } } return 0; } }\nconst thetaIota = (a: number): number => { return a; };\nexport { epsilonZeta, Holder, thetaIota };\n";
    let without = "import { x } from \"./x\"\nconst epsilonZeta = x\nclass Holder { gammaDelta(a: number, b: number, c: number, d: number, e: number): number { const alphaBeta = 1; if (a === 1) { return 1; } else if (a === 2) { return 2; } if (a) { if (a) { if (a) { if (a) { return alphaBeta; } } } } return 0; } }\nconst thetaIota = (a: number): number => { return a }\nexport { epsilonZeta, Holder, thetaIota }\n";
    let want = read("host.ts", with);
    assert_eq!(want.len(), 9, "{want:?}");
    assert_eq!(read("host.ts", without), want);
    assert_eq!(read("host.tsx", without), want);
}

#[test]
fn decls() {
    let ctl = "export let first = 1;\nexport let alphaBeta = 2;\nexport function host(): number { let one = 1; let gammaDelta = 2; return one + gammaDelta; }\n";
    let multi = "export let first = 1, alphaBeta = 2;\nexport function host(): number { let one = 1, gammaDelta = 2; return one + gammaDelta; }\n";
    let want = read("host.ts", ctl);
    assert_eq!(
        want,
        vec![
            ("word".to_string(), "alphaBeta".to_string()),
            ("word".to_string(), "gammaDelta".to_string()),
        ]
    );
    agree(multi, &want);
}

#[test]
fn classes() {
    let decl = "export class Holder {\n  alphaBeta = 1;\n  run(a: number, b: number, c: number, d: number, e: number) { return a; }\n}\n";
    let expr = "export const Holder = class {\n  alphaBeta = 1;\n  run(a: number, b: number, c: number, d: number, e: number) { return a; }\n};\n";
    let named = "export const Holder = class Named {\n  alphaBeta = 1;\n  run(a: number, b: number, c: number, d: number, e: number) { return a; }\n};\n";
    let want = read("host.ts", decl);
    assert_eq!(want.len(), 2, "{want:?}");
    agree(expr, &want);
    agree(named, &want);
}

#[test]
fn enums() {
    for text in [
        "export const enum AlphaBeta { One, Two }",
        "const enum AlphaBeta { One, Two }",
    ] {
        let found = read("host.ts", text);
        assert!(
            found.iter().any(|hit| hit.0 == "coverage"),
            "{text}: {found:?}"
        );
        assert_eq!(read("host.tsx", text), found, "{text}");
    }
    let plain = read("host.ts", "enum AlphaBeta { One, Two }");
    assert!(plain.iter().any(|hit| hit.0 == "coverage"), "{plain:?}");
}

#[test]
fn ambient() {
    let body = "export function alphaBeta(a: number, b: number, c: number, d: number, e: number): void {}\nexport interface GammaDelta { kind: string }\n";
    let module = "declare module \"probe\" {\n  export function alphaBeta(a: number, b: number, c: number, d: number, e: number): void;\n  export interface GammaDelta { kind: string }\n}\n";
    let want = read("host.ts", body);
    assert_eq!(want.len(), 3, "{want:?}");
    let got = read("host.ts", module);
    assert_eq!(got, want, "ambient body as top-level");
    assert_eq!(read("host.tsx", module), want);
}

#[test]
fn limbs() {
    let braced = "export function host(a: number): number {\n  if (a === 1) { return 1; }\n  if (a === 2) { return 2; }\n  return 0;\n}\n";
    let bare = "export function host(a: number): number {\n  if (a === 1) return 1;\n  if (a === 2) return 2;\n  return 0;\n}\n";
    let want = read("host.ts", braced);
    assert_eq!(want, vec![("dispatch".to_string(), "a".to_string())]);
    agree(bare, &want);
}

#[test]
fn markup() {
    let deep = "export function host() { return <a><b><c><d><e><f><g><h><i><j>x</j></i></h></g></f></e></d></c></b></a>; }\n";
    let ts = read("host.ts", deep);
    assert!(
        ts.iter().any(|hit| hit.0 == "coverage"),
        "tsx-less dialect refuses markup: {ts:?}"
    );
    assert!(
        ts.iter().all(|hit| hit.0 == "coverage"),
        "no silent clean evidence: {ts:?}"
    );
    let tsx = read("host.tsx", deep);
    assert!(
        tsx.iter().any(|hit| hit.0 == "markup"),
        "tsx still reports markup: {tsx:?}"
    );
    assert!(tsx.iter().all(|hit| hit.0 == "markup"), "{tsx:?}");
    assert_eq!(
        read(
            "host.ts",
            "export function host(a: number, b: number): boolean { return a < b; }\n"
        ),
        Marks::new()
    );
}

#[test]
fn kept() {
    agree("export const run = (a, b, c, d, e) => a;", &over(1));
    agree(
        "export const run = (a, b, c, d, e) => { return a; };",
        &over(1),
    );
    let word = "page => { const alphaBeta = 1; go(alphaBeta); };";
    agree(
        word,
        &read("host.ts", &format!("function host() {{\n{word}\n}}\n")),
    );
}

const BODY: &str = "(a: number) => { const alphaBeta = 1; if (a === 1) { return 1; } else if (a === 2) { return 2; } if (a) { if (a) { if (a) { if (a) { return alphaBeta; } } } } return 0; }";

const FIVE: &str = "export function gammaDelta(a: number, b: number, c: number, d: number, e: number): number { return a; }\n";

fn five() -> Marks {
    vec![
        ("param".to_string(), "5 parameters over limit 4".to_string()),
        ("word".to_string(), "gammaDelta".to_string()),
    ]
}

#[test]
fn templates() {
    let want = read(
        "host.ts",
        &format!("export const page = String([1].map({BODY}));\n"),
    );
    assert_eq!(want.len(), 3, "{want:?}");
    agree(
        &format!("export const page = `${{[1].map({BODY})}}`;\n"),
        &want,
    );
    agree(
        &format!("export const page = `a${{`b${{[1].map({BODY})}}c`}}d`;\n"),
        &want,
    );
    agree(
        &format!("export const page = `x ${{1}} y ${{[1].map({BODY})}} z`;\n"),
        &want,
    );
    let tagged = read("host.ts", &format!("export const page = sql({BODY});\n"));
    assert_eq!(tagged.len(), 3, "{tagged:?}");
    agree(
        &format!("export const page = sql`select ${{{BODY}}}`;\n"),
        &tagged,
    );
    let deep = "export function host(a: number): number { if (a) { if (a) { if (a) { return HOLE.length; } } } return 0; }\n";
    let want = read("host.ts", &deep.replace("HOLE", "String({ x: 1 }.x)"));
    assert_eq!(want, Marks::new());
    agree(&deep.replace("HOLE", "`${`${ { x: 1 }.x }`}`"), &want);
    agree(
        &format!("export const page = `\\${{a}} \\` ${{b}}`;\n{FIVE}"),
        &five(),
    );
}

#[test]
fn apostrophe() {
    let ctl = format!(
        "export function View(): JSX.Element {{ return <p>It is here</p>; }}\n{FIVE}export function Other(): JSX.Element {{ return <p>Do not</p>; }}\n"
    );
    assert_eq!(read("host.tsx", &ctl), five());
    for (one, two) in [
        ("It's", "Don't"),
        ("say \"hi", "it\" now"),
        ("a `b", "c` d"),
        ("see https://x.org", "or /* this"),
    ] {
        let body = ctl.replace("It is here", one).replace("Do not", two);
        assert_eq!(read("host.tsx", &body), five(), "{body}");
        let nested = format!(
            "export function View(a: boolean): JSX.Element {{ return <div>{{a && <p>{one}</p>}}<p>{two}</p></div>; }}\n{FIVE}"
        );
        assert_eq!(read("host.tsx", &nested), five(), "{nested}");
        let page = format!(
            "<script lang=\"ts\">\n  let a = $state(0);\n</script>\n{one}\n<p>{one}</p>\n<button onclick={{a}}>go</button>\n<style>\n  p::before {{ content: \"'\"; }}\n</style>\n<script lang=\"ts\" context=\"module\">\n  {FIVE}</script>\n<p>{two}</p>\n"
        );
        assert_eq!(read("host.svelte", &page), five(), "{page}");
    }
}

#[test]
fn generics() {
    for code in [
        "const [s] = useState<string>('a//b');",
        "const id = <T,>(x: T) => x;\nconst s = 'a//b';",
        "const id = <T extends unknown>(x: T) => x;\nconst s = 'a//b';",
        "type Fn = <T>(x: T) => T;\nconst s = 'a//b';",
        "const n = (1 << 2) >> 1;\nconst s = 'a//b';",
        "const b = a < c && d > e;\nconst s = 'a//b';",
    ] {
        let body = format!("{code}\n{FIVE}");
        assert_eq!(read("host.ts", &body), five(), "{body}");
        assert_eq!(read("host.tsx", &body), five(), "{body}");
    }
}

#[test]
fn angles() {
    let five = "(a: A, b: B, c: C, d: D, e: E) => void";
    let angle = format!("export function host(y: unknown): unknown {{ return <{five}>y; }}\n");
    let via = format!("export function host(y: unknown): unknown {{ return y as {five}; }}\n");
    let want = read("host.ts", &via);
    assert_eq!(want, Marks::new(), "{via}");
    assert_eq!(read("host.ts", &angle), want, "{angle}");
    let script = format!("<script lang=\"ts\">\n{angle}</script>\n");
    assert_eq!(read("host.svelte", &script), want, "{script}");
    assert_eq!(
        read(
            "host.ts",
            "export function host(y: unknown): unknown { return <number>y; }\n"
        ),
        Marks::new()
    );
}

#[test]
fn defaults() {
    let five = "(b: number, c: number, d: number, e: number, f: number): number";
    let deep = "() => { if (p) { if (p) { if (p) { if (p) { go(); } } } } }";
    let block = vec![("block".to_string(), "depth over limit".to_string())];
    let function = "export function host(LIST): void { go(a); }";
    for (host, value, want) in [
        (
            function,
            format!("function {five} {{ return b; }}"),
            over(1),
        ),
        (
            "export const host = (LIST): void => { go(a); };",
            format!("{five} => {{ return b; }}"),
            over(1),
        ),
        (
            "export class Host { run(z: number, LIST): void { go(z, a); } }",
            format!("{five} => b"),
            over(1),
        ),
        (function, deep.to_string(), block),
    ] {
        let untyped = host.replace("LIST", &format!("a = {value}"));
        assert_eq!(read("host.ts", &untyped), want, "{untyped}");
        agree(
            &host.replace("LIST", &format!("a: unknown = {value}")),
            &want,
        );
    }
    for (typed, untyped, want) in [
        (
            "a: (x: number) => void = (x) => go(x), b: (y: number) => number = (y: number): number => y",
            "a = (x) => go(x), b = (y: number): number => y",
            over(0),
        ),
        (
            "a: Map<string, number> = new Map<string, number>(), b: number[] = useState<number[]>([1, 2]), c: number, d: number, e: number",
            "a = new Map<string, number>(), b = useState<number[]>([1, 2]), c: number, d: number, e: number",
            over(1),
        ),
        (
            "a: boolean = x < y, b: boolean = z > w, c: number = 1 << 2, d: boolean = p < q && r > s, e: number",
            "a = x < y, b = z > w, c = 1 << 2, d = p < q && r > s, e: number",
            over(1),
        ),
        (
            "a: { x: number } = { x: 1 }, b: number[] = [1, 2], c: [number, number] = [1, 2], d: string = `s${1}`, e: Opts = { run: 1 }",
            "a = { x: 1 }, b = [1, 2], c = [1, 2], d = `s${1}`, e = { run: 1 }",
            over(1),
        ),
        (
            "a: Fn = y as (b: B, c: C, d: D, e: E, f: F) => void, g: Fn = y satisfies (b: B, c: C, d: D, e: E, f: F) => void",
            "a = y as (b: B, c: C, d: D, e: E, f: F) => void, g = y satisfies (b: B, c: C, d: D, e: E, f: F) => void",
            over(0),
        ),
        (
            "a: (b: number, c: number, d: number, e: number, f: number) => void = noop",
            "a = noop",
            over(0),
        ),
    ] {
        let body = |list: &str| format!("export function host({list}): void {{ go(); }}\n");
        assert_eq!(read("host.ts", &body(untyped)), want, "{untyped}");
        agree(&body(typed), &want);
    }
}
