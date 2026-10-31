use super::*;

#[test]
fn rust() {
    let bare = config(&[], 4);
    for operation in [
        "lock",
        "lock_shared",
        "try_lock",
        "try_lock_shared",
        "unlock",
    ] {
        let source = format!("fn run() {{ std::fs::File::{operation}(&file); }}");
        assert_eq!(hits("src/lib.rs", &source, &bare), 1, "{operation}");
    }
    for operation in ["lock_exclusive", "try_lock_exclusive", "unlock"] {
        let source = format!("fn run() {{ fs2::FileExt::{operation}(&file); }}");
        assert_eq!(hits("src/lib.rs", &source, &bare), 1, "{operation}");
    }
    assert_eq!(
        hits("src/lib.rs", "fn run() { libc::flock(fd, mode); }", &bare),
        1
    );
}

#[test]
fn python() {
    let bare = config(&[], 4);
    for source in [
        "import fcntl\nfcntl.flock(fd, fcntl.LOCK_EX)\n",
        "import fcntl\nfcntl.lockf(fd, fcntl.LOCK_EX)\n",
        "import msvcrt\nmsvcrt.locking(fd, mode, count)\n",
        "if ready: fcntl.flock(fd, mode)\n",
    ] {
        assert_eq!(hits("src/main.py", source, &bare), 1, "{source}");
    }
}

#[test]
fn ordinary() {
    let bare = config(&[], 4);
    for source in [
        "use std::fs::File; fn run() { File::open(path); }",
        "fn run() { std::fs::File::open(path); }",
        "fn run() { fs2::FileExt::read_at(&file, buffer, offset); }",
        "fn run(data: &std::sync::Mutex<Vec<u8>>) { data.lock(); }",
        "fn run(data: &std::sync::Mutex<()>) { data.lock(); }",
        "fn run() { std::io::stdout().lock(); }",
        "fn run(clock: &Clock) { clock.lock(); clock.unlock(); }",
        "fn run(gate: &Gate) { gate.guard(|| work()); }",
        "fn run() { let text = \"std::fs::File::lock\"; }",
        "fn run() { other::std::fs::File::lock(&file); }",
    ] {
        assert_eq!(hits("src/lib.rs", source, &bare), 0, "{source}");
    }
}

#[test]
fn surface() {
    let bare = config(&[], 4);
    for source in [
        "import fcntl\nfcntl.ioctl(fd, mode)\n",
        "import threading\ngate = threading.Lock()\nwith gate:\n    work()\n",
        "other.fcntl.flock(fd, mode)\n",
        "text = \"fcntl.flock\"\n",
    ] {
        assert_eq!(hits("src/main.py", source, &bare), 0, "{source}");
    }
}

#[test]
fn boundary() {
    let mut policy = config(&[], 4);
    policy.file.grant.push(kernel::config::Grant {
        syntax: "lock".into(),
        paths: vec!["src/carrier.rs".into()],
    });
    let source = "fn run() { std::fs::File::lock(&file); }";
    assert_eq!(hits("src/carrier.rs", source, &policy), 0);
    assert_eq!(hits("src/consumer.rs", source, &policy), 1);
    policy.file.ban.push(kernel::config::Ban {
        syntax: "lock".into(),
        paths: vec!["src/carrier.rs".into()],
    });
    assert_eq!(
        count(
            findings("src/carrier.rs", source, &policy)
                .iter()
                .map(|f| f.law.clone())
                .collect(),
            "ban"
        ),
        1
    );
}

#[test]
fn evidence() {
    let source = "fn run() { let op = std::fs::File::try_lock; }";
    let bare = config(&[], 4);
    let found = findings("src/lib.rs", source, &bare);
    let hit = found.iter().find(|finding| finding.law == "grant").unwrap();
    assert_eq!(hit.line, 1);
    assert_eq!(hit.col, 21);
    assert_eq!(hit.note, "lock syntax outside granted paths");
}

#[test]
fn limits() {
    let bare = config(&[], 4);
    for source in [
        "use std::fs::File as Native; fn run() { Native::lock(&file); }",
        "fn run(file: &std::fs::File) { file.lock(); }",
        "fn run(file: &std::fs::File) { wrap!(file); }",
        "fn run() { ::std::fs::File::lock(&file); }",
    ] {
        assert_eq!(hits("src/lib.rs", source, &bare), 0, "{source}");
    }
    assert_eq!(
        hits(
            "src/main.py",
            "import fcntl as native\nnative.flock(fd, mode)\n",
            &bare
        ),
        0
    );
}
