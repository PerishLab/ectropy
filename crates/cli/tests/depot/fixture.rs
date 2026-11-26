use plumb::depot::{sha, v3};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Output};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) const LIMITS: &str = r#"
rule = "structure.ectropy-policy"

[limit]
block = 4
fanout = 10
file = 300
markup = 8
param = 4
path = 3

[comment]
allow = false

[word]
single = true
"#;

const SCAN: &str = r#"
addressed = true
rule = "structure.ectropy-policy"

[[set]]
name = "crates-rust"
include = ["crates/**/*.rs"]
exclude = ["**/target/**"]
roots = ["crates/*/src", "crates/*/tests"]
tests = ["crates/*/src/**/*.rs", "crates/*/tests/**/*.rs"]
environment = ["crates/*/tests/**/*.rs"]

[[set]]
name = "lib-ts"
include = ["lib/**/*.ts"]
roots = ["lib"]

[[set]]
name = "tests-ts"
include = ["tests/**/*.ts"]
roots = ["tests"]
tests = ["tests/**/*.test.ts"]
"#;

const SHAPE: &str = r#"
[[shape]]
when = ["crates"]
use = ["rule://scan/crates-rust"]

[[shape]]
when = ["lib"]
use = ["rule://scan/lib-ts"]

[[shape]]
when = ["tests"]
use = ["rule://scan/tests-ts"]
"#;

pub(super) struct Mock {
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Mock {
    pub(super) fn open() -> (TcpListener, String) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let source = format!("http://{}", listener.local_addr().expect("addr"));
        (listener, source)
    }

    pub(super) fn start(listener: TcpListener, routes: BTreeMap<String, Vec<u8>>) -> Self {
        let address = listener.local_addr().expect("addr");
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            for incoming in listener.incoming() {
                if flag.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(mut stream) = incoming else {
                    break;
                };
                reply(&mut stream, &routes);
            }
        });
        Self {
            address,
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for Mock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.address);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn reply(stream: &mut TcpStream, routes: &BTreeMap<String, Vec<u8>>) {
    let mut buffer = [0u8; 4096];
    let Ok(size) = stream.read(&mut buffer) else {
        return;
    };
    let path = String::from_utf8_lossy(&buffer[..size])
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .to_string();
    let (status, body) = match routes.get(&path) {
        Some(held) => ("200 OK", held.as_slice()),
        None => ("404 Not Found", &[][..]),
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

pub(super) fn bundle(version: &str, limits: &str) -> v3::Bundle {
    let mut bodies = BTreeMap::new();
    bodies.insert(
        "rules/atoms/limit.toml".to_string(),
        limits.as_bytes().to_vec(),
    );
    bodies.insert(
        "rules/atoms/scan.toml".to_string(),
        SCAN.as_bytes().to_vec(),
    );
    bodies.insert(
        "rules/suites/shape.toml".to_string(),
        SHAPE.as_bytes().to_vec(),
    );
    let objects = bodies
        .iter()
        .map(|(path, body)| v3::Object {
            path: path.clone(),
            sha256: sha(body),
            size: body.len() as u64,
            media: "application/toml".into(),
            executable: false,
        })
        .collect();
    let identity = v3::Identity {
        product: "plumb".into(),
        channel: "stable".into(),
        version: version.into(),
        marker: v3::Marker {
            name: version.into(),
            sha256: "a".repeat(64),
        },
        kind: v3::Kind::Configuration,
    };
    let manifest = v3::Manifest::new(identity, objects).expect("manifest");
    v3::Bundle { manifest, bodies }
}

pub(super) fn publish(
    routes: &mut BTreeMap<String, Vec<u8>>,
    source: &str,
    held: &v3::Bundle,
) -> v3::Pointer {
    let pointer = v3::Pointer::new(
        &held.manifest,
        v3::Publication {
            source,
            prior: None,
            created: "2026-10-28T00:00:00Z".into(),
        },
    )
    .expect("pointer");
    let route = v3::Route::new("stable", v3::Kind::Configuration, &held.manifest.version);
    let prefix = v3::generation(route, &pointer.generation).expect("prefix");
    routes.insert(
        format!("/{}", v3::latest(route).expect("latest")),
        pointer.encode().expect("pointer"),
    );
    routes.insert(
        format!("/{prefix}/{}", v3::LEAF),
        held.manifest.encode().expect("manifest"),
    );
    for (name, body) in &held.bodies {
        routes.insert(format!("/{prefix}/objects/{name}"), body.clone());
    }
    pointer
}

pub(super) fn channel(routes: &mut BTreeMap<String, Vec<u8>>, pointer: &v3::Pointer) {
    let key = format!(
        "/{}",
        v3::channel("stable", v3::Kind::Configuration).expect("channel")
    );
    routes.insert(key, pointer.encode().expect("pointer"));
}

pub(super) fn dead() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("reserve");
    let source = format!("http://{}", listener.local_addr().expect("addr"));
    drop(listener);
    source
}

pub(super) fn put(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}

pub(super) struct Env<'a> {
    pub(super) home: &'a Path,
    pub(super) source: Option<&'a str>,
    pub(super) config: Option<&'a str>,
    pub(super) mark: Option<&'a str>,
}

pub(super) fn invoke(env: &Env<'_>, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ectropy"));
    command
        .args(args)
        .env("PLUMB_HOME", env.home)
        .env("ECTROPY_HOME", env.home.join("ectropy"))
        .env_remove("PLUMB_RULES_SOURCE")
        .env_remove("PLUMB_CONFIGURATION")
        .env_remove("PLUMB_DEPOT_MARK");
    if let Some(source) = env.source {
        command.env("PLUMB_RULES_SOURCE", source);
    }
    if let Some(config) = env.config {
        command.env("PLUMB_CONFIGURATION", config);
    }
    if let Some(mark) = env.mark {
        command.env("PLUMB_DEPOT_MARK", mark);
    }
    command.output().expect("ectropy should run")
}
