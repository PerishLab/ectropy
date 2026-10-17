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
