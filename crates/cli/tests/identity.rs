use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Seat(PathBuf);

impl Drop for Seat {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove temp root");
    }
}

#[test]
fn identity() {
    let seat = Seat(std::env::temp_dir().join(format!("ectropy-identity-{}", std::process::id())));
    fs::create_dir(&seat.0).expect("create temp root");
    let source = Path::new(env!("CARGO_BIN_EXE_ectropy"));
    let bytes = fs::read(source).expect("executable");
    let (origin, held) = plumb::identity::inspect(&bytes).expect("reserved identity");
    assert_eq!(origin.prefix, "ECTROPY");
    assert!(held.is_none());
    for marker in ["v0.8.0-rc.1", "v0.8.0"] {
        let binding = plumb::identity::Binding {
            product: "ectropy".into(),
            marker: marker.into(),
            digest: "a".repeat(64),
            commit: "b".repeat(40),
            workload: "c".repeat(64),
        };
        let bound = plumb::identity::bind(&bytes, &binding).expect("bind copy");
        assert_eq!(
            plumb::identity::inspect(&bound).expect("read binding"),
            (origin.clone(), Some(binding.clone()))
        );
        assert_eq!(plumb::identity::bind(&bound, &binding).unwrap(), bound);
        let mut other = binding;
        other.marker = "v0.8.1".into();
        assert!(plumb::identity::bind(&bound, &other).is_err());
        #[cfg(target_os = "linux")]
        probe(&seat.0, source, marker, &bound);
    }
    assert_eq!(fs::read(source).unwrap(), bytes);
}

#[cfg(target_os = "linux")]
fn probe(root: &Path, source: &Path, marker: &str, bytes: &[u8]) {
    let path = root.join(marker);
    fs::write(&path, bytes).expect("bound copy");
    fs::set_permissions(&path, fs::metadata(source).unwrap().permissions())
        .expect("executable permissions");
    let output = Command::new(&path)
        .arg("--version")
        .output()
        .expect("run bound copy");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("ectropy {marker}")
    );
}
