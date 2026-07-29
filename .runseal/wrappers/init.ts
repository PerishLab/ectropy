import { cli, flags } from "@perish/sealkit/cli";
import { bin, exists } from "@perish/sealkit/cmd";
import { fs } from "@perish/sealkit/fs";
import { io } from "@perish/sealkit/io";
import { path } from "@perish/sealkit/path";

const args = cli.parse(Deno.args, { boolean: ["help", "h"] });
flags(args).positionals("init", { allowHelp: true });
if (flags(args).help()) {
  io.print("Usage: runseal :init");
  io.print("");
  io.print("Validate the repository and install versioned git hooks.");
  Deno.exit(0);
}

const tools = ["git", "tea", "deno", "cargo", "plumb", "runseal", "sh", "bash", "sed", "grep"];
const paths = [
  "Cargo.toml",
  "ectropy.toml",
  "runseal.toml",
  "manage.sh",
  "skills/ectropy/SKILL.md",
  "crates/cli/Cargo.toml",
  "crates/kernel/Cargo.toml",
  "crates/grammar/Cargo.toml",
  ".runseal/deno.json",
  ".runseal/deno.lock",
  ".runseal/hooks/pre-commit",
  ".runseal/hooks/commit-msg",
  ".runseal/wrappers/guard.ts",
  ".runseal/wrappers/init.ts",
  ".runseal/wrappers/land.ts",
  ".forgejo/workflows/guard.yml",
  ".forgejo/workflows/release-beta.yml",
  ".forgejo/workflows/release-stable.yml",
  ".forgejo/scripts/release/assets/skill.sh",
  ".forgejo/release.env.example",
  ".runseal/wrappers/release-env.ts",
];

io.print("==> resolving repository");
const root = await bin("git").text(["rev-parse", "--show-toplevel"]);
io.print(`repository: ${root}`);

io.print("==> checking required tools");
for (const tool of tools) {
  if (!(await exists(tool))) {
    io.fail(`init: missing required tool: ${tool}`);
  }
}
io.print(`ok: ${tools.join(", ")}`);

io.print("==> checking repository entrypoints");
for (const entry of paths) {
  if (!(await fs.file.exists(path.join(root, entry)))) {
    io.fail(`init: missing required path: ${entry}`);
  }
}
io.print("ok: repository entrypoints");

io.print("==> installing git hooks");
await bin("git").run(["config", "core.hooksPath", ".runseal/hooks"], { cwd: root });
const current = await bin("git").text(["config", "--get", "core.hooksPath"], { cwd: root });
io.print(`core.hooksPath = ${current}`);

await bin("deno").run(["--version"], { stdout: "null" });
io.print("development environment ready");
