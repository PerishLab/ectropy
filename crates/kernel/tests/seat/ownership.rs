use super::*;

fn notes(path: &str, text: &str) -> Vec<String> {
    let config = config(&[], 4);
    findings(path, text, &config)
        .into_iter()
        .filter(|finding| finding.law == "word")
        .map(|finding| finding.note)
        .collect()
}

fn words(text: &str) -> Vec<String> {
    notes("t.rs", text)
}

#[test]
fn traitimpl() {
    let cases = [
        "impl std::str::FromStr for Unit {\n    type ItemType = u8;\n    const MAX_SIZE: usize = 1;\n    fn from_str(input: &str) -> Result<Self, Self::ItemType> {\n        let _ = input;\n        Ok(Unit)\n    }\n}\n",
        "impl<'a> serde::de::Visitor<'a> for Unique {\n    type Value = String;\n    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {\n        Ok(value.to_string())\n    }\n}\n",
        "impl<F> Trait for Foo<F> where F: for<'a> Fn(&'a str) {\n    fn visit_i64(&self) {}\n}\n",
        "impl<T> Trait for <T as Other>::Assoc {\n    fn visit_i64(&self) {}\n}\n",
        "impl Trait for for<'a> fn(&'a str) {\n    fn visit_i64(&self) {}\n}\n",
        "impl !Send for Unit {}\n",
        "fn host() {\n    impl Trait for Local {\n        fn visit_i64(&self) {}\n    }\n}\n",
    ];
    for source in cases {
        let found = laws(source);
        assert!(
            !found.contains(&"coverage".to_string()),
            "{source}: {found:?}"
        );
        assert!(!found.contains(&"word".to_string()), "{source}: {found:?}");
    }
}

#[test]
fn body() {
    let local = "impl Trait for Unit {\n    fn visit_i64(&self) {\n        let bad_local = 1;\n        let _ = bad_local;\n    }\n}\n";
    assert_eq!(words(local), ["bad_local"]);
    let nested = "impl Trait for Unit {\n    fn visit_i64(&self) {\n        fn bad_nested() {}\n        bad_nested();\n    }\n}\n";
    assert_eq!(words(nested), ["bad_nested"]);
    let inner = "impl Trait for Unit {\n    const MAX_SIZE: usize = {\n        let bad_const = 1;\n        bad_const\n    };\n}\n";
    assert_eq!(words(inner), ["bad_const"]);
}

#[test]
fn inherent() {
    let plain = "impl Foo {\n    fn bad_method(&self) {}\n    type BadAssoc = u8;\n    const BAD_CONST: u8 = 1;\n}\n";
    assert_eq!(words(plain), ["bad_method", "BadAssoc", "BAD_CONST"]);
    let hrtb = "impl<F> Foo<F> where F: for<'a> Fn(&'a str) {\n    fn bad_method(&self) {}\n}\n";
    assert_eq!(words(hrtb), ["bad_method"]);
    let bound = "impl<F: for<'a> Fn(&'a str)> Foo<F> {\n    fn bad_method(&self) {}\n}\n";
    assert_eq!(words(bound), ["bad_method"]);
    let object = "impl dyn for<'a> Fn(&'a str) {\n    fn bad_method(&self) {}\n}\n";
    assert_eq!(words(object), ["bad_method"]);
}

#[test]
fn foreign() {
    let cases = [
        "extern \"C\" {\n    fn plectron_window_create();\n    static FFI_STATE: i32;\n    static mut MUT_STATE: i32;\n    type ForeignHandle;\n}\n",
        "unsafe extern \"C\" {\n    safe fn foo_bar();\n    unsafe fn baz_qux();\n    safe static SAFE_STATE: i32;\n}\n",
    ];
    for source in cases {
        let found = laws(source);
        assert!(
            !found.contains(&"coverage".to_string()),
            "{source}: {found:?}"
        );
        assert!(!found.contains(&"word".to_string()), "{source}: {found:?}");
    }
    let exported = "pub extern \"C\" fn exported_symbol() -> i32 { 1 }\n";
    assert_eq!(words(exported), ["exported_symbol"]);
    let defined = "trait Local {\n    fn bad_method(&self);\n}\n";
    assert_eq!(words(defined), ["bad_method"]);
}

#[test]
fn overrides() {
    let cases = [
        "class Sub(Base):\n    @override\n    def do_something(self, value):\n        return value\n",
        "class Sub(Base):\n    @typing.override\n    async def fetch_item(self, value):\n        return value\n",
        "class Sub(Base):\n    @classmethod\n    @override\n    def from_raw(cls, data):\n        return data\n",
        "class Sub(Base):\n    @typing.override\n    @property\n    def full_name(self):\n        return self.name\n",
    ];
    for source in cases {
        let found = scan("t.py", source);
        assert!(found.is_empty(), "{source}: {found:?}");
    }
    let body = "class Sub(Base):\n    @override\n    def do_something(self, a, b, c, d, e):\n        bad_local = a\n        return bad_local\n";
    let found = scan("t.py", body);
    assert!(found.contains(&"param".to_string()), "{found:?}");
    assert_eq!(notes("t.py", body), ["bad_local"]);
    let plain = "class Sub(Base):\n    def bad_method(self):\n        pass\n";
    assert_eq!(notes("t.py", plain), ["bad_method"]);
    let other = "class Sub(Base):\n    @other\n    def bad_method(self):\n        pass\n";
    assert_eq!(notes("t.py", other), ["bad_method"]);
    let call = "class Sub(Base):\n    @override()\n    def bad_method(self):\n        pass\n";
    assert_eq!(notes("t.py", call), ["bad_method"]);
    let top = "@override\ndef top_bad():\n    pass\n";
    assert_eq!(notes("t.py", top), ["top_bad"]);
    let nested = "def host():\n    @override\n    def inner_bad():\n        pass\n";
    assert_eq!(notes("t.py", nested), ["inner_bad"]);
    let method = "class Sub(Base):\n    @override\n    def do_something(self):\n        @override\n        def inner_bad():\n            pass\n";
    assert_eq!(notes("t.py", method), ["inner_bad"]);
    let class = "@override\nclass BadClass:\n    pass\n";
    assert_eq!(notes("t.py", class), ["BadClass"]);
    let member = "class Outer:\n    @override\n    class BadClass:\n        pass\n";
    assert_eq!(notes("t.py", member), ["BadClass"]);
}
