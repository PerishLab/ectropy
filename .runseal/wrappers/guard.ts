import { cli, flags } from "@perish/sealkit/cli";
import { bin } from "@perish/sealkit/cmd";
import { io } from "@perish/sealkit/io";

function usage(): void {
  io.print("Usage: runseal :guard");
  io.print("");
  io.print("Run the complete repository guard.");
}

const args = cli.parse(Deno.args, { boolean: ["help", "h"] });
flags(args).positionals("guard", { allowHelp: true });
if (flags(args).help()) {
  usage();
  Deno.exit(0);
}

io.print("==> cargo fmt");
await bin("cargo").run(["fmt", "--all", "--check"]);

io.print("==> cargo clippy");
await bin("cargo").run([
  "clippy",
  "--locked",
  "--workspace",
  "--all-targets",
  "--",
  "-D",
  "warnings",
]);

io.print("==> cargo test");
await bin("cargo").run(["test", "--locked", "--workspace"]);

io.print("==> deno fmt");
await bin("deno").run(["fmt", "--check", ".runseal"]);

io.print("==> deno check");
await bin("deno").run([
  "check",
  "--config",
  ".runseal/deno.json",
  "--lock",
  ".runseal/deno.lock",
  "--frozen=true",
  ".runseal/wrappers/guard.ts",
  ".runseal/wrappers/init.ts",
  ".runseal/wrappers/land.ts",
]);

io.print("==> plumb doctor");
await bin("plumb").run(["doctor", "."]);

io.print("==> ectropy self-check");
await bin("cargo").run(["run", "--quiet", "--locked", "-p", "ectropy", "--", "."]);

io.print("==> ectropy skill check");
await bin("cargo").run([
  "run",
  "--quiet",
  "--locked",
  "-p",
  "ectropy",
  "--",
  "skills/ectropy",
]);
