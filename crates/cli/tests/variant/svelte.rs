use super::{Marks, over, read};

const BODY: &str = "const alphaBeta = 1; if (a === 1) { return 1; } else if (a === 2) { return 2; } if (a) { if (a) { if (a) { if (a) { return alphaBeta; } } } } return 0;";

const LADDER: &str = "if (a === 1) { go(); } else if (a === 2) { go(); } else { go(); }";

fn page(script: &str, template: &str) -> String {
    format!("<script lang=\"ts\">\n  let a = $state(0);\n  {script}\n</script>\n{template}\n")
}

fn body() -> Marks {
    let want = read(
        "host.svelte",
        &page(&format!("function go() {{ {BODY} }}"), ""),
    );
    assert_eq!(want.len(), 3, "{want:?}");
    want
}

fn dispatch(times: usize) -> Marks {
    let hit = ("dispatch".to_string(), "a".to_string());
    std::iter::repeat_n(hit, times).collect()
}

#[test]
fn handlers() {
    let want = body();
    for template in [
        "<button onclick={() => { BODY }}>go</button>",
        "<button on:click={() => { BODY }}>go</button>",
        "<button onclick=\"{() => { BODY }}\">go</button>",
        "<button onclick='{() => { BODY }}'>go</button>",
        "<button on:click=\"{() => { BODY }}\">go</button>",
        "<button onclick=\"{() => { go('}'); go(\"x\"); BODY }}\">go</button>",
        "<button onclick={function () { BODY }}>go</button>",
        "<Card onpick={() => { BODY }} />",
        "<p>{() => { BODY }}</p>",
        "{() => { BODY }}",
    ] {
        let text = page("", &template.replace("BODY", BODY));
        assert_eq!(read("host.svelte", &text), want, "{text}");
    }
    let held = page(
        "",
        &format!(
            "{{#each [1] as b}}{{@const pick = () => {{ {BODY} }}}}<p>{{pick()}}</p>{{/each}}"
        ),
    );
    let called = page(
        "",
        &format!(
            "{{#each [1] as b}}{{@const pick = run(() => {{ {BODY} }})}}<p>{{pick}}</p>{{/each}}"
        ),
    );
    assert_eq!(
        read("host.svelte", &held),
        read("host.svelte", &called),
        "{held}"
    );
    assert_eq!(read("host.svelte", &called).len(), 4);
}

#[test]
fn values() {
    let five = "(a, b, c, d, e) => go(a)";
    for template in [
        format!("<button onclick={{{five}}}>go</button>"),
        format!("<button onclick=\"{{{five}}}\">go</button>"),
        format!("<p>{{{five}}}</p>"),
        format!("<button onclick={{run({five})}}>go</button>"),
    ] {
        assert_eq!(
            read("host.svelte", &page("", &template)),
            over(1),
            "{template}"
        );
    }
    for template in [
        "<p>{(f as (a: number, b: number, c: number, d: number, e: number) => void)}</p>",
        "<p title=\"{a} b\">t</p><p title=\"{a}{a}\">t</p>",
        "{#each items as { alphaBeta, gamma }, i (alphaBeta)}<p>{gamma}</p>{/each}",
    ] {
        assert_eq!(
            read("host.svelte", &page("", template)),
            over(0),
            "{template}"
        );
    }
    let quoted = format!(
        "export function View(): JSX.Element {{ return <b title=\"{{() => {{ {BODY} }}}}\">x</b>; }}\n"
    );
    assert_eq!(read("host.tsx", &quoted), over(0), "{quoted}");
}

#[test]
fn reactive() {
    let want = read(
        "host.svelte",
        &page(&format!("$effect(() => {{ {BODY} }});"), ""),
    );
    assert_eq!(want, body());
    assert_eq!(
        read("host.svelte", &page(&format!("$: {{ {BODY} }}"), "")),
        want
    );
    assert_eq!(
        read("host.svelte", &page(&format!("$: {{ {BODY} }};"), "")),
        want
    );
    let plain =
        "$: {}\n  $: doubled = a * 2;\n  $: if (a === 1) { go(); } else if (a === 2) { go(); }";
    assert_eq!(read("host.svelte", &page(plain, "")), dispatch(1));
}

#[test]
fn ladders() {
    let script = read(
        "host.svelte",
        &page(&format!("function pick() {{ {LADDER} }}"), ""),
    );
    assert_eq!(script, dispatch(1));
    for template in [
        "{#if a === 1}<p>one</p>{:else if a === 2}<p>two</p>{:else}<p>none</p>{/if}",
        "{#if a === 1}<p>one</p>{:else if a === 2}<p>two</p>{/if}",
        "{#if a === 1}{:else if a === 2}{/if}",
        "<div><span>{#if a == 1}<p>one</p>\n{:else if a == 2}<p>two</p>{/if}</span></div>",
        "{#each [1] as b}{#if a === 1}<p>{b}</p>{:else if a === 2}<p>two</p>{/if}{/each}",
    ] {
        assert_eq!(
            read("host.svelte", &page("", template)),
            script,
            "{template}"
        );
    }
    let three =
        "if (a === 'x') { go(); } else if (a === 'y') { go(); } else if (a === `z`) { go(); }";
    let want = read(
        "host.svelte",
        &page(&format!("function pick() {{ {three} }}"), ""),
    );
    assert_eq!(want, dispatch(2));
    let template =
        "{#if a === 'x'}<p>x</p>{:else if a === 'y'}<p>y</p>{:else if a === `z`}<p>z</p>{/if}";
    assert_eq!(read("host.svelte", &page("", template)), want);
    for template in [
        "{#if a > 1}<p>one</p>{:else if a > 2}<p>two</p>{/if}",
        "{#if a === 1 && b}<p>one</p>{:else if a === 2}<p>two</p>{/if}",
        "{#if a === 1}<p>one</p>{:else if b === 2}<p>two</p>{/if}",
        "{#if a === 1}<p>one</p>{:else}{#if a === 2}<p>two</p>{/if}{/if}",
    ] {
        assert_eq!(
            read("host.svelte", &page("", template)),
            over(0),
            "{template}"
        );
    }
}

#[test]
fn branches() {
    let block = ("block".to_string(), "depth over limit".to_string());
    let nest = "{#if a}{#if a}{#if a}{#if a}{#if a}<p>x</p>INNER{/if}{/if}{/if}{/if}{/if}";
    let plain = read("host.svelte", &page("", &nest.replace("INNER", "")));
    assert_eq!(plain, vec![block.clone()]);
    let forked = read(
        "host.svelte",
        &page("", &nest.replace("INNER", "{:else}<p>y</p>")),
    );
    assert_eq!(forked, vec![block.clone(), block.clone()]);
    let each = "{#each x as y}{#each x as y}{#each x as y}{#each x as y}{#each x as y}<p>x</p>{:else}<p>y</p>{/each}{/each}{/each}{/each}{/each}";
    assert_eq!(read("host.svelte", &page("", each)), vec![block]);
    let head = page(
        "",
        &format!("{{#if [1].some((a) => {{ {BODY} }})}}<p>x</p>{{/if}}"),
    );
    let text = page("", &format!("<p>{{[1].some((a) => {{ {BODY} }})}}</p>"));
    assert_eq!(
        read("host.svelte", &head),
        read("host.svelte", &text),
        "{head}"
    );
    let deep = "if (v) { if (v) { if (v) { return v; } } } return 0;";
    let headed = page(
        "",
        &format!("{{#each [1].map(function (v) {{ {deep} }}) as x}}<p>{{x}}</p>{{/each}}"),
    );
    let shown = page("", &format!("{{[1].map(function (v) {{ {deep} }})}}"));
    assert_eq!(
        read("host.svelte", &headed),
        read("host.svelte", &shown),
        "{headed}"
    );
    let awaited = page(
        "",
        &format!(
            "{{#await Promise.resolve([1].map(function (v) {{ {deep} }})) then x}}<p>{{x}}</p>{{/await}}"
        ),
    );
    assert_eq!(
        read("host.svelte", &awaited),
        read("host.svelte", &shown),
        "{awaited}"
    );
    let keyed = page(
        "",
        &format!("{{#key [1].map(function (v) {{ {deep} }})}}<p>x</p>{{/key}}"),
    );
    assert_eq!(
        read("host.svelte", &keyed),
        read("host.svelte", &shown),
        "{keyed}"
    );
}

#[test]
fn snippets() {
    let want = read(
        "host.svelte",
        &page(
            "function gammaDelta(a: number, b: number, c: number, d: number, e: number) { return a; }",
            "",
        ),
    );
    assert_eq!(want.len(), 2, "{want:?}");
    for list in [
        "a: number, b: number, c: number, d: number, e: number",
        "a, b, c, { d }, ...e",
        "a, b = 1, [c], d: string, e?: number",
    ] {
        let template = format!("{{#snippet gammaDelta({list})}}<p>{{a}}</p>{{/snippet}}");
        assert_eq!(
            read("host.svelte", &page("", &template)),
            want,
            "{template}"
        );
    }
    let small =
        "{#snippet row()}<p>x</p>{/snippet}{#snippet cell(a)}<p>{a}</p>{/snippet}{@render row()}";
    assert_eq!(read("host.svelte", &page("", small)), over(0));
}
