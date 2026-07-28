mod seat;
use seat::*;

#[test]
fn depth() {
    assert!(
        laws("fn f() { if a { for b in c { while d { if e { let x = 1; } } } } }")
            .contains(&"block".to_string())
    );
    assert!(
        !laws("fn f() { if a { for b in c { while d { let x = 1; } } } }")
            .contains(&"block".to_string())
    );
}

#[test]
fn literal() {
    assert!(
        !laws("fn f() { if a { for b in c { let v = S { x: 1 }; } } }")
            .contains(&"block".to_string())
    );
}

#[test]
fn raw() {
    assert!(!laws("fn f() { let text = r#\"{\"#; }").contains(&"coverage".to_string()));
}

#[test]
fn coverage() {
    assert!(
        !laws("use a::{B, C};\npub fn f() -> u32 { 1 }\ntrait T { fn m(&self); }")
            .contains(&"coverage".to_string())
    );
}

#[test]
fn knit() {
    let make = "macro_rules! gate {\n    ($actor:ident) => {\n        pub struct Token {\n            actor: $actor,\n        }\n    };\n}\n";
    assert!(!laws(make).contains(&"coverage".to_string()));
    assert!(!laws("keel_gate::gate!(Actor);").contains(&"coverage".to_string()));
    assert!(!laws(make).contains(&"word".to_string()));
}

#[test]
fn spell() {
    let deep = "macro_rules! deep {\n    () => {\n        fn f() {\n            if a {\n                for b in c {\n                    while d {\n                        let x = 1;\n                    }\n                }\n            }\n        }\n    };\n}\n";
    assert!(laws(deep).contains(&"block".to_string()));
    assert!(laws("macro_rules! read_file { () => { 1 }; }").contains(&"word".to_string()));
    assert!(!laws("macro_rules! read { ($a_b:ident) => { 1 }; }").contains(&"word".to_string()));
}

#[test]
fn comment() {
    assert!(laws("fn f() {}\n// note").contains(&"comment".to_string()));
}

#[test]
fn compound() {
    assert!(laws("fn read_file() {}").contains(&"word".to_string()));
    assert!(laws("fn APIBase() {}").contains(&"word".to_string()));
}

#[test]
fn reference() {
    assert!(!laws("fn read() { write_all(); }").contains(&"word".to_string()));
}

#[test]
fn binding() {
    assert!(laws("fn f() { let bad_name = 1; }").contains(&"word".to_string()));
    assert!(!laws("fn f() { let good = 1; if let Some(x) = y {} }").contains(&"word".to_string()));
}

#[test]
fn field() {
    assert!(laws("struct S { bad_field: u32 }").contains(&"word".to_string()));
    assert!(!laws("struct S { good: u32 }\nenum E { Ok, Bad }").contains(&"word".to_string()));
}

#[test]
fn typescript() {
    let laws = scan(
        "t.ts",
        "export type Shape = { badName: string };\nexport function readFile(options: { dryRun?: boolean } = {}): void { if (options.dryRun) { return; } }\nexport const io = { readFile };",
    );
    assert!(!laws.contains(&"coverage".to_string()));
    assert!(laws.contains(&"word".to_string()));
}

#[test]
fn reexport() {
    let found = scan(
        "t.ts",
        "export { read } from \"./io\";\nexport { read as fetch } from \"./io\";\nexport type { Shape } from \"./form\";\nexport { pull };",
    );
    assert!(!found.contains(&"coverage".to_string()));
    assert!(!found.contains(&"word".to_string()));
    let debt = scan("t.ts", "export { readFile } from \"./io\";");
    assert!(!debt.contains(&"coverage".to_string()));
    assert!(debt.contains(&"word".to_string()));
}

#[test]
fn generic() {
    let found = scan(
        "t.ts",
        "export function kind<M>(): void {}\nexport async function run<T>(fn: () => T): void {}\nexport function family<F extends string, S extends Spec>(name: F, spec: S): void {}",
    );
    assert!(!found.contains(&"coverage".to_string()));
    assert!(!found.contains(&"word".to_string()));
    let debt = scan("t.ts", "export function readFile<T>(): void {}");
    assert!(!debt.contains(&"coverage".to_string()));
    assert!(debt.contains(&"word".to_string()));
}

#[test]
fn typeonly() {
    let found = scan(
        "t.ts",
        "export { type Shape, read };\nexport { type Fault, type Infer, kind };",
    );
    assert!(!found.contains(&"coverage".to_string()));
    assert!(!found.contains(&"word".to_string()));
    let debt = scan("t.ts", "export { type ReadFile };");
    assert!(!debt.contains(&"coverage".to_string()));
    assert!(debt.contains(&"word".to_string()));
}

#[test]
fn ambient() {
    let found = scan(
        "t.ts",
        "declare module \"*.scss\" { const sheet: string; export default sheet; }",
    );
    assert!(!found.contains(&"coverage".to_string()));
}

#[test]
fn tsx() {
    let found = scan(
        "t.tsx",
        "export function App(): unknown { const on = true; return <main className=\"page\"><section>{on && <span>hi</span>}</section><input value={on}/><a.B/><>go</></main>; }",
    );
    assert!(!found.contains(&"coverage".to_string()));
    assert!(!found.contains(&"word".to_string()));
}

fn nest(deep: usize) -> String {
    let open = "<div>".repeat(deep);
    let shut = "</div>".repeat(deep);
    format!("export function App(): unknown {{ return {open}x{shut}; }}")
}

#[test]
fn markup() {
    assert!(scan("t.tsx", &nest(9)).contains(&"markup".to_string()));
    assert!(!scan("t.tsx", &nest(8)).contains(&"markup".to_string()));
}

#[test]
fn axes() {
    let layered = "export function App(): unknown { return <a><b><c><d><e><f><g>{run(() => { if (on) { go(); } })}</g></f></e></d></c></b></a>; }";
    let found = scan("t.tsx", layered);
    assert!(!found.contains(&"block".to_string()));
    assert!(!found.contains(&"markup".to_string()));
    let blocky = "export function App(): unknown { if (a) { if (b) { if (c) { return <div><span>hi</span></div>; } } } }";
    let inner = scan("t.tsx", blocky);
    assert!(!inner.contains(&"block".to_string()));
    assert!(!inner.contains(&"markup".to_string()));
}

#[test]
fn hooks() {
    assert!(!scan("t.ts", "export function useCart(): void {}").contains(&"word".to_string()));
    assert!(scan("t.ts", "export function useUserCart(): void {}").contains(&"word".to_string()));
    assert!(
        !scan("t.tsx", "export const useTheme = () => <div>x</div>;").contains(&"word".to_string())
    );
}

fn territory() -> kernel::config::Config {
    let mut config = config(&[], 4);
    config.file.grant.push(kernel::config::Grant {
        syntax: "style".to_string(),
        paths: vec!["styles/**".to_string()],
    });
    config
}

#[test]
fn style() {
    let attr = "export function App(): unknown { return <div style={{color: on}}>x</div>; }";
    assert_eq!(hits("src/App.tsx", attr, &territory()), 1);
    assert_eq!(hits("styles/App.tsx", attr, &territory()), 0);
    assert_eq!(count(scan("src/App.tsx", attr), "grant"), 0);
}

#[test]
fn sheet() {
    let tagged = "export function App(): unknown { return <style>{code}</style>; }";
    assert_eq!(hits("src/App.tsx", tagged, &territory()), 1);
    assert_eq!(count(scan("src/App.tsx", tagged), "grant"), 0);
}

#[test]
fn weave() {
    let pull = "import sheet from \"./app.scss\";\nexport function go(): void {}";
    assert_eq!(hits("src/app.ts", pull, &territory()), 1);
    assert_eq!(hits("styles/app.ts", pull, &territory()), 0);
    let plain = "import { read } from \"./io\";\nexport function go(): void {}";
    assert_eq!(hits("src/app.ts", plain, &territory()), 0);
}

#[test]
fn scss() {
    let card = ".card { color: red; }";
    assert_eq!(hits("src/card.scss", card, &territory()), 1);
    assert_eq!(hits("styles/card.scss", card, &territory()), 0);
    assert_eq!(count(scan("src/card.scss", card), "grant"), 0);
    assert!(!scan("src/card.scss", card).contains(&"coverage".to_string()));
}

#[test]
fn pathed() {
    assert!(
        !laws("fn f() { let some_crate::Kind::Held { x } = y else { return; }; }")
            .contains(&"word".to_string())
    );
    assert!(laws("fn f() { let held_name = 1; }").contains(&"word".to_string()));
}
