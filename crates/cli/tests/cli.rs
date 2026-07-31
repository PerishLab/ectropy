use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Seat(PathBuf);

impl Seat {
    fn new() -> Self {
        let name = format!(
            "ectropy-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        fs::create_dir(&path).expect("create temp root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove temp root");
    }
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .args(args)
        .output()
        .expect("run ectropy")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf8")
}

#[test]
fn malformed() {
    let seat = Seat::new();
    fs::write(seat.path().join("ectropy.toml"), "[limit\n").expect("write config");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("cannot parse"));
    assert!(!text(&output.stdout).contains("clean"));
}

#[test]
fn unreadable() {
    let seat = Seat::new();
    fs::write(seat.path().join("bad.rs"), [0xff, 0xfe]).expect("write source");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("cannot read"));
    assert!(!text(&output.stdout).contains("clean"));
}

#[test]
fn empty() {
    let seat = Seat::new();
    fs::write(
        seat.path().join("ectropy.toml"),
        "[scan]\ninclude = []\nexclude = []\n",
    )
    .expect("write config");
    fs::write(seat.path().join("bad.rs"), [0xff, 0xfe]).expect("write source");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), "clean\n");
}

#[test]
fn css() {
    let seat = Seat::new();
    fs::write(
        seat.path().join("ectropy.toml"),
        "[scan]\ninclude = [\"**/*.css\"]\n[[ban]]\nsyntax = \"style\"\npaths = [\"**/*.css\"]\n",
    )
    .expect("write config");
    fs::write(seat.path().join("app.css"), ".app { color: red; }\n").expect("write css");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stdout).contains(" ban style syntax banned"));
}

#[test]
fn configured() {
    let seat = Seat::new();
    let source = seat.path().join("crates/lib/src");
    fs::create_dir_all(&source).expect("create source");
    fs::write(
        seat.path().join("ectropy.toml"),
        "[scan]\ninclude = [\"crates/**/*.rs\"]\n",
    )
    .expect("write config");
    fs::write(source.join("bad.rs"), "fn bad_name() {}\n").expect("write source");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stdout).contains("crates/lib/src/bad.rs:1:4 word bad_name"),
        "{}",
        text(&output.stdout)
    );
}

#[test]
fn error() {
    let seat = Seat::new();
    fs::write(seat.path().join("bad.rs"), "fn read_file() {}\n").expect("write source");
    let root = seat.path().to_str().expect("path");
    let output = run(&[root]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = text(&output.stdout);
    assert!(stdout.contains(" word read_file"), "{stdout}");
    assert!(stdout.contains("1 error"), "{stdout}");
    assert!(stdout.contains("by law: word=1"), "{stdout}");
}

#[test]
fn coverage() {
    let seat = Seat::new();
    fs::write(seat.path().join("bad.rs"), "@\n").expect("write source");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert_eq!(output.status.code(), Some(1));
    let stdout = text(&output.stdout);
    assert!(stdout.contains(" coverage unparsed region"), "{stdout}");
    assert!(stdout.contains("1 error"), "{stdout}");
}

#[test]
fn retired() {
    let strict = run(&["--strict", "."]);
    assert_eq!(strict.status.code(), Some(2));
    assert!(text(&strict.stderr).contains("unexpected argument '--strict'"));
    let debt = run(&["--debt", "."]);
    assert_eq!(debt.status.code(), Some(2));
    assert!(text(&debt.stderr).contains("unexpected argument '--debt'"));
    let help = run(&["--help"]);
    let stdout = text(&help.stdout);
    assert!(!stdout.contains("--strict"), "{stdout}");
    assert!(!stdout.contains("--debt"), "{stdout}");
}

#[test]
fn ordered() {
    let seat = Seat::new();
    fs::write(seat.path().join("z.rs"), "fn z() {}\n").expect("write z");
    fs::write(seat.path().join("a.rs"), "fn a() {}\n").expect("write a");
    let root = seat.path().to_str().expect("path");
    let first = run(&["shape", root]);
    let second = run(&["shape", root]);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let rows: Vec<serde_json::Value> = text(&first.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).expect("valid json"))
        .collect();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["path"], "a.rs");
    assert_eq!(rows[1]["path"], "z.rs");
}

#[test]
fn legacy() {
    let output = run(&["--json", "--vocabulary"]);
    assert_eq!(output.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn symlink() {
    use std::os::unix::fs::symlink;

    let seat = Seat::new();
    let outside = Seat::new();
    fs::write(outside.path().join("bad.rs"), "fn bad_name() {}\n").expect("write source");
    symlink(outside.path(), seat.path().join("outside")).expect("link directory");
    let output = run(&[seat.path().to_str().expect("path")]);
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), "clean\n");
}

#[test]
fn pipe() {
    let seat = Seat::new();
    for at in 0..1000 {
        fs::write(
            seat.path().join(format!("file{at}.rs")),
            format!("fn seat{at}() {{}}\n"),
        )
        .expect("write source");
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_ectropy"))
        .args(["shape", seat.path().to_str().expect("path")])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn ectropy");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader.read_line(&mut line).expect("read first line");
    drop(reader);
    let status = child.wait().expect("wait ectropy");
    assert!(status.success());
}
