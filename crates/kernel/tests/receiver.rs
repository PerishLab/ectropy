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
        "4 functions share app: load, save, query, delete; see: ectropy cookbook receiver"
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
        "_plan is accepted but unused; see: ectropy cookbook burr"
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
