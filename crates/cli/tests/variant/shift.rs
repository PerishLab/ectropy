use super::{Marks, agree, over, read};

#[test]
fn shifts() {
    for list in [
        "a, b, c = 1 << 2, d = n >> 1, e",
        "a: number, b: number, c: number = 1 << 2, d: number = n >> 1, e: number",
        "a, b, c = 1 << 2, d = n >>> 1, e",
        "a, b, c = (1 << 2), d = n >> 1, e",
        "a, b, c = 1 < 2, d = n >> 1, e",
        "a, b, c = x << 2, d = n > 1, e",
    ] {
        agree(&format!("export function host({list}) {{}}\n"), &over(1));
    }
    for code in [
        "go(1 << 2, (a, b, c, d, e) => a, n >> 1);",
        "export const k = [1 << 2, (a, b, c, d, e) => a, n >> 1];",
        "export function host(): void { x <<= 2, go((a, b, c, d, e) => a), y >>= 1; }",
        "export const s = `${1 << 2}${go((a, b, c, d, e) => a)}${n >> 1}`;",
    ] {
        agree(code, &over(1));
    }
    let markup = "<p>{go(1 << 2, (a, b, c, d, e) => a, n >> 1)}</p>\n";
    assert_eq!(read("host.svelte", markup), over(1), "{markup}");
    let five = "(a: A, b: B, c: C, d: D, e: E) => void";
    for cast in [
        "<<T>(a: T, b: T, c: T, d: T, e: T) => T>y".to_string(),
        "<new <T>(a: T, b: T, c: T, d: T, e: T) => T>y".to_string(),
        format!("<Array<{five}>>y"),
        format!("go(1 << 2, <{five}>y, n >> 1)"),
    ] {
        let host = format!("export function host(y: unknown): unknown {{ return {cast}; }}\n");
        assert_eq!(read("host.ts", &host), Marks::new(), "{host}");
        let script = format!("<script lang=\"ts\">\n{host}</script>\n");
        assert_eq!(read("host.svelte", &script), Marks::new(), "{script}");
    }
}
