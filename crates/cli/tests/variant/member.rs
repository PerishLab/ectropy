use super::{Marks, agree};

fn words(names: &[&str]) -> Marks {
    let word = |name: &&str| ("word".to_string(), name.to_string());
    names.iter().map(word).collect()
}

fn seats(text: &str) -> usize {
    let source = grammar::Source {
        path: "host.ts".to_string(),
        text: text.to_string(),
    };
    let mut seats = 0;
    let mut stack = vec![kernel::structure(&source)];
    while let Some(node) = stack.pop() {
        let kind = node.kind;
        seats += usize::from(kind == grammar::Kind::Receiver || kind == grammar::Kind::Param);
        stack.extend(node.kids);
    }
    seats
}

const VALUES: [&str; 10] = [
    "export const t = { setDepth: () => {} };",
    "export const t = { setDepth: (d: number) => d };",
    "export const t = { setDepth: function () {} };",
    "export function make() { return { setDepth: () => {} }; }",
    "export function make() { const t = { setDepth: () => {} }; return t; }",
    "export const t = { size: 1, setDepth: async () => {} };",
    "export const t = { setDepth: d => d };",
    "export const t = { setDepth: <T,>(x: T) => x };",
    "register({ size: 1, setDepth: function* () {} });",
    "export function make(p: boolean) {\n  return p ? {\n    size: 1,\n    setDepth: () => 1,\n  } : {};\n}",
];

const MEMBERS: [&str; 12] = [
    "export interface T { setDepth: number }",
    "export interface T { setDepth(depth: number): void }",
    "export interface T { setDepth: (depth: number) => void }",
    "export type T = { setDepth: number };",
    "export type T = { setDepth(depth: number): void };",
    "export type T = { setDepth: (depth: number) => void };",
    "export interface T {\n  size: number\n  setDepth(depth: number): void\n}",
    "export type T = {\n  size: number\n  setDepth: number\n};",
    "export interface T { setDepth<D>(d: D): void }",
    "export interface T { get setDepth(): number }",
    "export function f(api: { setDepth: number }): void { go(api); }",
    "export const x = make<{ setDepth: number }>();",
];

const USES: [&str; 15] = [
    "export const t = { setDepth: 1 };",
    "export const t = { setDepth: ref };",
    "export const t = { followEnd: controller.followEnd };",
    "export const t = { setDepth: make(() => 1) };",
    "export const t = { \"setDepth\": () => {} };",
    "export const t = { [setDepth]: () => {} };",
    "export const t = { setDepth };",
    "export function f({ setDepth }: Api): void { go(setDepth); }",
    "export function f(api: Api): void { const { setDepth: depth } = api; go(depth); }",
    "export function f({ setDepth = () => 1 }: Api): void { go(setDepth); }",
    "export interface T { \"setDepth\": number; [setDepth]: number }",
    "export interface T { [keyName: string]: unknown }",
    "export type M = { readonly [KeyName in Keys]?: number };",
    "export interface T {\n  size: A | MapLike<string>\n  other: Base<{ run: number }>\n}",
    "export interface T {\n  size: A extends B ? FooBar : C extends D ? EeFf : GgHh\n  other: ReadonlyArray<number>\n  (depthValue: number): void\n  new (depthValue: number): T\n}",
];

#[test]
fn values() {
    let want = words(&["setDepth"]);
    agree("export const t = { setDepth() {} };", &want);
    for text in VALUES {
        agree(text, &want);
    }
}

#[test]
fn members() {
    let want = words(&["setDepth"]);
    agree(
        "export abstract class A { abstract setDepth(): void; }",
        &want,
    );
    for text in MEMBERS {
        agree(text, &want);
    }
    let both = words(&["setDepth", "toggleLayer"]);
    agree(
        "export class A { setDepth: number = 1; toggleLayer!: string; }",
        &both,
    );
    agree(
        "export interface T { setDepth?: number; readonly toggleLayer: string }",
        &both,
    );
    agree(
        "export interface T {\n  setDepth?(): void\n  readonly toggleLayer: string\n}",
        &both,
    );
}

#[test]
fn uses() {
    for text in USES {
        agree(text, &Marks::new());
    }
}

#[test]
fn signatures() {
    let list = "a: A, b: B, c: C, d: D, e: E";
    for text in [
        format!("export interface T {{ setDepth({list}): void }}"),
        format!("export type T = {{ setDepth({list}): void, run: ({list}) => void }};"),
        format!("export interface T {{\n  setDepth<V>({list}): V\n  run({list}): void\n}}"),
    ] {
        agree(&text, &words(&["setDepth"]));
        assert_eq!(seats(&text), 0, "{text}");
    }
    let walk = words(&["walkTree"]);
    agree(
        "export interface Api { walkTree(): Base | { extra: number } }\n",
        &walk,
    );
    agree(
        "export function host(api: { walkTree(): Base | { extra: number } }): void {}\n",
        &walk,
    );
    let both = words(&["readTree", "walkTree"]);
    let text = "export type Api = { walkTree(): { first: number } & { second: number }, readTree(): void };\n";
    agree(text, &both);
}
