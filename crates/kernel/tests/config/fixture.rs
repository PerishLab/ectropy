use plumb::depot::{Rules, sha, v3};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) const LIMITS: &str = r#"
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
name = "crates-ts"
include = ["crates/**/*.ts", "crates/**/*.tsx"]
exclude = ["**/target/**", "**/node_modules/**"]
roots = ["crates/*/src", "crates/*/tests"]
derive = ["crates/*"]

[[set]]
name = "app-rust"
include = ["app/src/**/*.rs", "app/tests/**/*.rs"]
exclude = ["**/target/**"]
roots = ["app/src", "app/tests"]
tests = ["app/tests/**/*.rs"]

[[set]]
name = "lib-ts"
include = ["lib/**/*.ts"]
roots = ["lib"]

[[set]]
name = "web-components-banned"
bans = ["apps/web/src/lib/components/**"]

[[set]]
name = "apps-web"
include = ["apps/**/*.ts", "apps/**/*.css"]
exclude = ["**/node_modules/**", "**/dist/**"]
roots = ["apps/*", "apps/*/src"]
tests = ["apps/*/tests/**/*.test.ts"]

[[set]]
name = "apps-web-svelte"
include = ["apps/**/*.svelte"]
exclude = ["**/.svelte-kit/**"]

[[set]]
name = "apps-web-tsx"
include = ["apps/**/*.tsx"]
tests = ["apps/*/tests/**/*.test.tsx"]

[[set]]
name = "tools-rust"
include = ["tools/**/*.rs"]
roots = ["tools/*/src"]

[[set]]
name = "tools-js"
include = ["tools/**/*.js"]
roots = ["tools/*"]
"#;

const SHAPE: &str = r#"
[[shape]]
when = ["crates"]
use = ["rule://scan/crates-rust"]
tsx = ["rule://scan/crates-ts"]

[[shape]]
when = ["tools"]
use = ["rule://scan/tools-rust"]
js = ["rule://scan/tools-js"]

[[shape]]
when = ["app"]
use = ["rule://scan/app-rust"]

[[shape]]
when = ["lib"]
use = ["rule://scan/lib-ts"]

[[shape]]
when = ["apps/web/src/lib/components"]
use = ["rule://scan/web-components-banned"]

[[web]]
seat = "apps"
use = ["rule://scan/apps-web"]
svelte = ["rule://scan/apps-web-svelte"]
tsx = ["rule://scan/apps-web-tsx"]
"#;

pub(crate) fn rules(limits: &str) -> Rules {
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
        version: "v0.164.0".into(),
        marker: v3::Marker {
            name: "v0.164.0".into(),
            sha256: "a".repeat(64),
        },
        kind: v3::Kind::Configuration,
    };
    let manifest = v3::Manifest::new(identity, objects).expect("manifest");
    Rules::bundle(&v3::Bundle { manifest, bodies }).expect("rules")
}

pub(crate) fn put(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    fs::write(path, body).expect("write");
}
