use kernel::law::LAWS;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn boundary(allow: &str) -> Result<(), String> {
    let text =
        format!("[[boundary]]\npaths = [\"src/**\"]\nallow = [\"{allow}\"]\nnote = \"probe\"\n");
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("ectropy-law-{id}"));
    fs::create_dir(&root).expect("create law root");
    fs::write(root.join("ectropy.toml"), text).expect("write config");
    let loaded = kernel::config::load(&root);
    fs::remove_dir_all(&root).expect("remove law root");
    loaded.map(|_| ()).map_err(|error| error.to_string())
}

#[test]
fn catalog() {
    let names: Vec<&str> = LAWS.iter().map(|law| law.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted, names, "catalog must be sorted and unique");
    for law in LAWS {
        assert!(!law.note.trim().is_empty(), "{}", law.name);
    }
}

#[test]
fn sealed() {
    for name in ["ban", "coverage", "grant"] {
        assert!(!kernel::law::find(name).expect(name).exempt);
        let error = boundary(name).expect_err(name);
        assert!(error.contains("admits no exemption"), "{error}");
    }
}

#[test]
fn exemptable() {
    for law in LAWS.iter().filter(|law| law.exempt) {
        boundary(law.name).unwrap_or_else(|error| panic!("{}: {error}", law.name));
    }
}

#[test]
fn unknown() {
    let error = boundary("missing").expect_err("unknown law");
    assert!(error.contains("unknown boundary law `missing`"), "{error}");
}
