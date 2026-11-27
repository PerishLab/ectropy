use super::{Marks, agree, over, read};

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

#[test]
fn ambient() {
    let global = "export {};\ndeclare global {\n  interface SymbolConstructor {\n    readonly plectron: symbol;\n  }\n}\n";
    agree(global, &Marks::new());
    assert_eq!(read("global.d.ts", global), Marks::new());
    let member = "declare global {\n  interface SymbolConstructor {\n    readonly plectron: symbol;\n    readonly fooBar: string;\n  }\n}\n";
    agree(member, &words(&["fooBar"]));
    assert_eq!(read("global.d.ts", member), words(&["fooBar"]));
    let module = "declare module \"ext\" {\n  interface ExternalApi<T> extends BaseApi {\n    readonly ok: T;\n    readonly fooBar: string;\n    getValue(): void;\n  }\n}\n";
    agree(module, &words(&["fooBar", "getValue"]));
    let exported = "declare module \"ext\" {\n  export interface ExternalApi {\n    readonly ok: number;\n    readonly fooBar: string;\n  }\n}\n";
    agree(exported, &words(&["fooBar"]));
    let top = "export interface TopLevel {\n  readonly ok: number;\n}\n";
    assert_eq!(read("types.d.ts", top), words(&["TopLevel"]));
    assert_eq!(
        read("types.d.ts", "interface TopLevel { ok: number }\n"),
        words(&["TopLevel"])
    );
    let direct = "declare interface ExternalHost<T> extends BaseApi {\n  readonly ok: T;\n  \"appendChild\"(node: Node): Node;\n  readonly \"healthUrl\": string | null;\n}\n";
    agree(direct, &Marks::new());
    assert_eq!(read("types.d.ts", direct), Marks::new());
    let exported = "export declare interface ExternalHost {\n  readonly ok: number;\n  \"removeChild\"(node: Node): Node;\n};\n";
    agree(exported, &Marks::new());
    assert_eq!(read("types.d.ts", exported), Marks::new());
    let unquoted = "declare interface ExternalHost {\n  readonly ok: number;\n  fooBar: string;\n  appendChild(node: Node): Node;\n}\n";
    agree(unquoted, &words(&["appendChild", "fooBar"]));
    agree(
        "export declare interface ExternalHost { appendChild(node: Node): Node }\n",
        &words(&["appendChild"]),
    );
    for broken in [
        "declare interface { ok: number }\n",
        "declare interface ExternalHost\n",
        "export declare interface { ok: number }\n",
    ] {
        let found = read("host.ts", broken);
        assert!(
            found.iter().any(|hit| hit.0 == "coverage"),
            "{broken}: {found:?}"
        );
    }
}

#[test]
fn callbacks() {
    for clean in [
        "export const plugin = { name: \"stamp\", \"resolveId\"(id: string) { return id; }, \"handleHotUpdate\": ({ server, modules }: Ctx) => modules };\n",
        "export function wrap<T extends object>(value: T): T { return new Proxy(value, { get(t, k) { return Reflect.get(t, k); }, \"ownKeys\"(t) { return Reflect.ownKeys(t); }, \"defineProperty\": (t, k, d) => Reflect.defineProperty(t, k, d), \"deleteProperty\": function (t, k) { return Reflect.deleteProperty(t, k); } }); }\n",
        "export const dom = { 'appendChild'(node: Node) { return node; }, async \"removeChild\"(node: Node) { return node; } };\n",
    ] {
        agree(clean, &Marks::new());
    }
    agree(
        "export const plugin = { resolveId(id: string) { return id; }, ownKeys: (t: object) => Reflect.ownKeys(t) };\n",
        &words(&["ownKeys", "resolveId"]),
    );
    let mut strict = over(1);
    strict.extend(words(&["badLocal"]));
    strict.sort();
    for checked in [
        "export const plugin = { \"resolveId\"(a: number, b: number, c: number, d: number, e: number) { const badLocal = a; return badLocal; } };\n",
        "export const proxy = { \"ownKeys\": (a: number, b: number, c: number, d: number, e: number) => { const badLocal = a; return badLocal; } };\n",
    ] {
        agree(checked, &strict);
    }
}

#[test]
fn overrides() {
    for clean in [
        "export class Sub extends Base {\n  override doSomething(x: number): void { go(x); }\n}\n",
        "export class Sub extends Base {\n  public override async fetchItem<T>(x: T): Promise<T> { return x; }\n}\n",
        "export class Sub extends Base {\n  override get fullName(): string { return \"\"; }\n  override set fullName(v: string) { go(v); }\n  override *genItems() { yield 1; }\n}\n",
        "export abstract class Sub extends Base {\n  abstract override doSomething(): void;\n}\n",
        "export class Sub {\n  override(x: number): void { go(x); }\n}\n",
    ] {
        agree(clean, &Marks::new());
    }
    let body = "export class Sub extends Base {\n  protected override doSomething(a: number, b: number, c: number, d: number, e: number): void {\n    const badLocal = a;\n    go(badLocal);\n  }\n}\n";
    let mut want = over(1);
    want.extend(words(&["badLocal"]));
    want.sort();
    agree(body, &want);
    agree(
        "export class Sub extends Base {\n  doSomething(): void {}\n}\n",
        &words(&["doSomething"]),
    );
    agree(
        "export class Sub extends Base {\n  override doSomething = (): void => {};\n}\n",
        &words(&["doSomething"]),
    );
    agree(
        "export const obj = { setDepth() {}, getWidth: () => 1 };\n",
        &words(&["getWidth", "setDepth"]),
    );
}
