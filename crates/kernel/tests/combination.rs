mod seat;
use grammar::Kind;
use seat::*;

#[test]
fn threshold() {
    let four = "fn read() { if ready && open && known && local { send(); } }";
    let three = "fn read() { if ready && open && known { send(); } }";
    assert_eq!(count(laws(four), "combination"), 1);
    assert_eq!(count(laws(three), "combination"), 0);
}

#[test]
fn mixed() {
    let source = "fn read() { if ready && (open || known) && local { send(); } }";
    let found = findings("t.rs", source, &config(&[], 4));
    let hit = found
        .iter()
        .find(|finding| finding.law == "combination")
        .expect("combination");
    assert_eq!(
        hit.note,
        "4 decision atoms over limit 3, the boolean expression carries an anonymous combination model; see: ectropy cookbook combination"
    );
}

#[test]
fn negated() {
    let source = "fn read() { if ready && !(open || known) && local { send(); } }";
    assert_eq!(count(laws(source), "combination"), 1);
}

#[test]
fn nested() {
    let calm = "fn read() { if ready && check(open || known) && local { send(); } }";
    let crowded = "fn read() { if ready && check(open || known || safe || local) { send(); } }";
    assert_eq!(count(laws(calm), "combination"), 0);
    assert_eq!(count(laws(crowded), "combination"), 1);
}

#[test]
fn separate() {
    let source = "fn read() { if ready && open { one(); } if known && local { two(); } }";
    assert_eq!(count(laws(source), "combination"), 0);
}

#[test]
fn assigned() {
    let source = "fn read() { let ready = open && known && safe && local; send(ready); }";
    assert_eq!(count(laws(source), "combination"), 1);
}

#[test]
fn closure() {
    let source = "fn read() { let check = move || open && known && safe && local; check(); }";
    assert_eq!(count(laws(source), "combination"), 1);
    assert_eq!(
        count(
            laws("fn read() { let check = move || open && known; check(); }"),
            "combination"
        ),
        0
    );
}

#[test]
fn typescript() {
    let source =
        "export function read(): void { if (ready && open && known && local) { send(); } }";
    assert_eq!(count(scan("t.ts", source), "combination"), 1);
    assert_eq!(count(scan("t.tsx", source), "combination"), 1);
}

#[test]
fn branch() {
    let source = "const ready = open && known ? safe && local : remote && public;";
    assert_eq!(count(scan("t.ts", source), "combination"), 0);
}

#[test]
fn literal() {
    let rust = "fn read() { let ready = open && Some { held: true }.held && known && local; }";
    let typescript = "const ready = open && { held: true }.held && known && local;";
    assert_eq!(count(laws(rust), "combination"), 1);
    assert_eq!(count(scan("t.ts", typescript), "combination"), 1);
}

#[test]
fn block() {
    let source = "fn read() { let ready = open && { known } && safe && local; }";
    assert_eq!(count(laws(source), "combination"), 1);
}

#[test]
fn regex() {
    let source = "const pattern = /open&&known&&safe&&local/;";
    assert_eq!(count(scan("t.ts", source), "combination"), 0);
    let division = "const ready = open / total && known && safe && local;";
    assert_eq!(count(scan("t.ts", division), "combination"), 1);
    let mixed = "const ratio = total / /open&&known&&safe&&local/.source.length;";
    assert_eq!(count(scan("t.ts", mixed), "combination"), 0);
    let postfix = "const ratio = total++ / open && known && safe && local;";
    assert_eq!(count(scan("t.ts", postfix), "combination"), 1);
}

#[test]
fn prose() {
    let text = "export const View = () => <p>open && known && safe && local</p>;";
    let code = "export const View = () => <p>{open && known && safe && local}</p>;";
    assert_eq!(count(scan("t.tsx", text), "combination"), 0);
    assert_eq!(count(scan("t.tsx", code), "combination"), 1);
}

#[test]
fn shape() {
    let text = "fn read() { if ready && open && known && local { send(); } }";
    let source = grammar::Source {
        path: "t.rs".to_string(),
        text: text.to_string(),
    };
    let root = kernel::structure(&source);
    let node = root
        .kids
        .iter()
        .find(|node| node.kind == Kind::Decision)
        .expect("decision");
    assert_eq!(node.kids.len(), 4);
    assert!(node.kids.iter().all(|node| node.kind == Kind::Atom));
}

#[test]
fn boundary() {
    let mut config = config(&[], 4);
    config.file.boundary.push(kernel::config::Boundary {
        paths: vec!["vendor/**".to_string()],
        allow: vec!["combination".to_string()],
        note: "vendored query language owns its predicate shape".to_string(),
    });
    let source = "fn read() { if ready && open && known && local { send(); } }";
    assert_eq!(
        count(
            findings("vendor/read.rs", source, &config)
                .iter()
                .map(|finding| finding.law.clone())
                .collect(),
            "combination"
        ),
        0
    );
}
