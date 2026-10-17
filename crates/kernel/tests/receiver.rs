mod seat;
use seat::*;

#[test]
fn dispatch() {
    let ladder = "fn f() { if x == 1 { a(); } if x == 2 { b(); } }";
    assert!(laws(ladder).contains(&"dispatch".to_string()));
    let bytes = "fn f() { if x == b'a' { one(); } if x == b'b' { two(); } }";
    assert!(laws(bytes).contains(&"dispatch".to_string()));
    let mixed = "fn f() { if x == 1 { a(); } if y == 2 { b(); } }";
    assert!(!laws(mixed).contains(&"dispatch".to_string()));
    let vars = "fn f() { if x == y { a(); } if x == z { b(); } }";
    assert!(!laws(vars).contains(&"dispatch".to_string()));
    let split = "fn f() { if x == 1 { a(); } probe(); if x == 1 { b(); } }";
    assert!(!laws(split).contains(&"dispatch".to_string()));
}

#[test]
fn elseif() {
    let chain = "fn f() { if x == 1 { a(); } else if x == 2 { b(); } }";
    assert!(laws(chain).contains(&"dispatch".to_string()));
}

#[test]
fn switch() {
    let chain = scan(
        "t.ts",
        "export function f(x: string): void { if (x === 'a') { a(); } else if (x === 'b') { b(); } }",
    );
    assert!(chain.contains(&"dispatch".to_string()));
    let guard = scan(
        "t.ts",
        "export function f(x: string, y: string): void { if (x === 'a') { a(); } if (y === 'b') { b(); } }",
    );
    assert!(!guard.contains(&"dispatch".to_string()));
}

#[test]
fn receiver() {
    let quad = "fn load(app: &App) {}\nfn save(app: &App) {}\nfn query(app: &App) {}\nfn delete(app: &App) {}";
    let config = config(&[], 4);
    let found = findings("t.rs", quad, &config);
    let grouped: Vec<&kernel::Finding> = found
        .iter()
        .filter(|finding| finding.law == "receiver")
        .collect();
    assert_eq!(grouped.len(), 1);
    assert_eq!(grouped[0].line, 1);
    assert_eq!(grouped[0].col, 9);
    assert_eq!(
        grouped[0].note,
        "4 functions share app: load, save, query, delete; see: ectropy cookbook structure.receiver"
    );
    let trio = "fn a(st: &State) {}\nfn b(st: &State) {}\nfn c(st: &State) {}";
    assert_eq!(count(laws(trio), "receiver"), 0);
    let lives =
        "fn a(st: &State) {}\nfn b<'x>(st: &'x State) {}\nfn c(st: &State) {}\nfn d(st: &State) {}";
    assert_eq!(count(laws(lives), "receiver"), 1);
    let other =
        "fn a(st: &State) {}\nfn b(st: &Statement) {}\nfn c(st: &State) {}\nfn d(st: &State) {}";
    assert_eq!(count(laws(other), "receiver"), 0);
    let five = "fn a(st: &State) {}\nfn b(st: &State) {}\nfn c(st: &State) {}\nfn d(st: &State) {}\nfn e(st: &State) {}";
    let found = findings("t.rs", five, &config);
    let points: Vec<(usize, usize)> = found
        .iter()
        .filter(|finding| finding.law == "receiver")
        .map(|finding| (finding.line, finding.col))
        .collect();
    assert_eq!(points, vec![(1, 6)]);
    let eight = "fn a(st: &State) {}\nfn b(st: &State) {}\nfn c(st: &State) {}\nfn d(st: &State) {}\nfn e(st: &State) {}\nfn f(st: &State) {}\nfn g(st: &State) {}\nfn h(st: &State) {}";
    assert_eq!(count(laws(eight), "receiver"), 1);
    let pair = "fn a(st: &State) {}\nfn b(st: &State) {}\nfn c(st: &State) {}\nfn d(st: &State) {}\nfn e(dw: &Deck) {}\nfn f(dw: &Deck) {}\nfn g(dw: &Deck) {}\nfn h(dw: &Deck) {}";
    assert_eq!(count(laws(pair), "receiver"), 2);
}

#[test]
fn methods() {
    let body = "impl X {\n    fn a(&self, n: u32) {}\n    fn b(&self, n: u32) {}\n    fn c(&self, n: u32) {}\n    fn d(&self, n: u32) {}\n}";
    assert_eq!(count(laws(body), "receiver"), 0);
}

#[test]
fn exported() {
    let quad = "export function a(x: string): void {}\nexport function b(x: string): void {}\nfunction c(x: string): void {}\nexport function d(x: string): void {}";
    let found = scan("t.ts", quad);
    assert_eq!(count(found.clone(), "receiver"), 1);
    assert!(!found.contains(&"coverage".to_string()));
}

#[test]
fn arity() {
    let five = "fn f(a: u32, b: u32, c: u32, d: u32, e: u32) {}";
    assert_eq!(count(laws(five), "param"), 1);
    let four = "fn f(a: u32, b: u32, c: u32, d: u32) {}";
    assert_eq!(count(laws(four), "param"), 0);
    let method = "impl X {\n    fn f(&self, a: u32, b: u32, c: u32, d: u32, e: u32) {}\n}";
    assert_eq!(count(laws(method), "param"), 1);
}

#[test]
fn burr() {
    let config = config(&[], 4);
    let rust = "fn read(mut _plan: Plan, _mode: Mode) {}";
    let found = findings("t.rs", rust, &config);
    let burrs: Vec<&kernel::Finding> = found
        .iter()
        .filter(|finding| finding.law == "burr")
        .collect();
    assert_eq!(burrs.len(), 2);
    assert_eq!(burrs[0].line, 1);
    assert_eq!(burrs[0].col, 13);
    assert_eq!(
        burrs[0].note,
        "_plan is accepted but unused; see: ectropy cookbook structure.burr"
    );
    let typescript = "export function read(_plan: Plan, _mode?: Mode): void {}";
    assert_eq!(count(scan("t.ts", typescript), "burr"), 2);
    let plain = "fn read(_: Plan, plan: Plan, __: Mode) {}";
    assert_eq!(count(laws(plain), "burr"), 0);
    let pattern = "fn read((_plan, plan): (Plan, Plan)) {}";
    assert_eq!(count(laws(pattern), "burr"), 0);
}

#[test]
fn border() {
    let mut config = config(&[], 4);
    config.file.boundary.push(kernel::config::Boundary {
        paths: vec!["vendor/**".to_string()],
        allow: vec!["burr".to_string()],
        note: "vendored fixture accepts conventional ignored parameters".to_string(),
    });
    let found = findings("vendor/lib.rs", "fn read(_plan: Plan) {}", &config);
    assert!(!found.iter().any(|finding| finding.law == "burr"));
}

#[test]
fn minted() {
    let named = "const API_BASE: &str = \"x\";\nstatic RETRY_COUNT: u32 = 3;";
    assert_eq!(count(laws(named), "word"), 2);
    assert_eq!(count(laws("const BASE: &str = \"x\";"), "word"), 0);
    assert_eq!(count(laws("use a::b_c;"), "word"), 0);
}

#[test]
fn hoist() {
    let early = "function render(view: unknown): string {\n  return String(view);\n}\n\ntest(\"home\", () => {\n  render(1);\n});";
    assert!(!scan("t.ts", early).contains(&"coverage".to_string()));
    assert!(!scan("t.tsx", early).contains(&"coverage".to_string()));
    let shaped = "function shape(): { wide: boolean } | null {\n  return null;\n}";
    assert!(!scan("t.ts", shaped).contains(&"coverage".to_string()));
}

const PARAMS: &str = "(a: boolean, b: boolean, c: boolean, d: boolean, e: boolean): number";
const BODY: &str = "{\n    const deepValue = 1;\n    if (a) { if (b) { if (c) { if (d) { if (e) { return deepValue; } } } } }\n    return 0;\n  }";

type Marks = Vec<(String, String)>;

fn marks(path: &str, text: &str) -> Marks {
    let found = findings(path, text, &config(&[], 4)).into_iter();
    let mut found: Marks = found.map(|hit| (hit.law, hit.note)).collect();
    found.sort();
    found
}

fn hit(law: &str, note: &str) -> (String, String) {
    (law.to_string(), note.to_string())
}

fn tally(want: &Marks, law: &str) -> usize {
    want.iter().filter(|mark| mark.0 == law).count()
}

fn script(text: &str) -> String {
    format!("<script lang=\"ts\">\n{text}\n</script>\n")
}

fn class(members: &str) -> String {
    format!("export class Walker {{\n{members}\n}}\n")
}

fn agree(text: &str, want: &[(String, String)]) {
    assert_eq!(marks("walker.ts", text), want, "{text}");
    assert_eq!(marks("walker.tsx", text), want, "{text}");
    assert_eq!(marks("walker.svelte", &script(text)), want, "{text}");
}

fn free() -> Marks {
    let text = format!("export function walkTree{PARAMS} {BODY}\n");
    marks("walker.ts", &text)
}

fn method(head: &str) -> String {
    format!("  {head}{PARAMS} {BODY}")
}

#[test]
fn parity() {
    let want = free();
    assert_eq!(tally(&want, "block"), 2);
    assert!(want.contains(&hit("param", "5 parameters over limit 4")));
    assert!(want.contains(&hit("word", "walkTree")));
    assert!(want.contains(&hit("word", "deepValue")));
    agree(&class(&method("walkTree")), &want);
    let late = format!("class Walker {{\n  walkTree{PARAMS} {BODY}\n}}\nexport {{ Walker }};\n");
    agree(&late, &want);
}

#[test]
fn modifiers() {
    let want = free();
    let heads = [
        "static async walkTree",
        "public walkTree",
        "private static walkTree",
        "protected override walkTree",
        "async *walkTree",
        "get walkTree",
        "walkTree?",
        "walkTree<T>",
        "@trace(\"walk\")\n  @sealed\n  walkTree",
    ];
    for head in heads {
        agree(&class(&method(head)), &want);
    }
    let heads = [
        "export default class",
        "export default class extends Base",
        "class Walker extends Base<Item> implements Thing, Other",
        "export abstract class Walker",
    ];
    for head in heads {
        agree(&format!("{head} {{\n{}\n}}\n", method("walkTree")), &want);
    }
}

#[test]
fn constructor() {
    let mut want = free();
    want.retain(|mark| mark.1 != "walkTree");
    agree(&class(&method("constructor")), &want);
    let props = "(private readonly a: boolean, public b: boolean, protected c: boolean, readonly d: boolean, e: boolean)";
    agree(&class(&format!("  constructor{props} {BODY}")), &want);
    agree(&class("  constructor(private readonly db: Store) {}"), &[]);
}

#[test]
fn fields() {
    let arrow = format!("{PARAMS} => {BODY};");
    let want = marks("walker.ts", &format!("export const walkTree = {arrow}\n"));
    assert!(want.contains(&hit("word", "deepValue")));
    agree(&class(&format!("  private walkTree = {arrow}")), &want);
    let plain = "  #count = 0;\n  name?: string;\n  readonly size!: number;\n  static table: Map<string, () => void> = new Map();\n  [key: string]: unknown;\n  shape: { kind: string; deep: { wide: boolean } };\n  last = 1";
    agree(&class(plain), &[]);
    let named = class("  deepCount = 0;\n  #wideName = 1;\n  pathName?: string;");
    let found = marks("walker.ts", &named);
    let words: Vec<&str> = found.iter().map(|mark| mark.1.as_str()).collect();
    assert_eq!(words, ["deepCount", "pathName", "wideName"]);
}

#[test]
fn signatures() {
    let text = format!(
        "export abstract class Walker {{\n  abstract shape(): {{ kind: string }};\n  abstract load(): Promise<{{ kind: string }}>;\n  declare ready: boolean;\n  walk(a: string): void;\n  walk(a: number): void;\n  walk(a: string | number): void {{}}\n  static {{\n    setup();\n  }}\n  walkTree{PARAMS} {BODY}\n}}\n"
    );
    agree(&text, &free());
}

#[test]
fn nesting() {
    let local =
        format!("export function outer(): void {{\n  function walkTree{PARAMS} {BODY}\n}}\n");
    let want = marks("walker.ts", &local);
    assert_eq!(tally(&want, "block"), 3);
    let member = class(&method("walkTree"));
    let inner = format!("export function outer(): void {{\n  {member}}}\n");
    agree(&inner, &want);
    let shallow =
        "  walk(a: boolean, b: boolean): void {\n    if (a) { if (b) { if (a) { go(); } } }\n  }";
    agree(&class(shallow), &[]);
    let deep = "  walk(a: boolean, b: boolean): void {\n    if (a) { if (b) { if (a) { if (b) { go(); } } } }\n  }";
    agree(&class(deep), &[hit("block", "depth over limit")]);
}

#[test]
fn bound() {
    let top = "export function load(app: App): void {}\nexport function save(app: App): void {}\nexport function query(app: App): void {}\nexport function drop(app: App): void {}\n";
    assert_eq!(count(scan("walker.ts", top), "receiver"), 1);
    let bound = "  load(app: App): void {}\n  save(app: App): void {}\n  query(app: App): void {}\n  drop(app: App): void {}";
    agree(&class(bound), &[]);
}

#[test]
fn coverage() {
    agree("export class Walker {}\n", &[]);
    agree(&class("  ;"), &[]);
    let odd = class("  walk(): void {}\n  ~ ~\n  run(): void {}");
    assert!(scan("walker.ts", &odd).contains(&"coverage".to_string()));
    assert!(scan("walker.tsx", &odd).contains(&"coverage".to_string()));
    assert!(scan("walker.svelte", &script(&odd)).contains(&"coverage".to_string()));
}
