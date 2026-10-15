use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

const RUST: &str = "[scan]\ninclude = [\"crates/**/*.rs\"]\n";

struct Seat(PathBuf);

impl Seat {
    fn new(config: &str) -> Self {
        let name = format!(
            "ectropy-terrain-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        fs::create_dir(&path).expect("create temp root");
        let seat = Self(path);
        if !config.is_empty() {
            seat.put("ectropy.toml", config);
        }
        seat.put("crates/a/src/lib.rs", "fn main() {}\n");
        seat
    }

    fn put(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, text).expect("write file");
    }

    fn run(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ectropy"))
            .arg(self.path())
            .output()
            .expect("run ectropy")
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn outside() {
    let seat = Seat::new(RUST);
    seat.put("crates/a/x.ts", "let count = 0;\n");
    let output = seat.run();
    assert_eq!(output.status.code(), Some(1));
    let stdout = text(&output.stdout);
    assert!(
        stdout.contains("crates/a/x.ts coverage recognized source outside every scan include"),
        "{stdout}"
    );
    assert!(stdout.contains("1 error"), "{stdout}");
    assert!(stdout.contains("by law: coverage=1"), "{stdout}");
}

#[test]
fn control() {
    let seat = Seat::new(RUST);
    let output = seat.run();
    assert!(output.status.success());
    assert_eq!(text(&output.stdout), "clean\n");
}

#[test]
fn exclude() {
    let seat = Seat::new("[scan]\ninclude = [\"crates/**/*.rs\"]\nexclude = [\"crates/a/x.ts\"]\n");
    seat.put("crates/a/x.ts", "let count = 0;\n");
    assert_eq!(text(&seat.run().stdout), "clean\n");
}

#[test]
fn silent() {
    let seat = Seat::new(RUST);
    seat.put("crates/a/x.js", "let count = 0;\n");
    seat.put("crates/a/x.toml", "[a]\n");
    seat.put("crates/a/README.md", "# a\n");
    seat.put("AGENTS.md", "# agents\n");
    for dir in ["target", "node_modules", ".local", "dist"] {
        seat.put(&format!("crates/a/{dir}/x.ts"), "let count = 0;\n");
    }
    assert_eq!(text(&seat.run().stdout), "clean\n");
}

#[test]
fn whole() {
    let seat = Seat::new("");
    seat.put("tools/x.ts", "let count = 0;\n");
    seat.put("crates/a/src/card.css", ".card {\n  color: red;\n}\n");
    assert_eq!(text(&seat.run().stdout), "clean\n");
}

#[test]
fn boundary() {
    let seat = Seat::new(
        "[scan]\ninclude = [\"crates/**/*.rs\"]\n[[boundary]]\npaths = [\"crates/**\"]\nallow = [\"coverage\"]\nnote = \"probe\"\n",
    );
    seat.put("crates/a/x.ts", "let count = 0;\n");
    let output = seat.run();
    assert_eq!(output.status.code(), Some(2));
    assert!(text(&output.stderr).contains("admits no exemption"));
}
