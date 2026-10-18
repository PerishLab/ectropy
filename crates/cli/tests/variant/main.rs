mod rust;
mod shift;
mod statement;
mod svelte;
mod ts;

type Marks = Vec<(String, String)>;

const FIELDS: &str = "
    /// The listening port.
    #[arg(long)]
    alpha_beta: u8,
    pub gamma_delta: Vec<Option<HashMap<String, Vec<u8>>>>,
    #[serde(rename = \"epsilon\")]
    #[arg(long, default_value_t = 20)]
    epsilon_zeta: u16,
";

fn source(text: &str) -> grammar::Source {
    grammar::Source {
        path: "lib.rs".to_string(),
        text: text.to_string(),
    }
}

fn marks(text: &str) -> Marks {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    let source = source(text);
    let found = kernel::check(&source, &kernel::structure(&source), &config).into_iter();
    let mut found: Marks = found.map(|hit| (hit.law, hit.note)).collect();
    found.sort();
    found
}

fn kinds(text: &str, kind: grammar::Kind) -> usize {
    fn walk(node: &kernel::Node, kind: grammar::Kind) -> usize {
        let own = usize::from(node.kind == kind);
        own + node.kids.iter().map(|kid| walk(kid, kind)).sum::<usize>()
    }
    walk(&kernel::structure(&source(text)), kind)
}

fn host(body: &str) -> String {
    format!("pub fn host(flag: bool) -> u8 {{\n{body}\n1\n}}\n")
}

#[test]
fn words() {
    let named = marks(&format!("pub struct Circle {{{FIELDS}}}"));
    assert_eq!(named.len(), 4, "{named:?}");
    for text in [
        format!("pub enum Shape {{ Circle {{{FIELDS}}} }}"),
        format!("pub enum Shape {{ Plain, Circle {{{FIELDS}}}, Pair(u8, u8) }}"),
        format!("enum Shape {{ #[command(about = \"x\")] Circle {{{FIELDS}}} = 1, }}"),
        host(&format!("enum Shape {{ Circle {{{FIELDS}}} }}")),
    ] {
        assert_eq!(marks(&text), named, "{text}");
    }
}

#[test]
fn records() {
    let named = format!("pub struct Circle {{{FIELDS}}}");
    let variant = format!("pub enum Shape {{ Circle {{{FIELDS}}}, Pair(u8, u8), Plain = 3 }}");
    for kind in [
        grammar::Kind::Record,
        grammar::Kind::Label,
        grammar::Kind::Field,
    ] {
        assert_eq!(kinds(&variant, kind), kinds(&named, kind), "{variant}");
    }
    assert_eq!(kinds(&variant, grammar::Kind::Record), 1);
    assert_eq!(kinds(&variant, grammar::Kind::Field), 3);
}

#[test]
fn levels() {
    let deep = "if flag { if flag { if flag { go(); } } }";
    let enumed = host(&format!("enum Shape {{ Circle {{{FIELDS}}} }}\n{deep}"));
    assert_eq!(kinds(&enumed, grammar::Kind::Scope), 4);
    assert_eq!(
        marks(&enumed),
        marks(&host(&format!("struct Circle {{{FIELDS}}}\n{deep}")))
    );
}

#[test]
fn coverage() {
    let refused = vec![("coverage".to_string(), "unparsed region".to_string())];
    for item in [
        "enum Shape { Circle { 1 } }",
        "enum Shape { Circle { alpha: u8, 2 } }",
        "enum Shape { Circle { alpha: u8 }, Square { ? } }",
    ] {
        assert_eq!(marks(item), refused, "{item}");
        assert_eq!(marks(&host(item)), refused, "{item}");
    }
}

#[test]
fn others() {
    let text = "pub enum Shape { Plain, Pair(u8, Vec<u8>), Fixed = 3, Wide(u8, u8, u8) = 4 }";
    assert_eq!(marks(text), Marks::new());
    assert_eq!(kinds(text, grammar::Kind::Word), 5);
    assert_eq!(kinds(text, grammar::Kind::Record), 0);
    assert_eq!(kinds(text, grammar::Kind::Loose), 0);
}

const SPELLINGS: [&str; 15] = [
    "a: number, b: number, c: number, d: number, e: number",
    "a, b, c, d, e",
    "a = 1, b = 2, c = 3, d = 4, e = 5",
    "a?: number, b = 2, c, d: string = \"x\", e",
    "{ a }: { a: number }, b: number, c: number, d: number, e: number",
    "{ a, b }, c, d, e, f",
    "[a, b]: number[], c, d, e, f",
    "a: number, { b }: { b: number }, c: number, d: number, e: number",
    "{ a } = { a: 1 }, b, c, d, e",
    "a, b, c, d, ...e",
    "a: number, b: number, c: number, d: number, ...e: number[]",
    "a: Map<string, number>, b, c, d, e",
    "a, b: Record<string, number>, c, d, e",
    "a: Map<string, Array<Record<string, number>>>, b, c, d, e",
    "a: Map<string, (x: number, y: number) => void>, b, c, d, e",
];

const HOSTS: [&str; 12] = [
    "export function host(LIST) { return 1; }",
    "export class Host { run(LIST) { return 1; } }",
    "export const host = function (LIST) { return 1; };",
    "export const host = (LIST) => { return 1; };",
    "export const host = async <T,>(LIST): Promise<T> => { return go(); };",
    "export class Host { run = (LIST) => { return 1; }; }",
    "register((LIST) => { return 1; });",
    "register((LIST) => a + b);",
    "export const shape = { run: (LIST) => a, walk: (LIST) => { return a; } };",
    "export function host(): void { run((LIST) => a); }",
    "export function host(): void { const run = (LIST) => { return a; }; go(run); }",
    "export function host() { return (LIST) => a; }",
];

fn read(path: &str, text: &str) -> Marks {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    let source = grammar::Source {
        path: path.to_string(),
        text: text.to_string(),
    };
    let found = kernel::check(&source, &kernel::structure(&source), &config).into_iter();
    let mut found: Marks = found.map(|hit| (hit.law, hit.note)).collect();
    found.sort();
    found
}

fn agree(form: &str, want: &Marks) {
    assert_eq!(read("host.ts", form), *want, "{form}");
    assert_eq!(read("host.tsx", form), *want, "{form}");
    let script = format!("<script lang=\"ts\">\n{form}\n</script>\n");
    assert_eq!(read("host.svelte", &script), *want, "{form}");
}

fn over(times: usize) -> Marks {
    let hit = ("param".to_string(), "5 parameters over limit 4".to_string());
    std::iter::repeat_n(hit, times).collect()
}

#[test]
fn spellings() {
    for list in SPELLINGS {
        agree(&format!("export function host({list}) {{}}\n"), &over(1));
    }
    for list in ["a, b, c, d", "{ a, b }, [c], d = 1, ...e", "...e"] {
        agree(&format!("export function host({list}) {{}}\n"), &over(0));
    }
    let script = "<script>\n  function host(a, b, c, d, e) { return a; }\n</script>\n";
    assert_eq!(read("host.svelte", script), over(1));
}

#[test]
fn arrows() {
    for list in [
        "a, b, c, d, e",
        "a: T, b = 2, { c }: T, [d]: T[], ...e: T[]",
    ] {
        for host in HOSTS {
            let text = host.replace("LIST", list);
            agree(&text, &over(text.matches("=>").count().max(1)));
        }
    }
    for text in [
        "export const View = (a, b, c, d, e) => <p>x</p>;",
        "export function View() { return <ul>{items.map((a, b, c, d, e) => <li>x</li>)}</ul>; }",
        "export function View() { return <List render={(a, b, c, d, e) => <p>x</p>} />; }",
    ] {
        assert_eq!(read("view.tsx", text), over(1), "{text}");
    }
    let bare = "register(item => { return item; }, async item => item, { run: item => item });";
    let seats = ["host.ts", "host.tsx"].map(|path| {
        let source = grammar::Source {
            path: path.to_string(),
            text: bare.to_string(),
        };
        let mut seats = 0;
        let mut stack = vec![kernel::structure(&source)];
        while let Some(node) = stack.pop() {
            seats += usize::from(node.kind == grammar::Kind::Receiver);
            stack.extend(node.kids);
        }
        seats
    });
    assert_eq!(seats, [3, 3]);
}

const TYPES: [&str; 13] = [
    "export type Handler = FN;",
    "export type Handler = { run: FN; walk(x: X): void };",
    "export type Handler = Map<string, FN> | null;",
    "export type Handler = (x: X) => { run: FN };",
    "export interface Host { run: FN; walk(cb: FN): void; cfg: { go: FN } }",
    "export function host(cb: { run: FN }, go: (inner: FN) => void): void { cb.run(go); }",
    "export function host(): { run: FN } { return make(); }",
    "export const host = (cb: Map<K, FN>): (FN) => { return cb; };",
    "export class Host { run: FN; cfg: { go: FN } = make(); }",
    "export const host: { run: FN } = make();",
    "export function host(): void { type H = FN; go(make() as FN, run satisfies { go: FN }); }",
    "declare module \"x\" { export type H = FN; }",
    "export const host = make<FN>(run) as { run: FN };",
];

#[test]
fn types() {
    for list in [
        "a: A, b: B, c: C, d: D, e: E",
        "a, b, c, d, e",
        "_event: Event",
    ] {
        for host in TYPES {
            let form = host.replace("FN", &format!("({list}) => void"));
            agree(&form, &read("host.ts", &host.replace("FN", "Handler")));
        }
    }
}

const BODIES: [&str; 9] = [
    "() => sync()",
    "value => value.x",
    "(value: number): number => value + 1",
    "async (value) => await sync(value)",
    "<T,>(value: T) => value",
    "(a) => (b) => a + b",
    "(value) => ({ value })",
    "(value) =>\n    sync(value)",
    "(a, b) => [a, !b]",
];

const SEATS: [&str; 6] = [
    "export const run = BODY;",
    "let run = BODY;",
    "const run: (a: A, b: B, c: C, d: D, e: E) => (f: F) => R = BODY, size = 1;",
    "export const run = BODY\nexport const size = 1;",
    "export class Host { run = BODY; }",
    "export class Host {\n  static run = BODY\n}",
];

#[test]
fn bodies() {
    for seat in SEATS {
        for body in BODIES {
            agree(&seat.replace("BODY", body), &Marks::new());
        }
        for body in ["(p) => <a>x</a>", "(p) => (<a>x</a>)", "() =>\n  <b>y</b>"] {
            let text = seat.replace("BODY", body);
            assert_eq!(read("view.tsx", &text), Marks::new(), "{text}");
        }
    }
}

#[test]
fn evidence() {
    let deep = "go(function (a) { const alphaBeta = 1; if (a === 1) { go(); } else if (a === 2) { go(); } if (a) { if (a) { if (a) { if (a) { go(); } } } } })";
    let tags = "(<a><b><c><d><e><f><g><h><i><j>x</j></i></h></g></f></e></d></c></b></a>)";
    let want = read("view.tsx", &format!("function host() {{ go{tags}; }}"));
    assert_eq!(want.len(), 2, "{want:?}");
    for seat in SEATS {
        let plain = read("host.ts", &seat.replace("BODY", deep));
        assert_eq!(plain.len(), 3, "{plain:?}");
        agree(&seat.replace("BODY", &format!("(v) => {deep}")), &plain);
        let odd = seat.replace("BODY", "(v) => go(function () { class Inner { ~ } })");
        let odd = read("host.ts", &odd);
        assert!(odd.iter().any(|hit| hit.0 == "coverage"), "{odd:?}");
        agree(&seat.replace("BODY", "(a, b, c, d, e) => a"), &over(1));
        let text = seat.replace("BODY", &format!("(p) => {tags}"));
        assert_eq!(read("view.tsx", &text), want, "{text}");
    }
}
