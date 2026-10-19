use crate::{Marks, agree, marks, read};

fn hit(subject: &str, times: usize) -> Marks {
    std::iter::repeat_n(("dispatch".to_string(), subject.to_string()), times).collect()
}

fn script(first: &str, next: &str) -> String {
    format!(
        "export function host(s, m, k, g) {{\n  if ({first}) {{ return 1; }}\n  if ({next}) {{ return 2; }}\n  return 0;\n}}\n"
    )
}

#[test]
fn paths() {
    for (first, next, subject) in [
        ("s.kind === \"a\"", "s.kind === \"b\"", "s.kind"),
        ("s.a.b === 1", "s.a.b === 2", "s.a.b"),
        ("this.kind === 'a'", "this.kind === 'b'", "this.kind"),
    ] {
        agree(&script(first, next), &hit(subject, 1));
    }
    let chain =
        "export function host(s) { if (s.kind === 1) go(); else if (s.kind === 2) go(); }\n";
    agree(chain, &hit("s.kind", 1));
    let page = "<script lang=\"ts\">\n  let s = $props();\n</script>\n{#if s.mode === \"a\"}<p>a</p>{:else if s.mode === \"b\"}<p>b</p>{/if}\n";
    assert_eq!(read("host.svelte", page), hit("s.mode", 1));
}

#[test]
fn shifts() {
    for (first, next) in [
        ("s.a?.b === 1", "s.a?.b === 2"),
        ("m[k] === 1", "m[k] === 2"),
        ("g().b === 1", "g().b === 2"),
        ("s.a!.b === 1", "s.a!.b === 2"),
        ("s.kind() === 1", "s.kind() === 2"),
        ("(s.kind) === 1", "(s.kind) === 2"),
        ("s.a === 1", "s.b === 2"),
        ("s.a === 1", "s.a.b === 2"),
        ("s.kind === 1", "s.kind === 1"),
        ("k === 0", "k === 0"),
    ] {
        agree(&script(first, next), &Marks::new());
    }
}

#[test]
fn fields() {
    let probe = |first: &str, next: &str| {
        marks(&format!(
            "pub fn host(s: &S) -> u8 {{\n    if {first} {{ return 1; }}\n    if {next} {{ return 2; }}\n    0\n}}\n"
        ))
    };
    assert_eq!(
        probe("self.typed == \"/exit\"", "self.typed == \"/reload\""),
        hit("self.typed", 1)
    );
    assert_eq!(probe("s.kind == b'a'", "s.kind == b'b'"), hit("s.kind", 1));
    assert_eq!(probe("s.a.b == 1", "s.a.b == 2"), hit("s.a.b", 1));
    let chain = marks(
        "pub fn host(s: &S) -> u8 { if s.kind == 1 { 1 } else if s.kind == 2 { 2 } else { 0 } }",
    );
    assert_eq!(chain, hit("s.kind", 1));
    for (first, next) in [
        ("s.op == Op::A", "s.op == Op::B"),
        ("s.t.0 == 1", "s.t.0 == 2"),
        ("s.op() == 1", "s.op() == 2"),
        ("*s.op == 1", "*s.op == 2"),
        ("(s.op) == 1", "(s.op) == 2"),
        ("s.kind == 0", "s.kind == 0"),
    ] {
        assert_eq!(probe(first, next), Marks::new(), "{first}");
    }
}

#[test]
fn keys() {
    for (first, next) in [
        ("k == 0", "k === 0"),
        ("k ===0", "k === 0"),
        ("k === 0 ", "k === 0"),
        ("s.kind === \"a\"", "s.kind===\"a\""),
    ] {
        agree(&script(first, next), &Marks::new());
    }
    for (first, next) in [
        ("k === \"a b\"", "k === \"ab\""),
        ("k === \"a:b)\"", "k === \"ab\""),
        ("k === 0", "k === 0x0"),
    ] {
        agree(&script(first, next), &hit("k", 1));
    }
    let rust =
        "pub fn host(s: &S) -> u8 { if s.kind==0 { 1 } else if s.kind == 0 { 2 } else { 0 } }";
    assert_eq!(marks(rust), Marks::new());
}
