type Marks = Vec<(String, String)>;

const SHAPE: &str = "{ outer: { inner: { leaf: { value: 1 } } } }";
const DEEP: &str = "{ if (a) { if (b) { if (a) { go(); } } } }";
const SHALLOW: &str = "{ if (a) { if (b) { go(); } } }";

fn source(path: &str, text: &str) -> grammar::Source {
    grammar::Source {
        path: path.to_string(),
        text: text.to_string(),
    }
}

fn marks(path: &str, text: &str) -> Marks {
    let config = kernel::config::Config {
        file: Default::default(),
    };
    let source = source(path, text);
    let found = kernel::check(&source, &kernel::structure(&source), &config).into_iter();
    let mut found: Marks = found.map(|hit| (hit.law, hit.note)).collect();
    found.sort();
    found
}

fn script(text: &str) -> String {
    format!("<script lang=\"ts\">\n{text}\n</script>\n")
}

fn agree(text: &str, want: &Marks) {
    assert_eq!(marks("emit.ts", text), *want, "{text}");
    assert_eq!(marks("emit.tsx", text), *want, "{text}");
    assert_eq!(marks("emit.svelte", &script(text)), *want, "{text}");
}

fn build(line: &str) -> String {
    format!(
        "export function build(emit: (shape: object) => void, a: boolean, b: boolean): object {{\n  {line}\n  return a;\n}}\n"
    )
}

fn wrap(depth: usize, line: &str) -> String {
    let open = "if (a) { ".repeat(depth);
    let close = " }".repeat(depth);
    format!("{open}{line}{close}")
}

fn blocks(text: &str) -> usize {
    let found = marks("emit.ts", text);
    found.iter().filter(|mark| mark.0 == "block").count()
}

fn scopes(path: &str, text: &str) -> usize {
    fn walk(node: &kernel::Node) -> usize {
        let own = usize::from(node.kind == grammar::Kind::Scope);
        own + node.kids.iter().map(walk).sum::<usize>()
    }
    walk(&kernel::structure(&source(path, text)))
}

fn places(shape: &str) -> Vec<String> {
    [
        format!("emit({shape});"),
        format!("emit(1, {shape});"),
        format!("emit([{shape}, {shape}]);"),
        format!("emit(a ? {shape} : {shape});"),
        format!("emit(a && {shape});"),
        format!("emit(a || {shape});"),
        format!("emit(a ?? {shape});"),
        format!("emit({{ ...{shape} }});"),
        format!("emit({{ default: {shape}, case: {shape} }});"),
        format!("shape = {shape};"),
        format!("return {shape};"),
        format!("throw {shape};"),
    ]
    .to_vec()
}

#[test]
fn place() {
    let decl = build(&format!("const shape = {SHAPE};\n  emit(shape);"));
    let want = marks("emit.ts", &decl);
    assert_eq!(want, Marks::new());
    for line in places(SHAPE) {
        let text = build(&line);
        agree(&text, &want);
        assert_eq!(scopes("emit.ts", &text), scopes("emit.ts", &decl), "{text}");
    }
}

#[test]
fn depth() {
    for line in places(SHAPE) {
        for level in 0..=4 {
            let inside = blocks(&build(&wrap(level, &line)));
            let module = blocks(&format!("{}\n", wrap(level + 1, &line)));
            let bare = blocks(&build(&wrap(level, "go();")));
            assert_eq!(inside, bare, "{line} at {level}");
            assert_eq!(inside, module, "{line} at {level}");
            assert_eq!(inside, usize::from(level == 4), "{line} at {level}");
        }
    }
}

#[test]
fn nested() {
    let deep = marks("emit.ts", &build(&format!("emit(() => {DEEP});")));
    assert_eq!(deep.iter().filter(|mark| mark.0 == "block").count(), 1);
    let shallow = marks("emit.ts", &build(&format!("emit(() => {SHALLOW});")));
    assert_eq!(shallow, Marks::new());
    let forms = [
        "emit({ outer: { run: () => BODY } });",
        "emit({ outer: { run(): void BODY } });",
        "emit({ outer: { run: function (): void BODY } });",
        "emit([{ outer: { run: async () => BODY } }]);",
        "return { outer: { get value(): boolean BODY } };",
        "shape = a ? { outer: { run() BODY } } : {};",
    ];
    for form in forms {
        agree(&build(&form.replace("BODY", DEEP)), &deep);
        agree(&build(&form.replace("BODY", SHALLOW)), &shallow);
    }
}

#[test]
fn real() {
    let deep = "if (a) { if (b) { go(); } }";
    let held = [
        format!("switch (a) {{ case 'a': {{ {deep} }} }}"),
        format!("switch (a) {{ case 'a': case 'b': {{ {deep} }} }}"),
        format!("switch (a) {{ case 'a': go(); break; default: {{ {deep} }} }}"),
        format!("{{ {{ {deep} }} }}"),
        format!("go(); {{ {{ {deep} }} }}"),
        format!("if (a) {{ go(); }} else {{ while (b) {{ {deep} }} }}"),
        format!("for (;;) {{ try {{ {deep} }} catch {{ go(); }} }}"),
        format!("do {{ if (a) {{ {deep} }} }} while (a);"),
        format!("emit(() => {{ if (a) {{ {deep} }} }});"),
    ];
    let one = vec![("block".to_string(), "depth over limit".to_string())];
    for line in held {
        agree(&build(&line), &one);
    }
}

const BODY: &str = "const deepValue = input;
  if (input > 0) { if (input > 1) { if (input > 2) { if (input > 3) { if (input > 4) { return deepValue; } } } } }
  return 0;";
const PARAMS: &str =
    "(first: number, second: number, third: number, fourth: number, fifth: number): number";

fn count(kind: grammar::Kind, text: &str) -> usize {
    fn walk(node: &kernel::Node, kind: grammar::Kind) -> usize {
        let own = usize::from(node.kind == kind);
        own + node.kids.iter().map(|kid| walk(kid, kind)).sum::<usize>()
    }
    walk(&kernel::structure(&source("walk.ts", text)), kind)
}

#[test]
fn forms() {
    let top = marks(
        "walk.ts",
        &format!("export function walkTree(input: number): number {{ {BODY} }}"),
    );
    assert_eq!(top.len(), 4, "{top:?}");
    let bare: Marks = top
        .iter()
        .filter(|mark| mark.1 != "walkTree")
        .cloned()
        .collect();
    let arrows = [
        "register(\"walk\", (input: number): number => { BODY });",
        "app.on(\"walk\", async (input: number) => { BODY });",
        "export const walkers = [(input: number) => { BODY }];",
        "register(\"walk\", function (input: number): number { BODY });",
    ];
    for form in arrows {
        agree(&form.replace("BODY", BODY), &bare);
    }
    let named = [
        "export const walker = { walkTree(input: number): number { BODY }, };",
        "export const walker = { size: 1, async walkTree(input: number): Promise<number> { BODY } };",
        "register({ *walkTree(input: number): Iterable<number> { BODY } });",
        "export const walkTree = function (input: number): number { BODY };",
        "export const walkTree = async function (input: number): Promise<number> { BODY };",
    ];
    for form in named {
        agree(&form.replace("BODY", BODY), &top);
    }
}

#[test]
fn methods() {
    let top = format!("export function walkTree{PARAMS} {{ return first; }}\n");
    let want = marks("walk.ts", &top);
    assert_eq!(want.len(), 2, "{want:?}");
    let method = format!("walkTree{PARAMS} {{ return first; }}");
    let lambda = format!("function{PARAMS} {{ return first; }}");
    let forms = [
        format!("export const walker = {{ {method} }};"),
        format!("export const walker = {{ size: 1, async {method}, other: 2 }};"),
        format!("register({{ outer: {{ {method} }} }});"),
        build(&format!("return {{ {method} }};")),
        build(&format!("const walker = {{ {method} }};\n  emit(walker);")),
        build(&format!("emit(a ? {{ {method} }} : {{}});")),
        format!("export const walkTree = {lambda};"),
        build(&format!("const walkTree = {lambda};\n  emit(walkTree);")),
    ];
    let host = build("go();");
    for text in forms {
        agree(&text, &want);
        let base = if text.contains("build") {
            host.as_str()
        } else {
            ""
        };
        for kind in [grammar::Kind::Receiver, grammar::Kind::Param] {
            let added = count(kind, &text) - count(kind, base);
            assert_eq!(added, count(kind, &top), "{kind} {text}");
        }
    }
}

#[test]
fn coverage() {
    let odd = "class Inner { ~ ~ }";
    for form in [
        format!("register(\"walk\", () => {{ {odd} }});"),
        format!("export const walker = {{ walkTree() {{ {odd} }} }};"),
        format!("export const walkTree = function () {{ {odd} }};"),
        build(&format!("return {{ walkTree() {{ {odd} }} }};")),
    ] {
        for (path, text) in [
            ("walk.ts", form.clone()),
            ("walk.tsx", form.clone()),
            ("walk.svelte", script(&form)),
        ] {
            assert!(
                marks(path, &text).iter().any(|mark| mark.0 == "coverage"),
                "{path} {text}"
            );
        }
    }
}

#[test]
fn signatures() {
    for text in [
        "export interface Api { walkTree(): Base | { extra: number } }\n",
        "export type Api = { walkTree(): { first: number } & { second: number }, readTree(): void };\n",
        "export function host(api: { walkTree(): Base | { extra: number } }): void {}\n",
    ] {
        agree(text, &Marks::new());
        assert_eq!(count(grammar::Kind::Item, text), 1, "{text}");
    }
}
