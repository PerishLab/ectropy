import { cli, flags } from "@perish/sealkit/cli";
import { bin, exists } from "@perish/sealkit/cmd";
import { env } from "@perish/sealkit/env";
import { io } from "@perish/sealkit/io";
import { doc } from "@perish/sealkit/json";
import { hash } from "@perish/sealkit/hash";
import { version } from "@perish/sealkit/version";

function usage(): void {
  io.print("Usage: runseal :guard [version-check|version-hash]");
  io.print("");
  io.print("Run repository guard checks or one explicit version-policy helper.");
}

let mode = "full";
const args = cli.parse(Deno.args, { boolean: ["help", "h"] });
if (flags(args).help()) {
  flags(args).positionals("guard", { allowHelp: true });
  usage();
  Deno.exit(0);
}
if (args._.length > 0) {
  const arg = args._.shift()!;
  switch (arg) {
    case "version-check":
      mode = "version-check";
      break;
    case "version-hash":
      mode = "version-hash";
      break;
    default:
      io.fail(`guard: unknown command: ${arg}`);
  }
}
if (args._.length > 0) {
  io.fail("guard: unexpected arguments");
}

async function currentHash(): Promise<string> {
  return await hash.tree(["crates"]);
}

async function versionPolicy(): Promise<void> {
  const publicUrl = env.get("ECTROPY_RELEASES_PUBLIC_URL", "");
  const metadataUrl = env.get(
    "ECTROPY_STABLE_METADATA_URL",
    publicUrl === "" ? "" : `${publicUrl}/stable/latest/metadata.json`,
  );
  if (metadataUrl === "") {
    io.print("guard version policy: no releases url configured; skipping");
    return;
  }

  const cargoMetadata = await bin("cargo").text(["metadata", "--no-deps", "--format-version", "1"]);
  const currentVersion = doc(cargoMetadata).get(".packages[0].version");
  const hash = await currentHash();
  const response = await fetch(`${metadataUrl}?version=${encodeURIComponent(currentVersion)}`);
  if (response.status === 404) {
    io.print("guard version policy: no stable metadata; skipping");
    return;
  }
  if (response.status !== 200) {
    io.fail(`guard version policy: failed to fetch stable metadata: HTTP ${response.status}`);
  }

  const metadata = await response.text();
  const hasPriorHash = doc(metadata).has(".guard.version.hash");
  const priorHash = hasPriorHash ? doc(metadata).get(".guard.version.hash") : "";
  if (priorHash === "") {
    io.print("guard version policy: stable metadata has no guard.version.hash; skipping");
    return;
  }

  const hasStableVersion = doc(metadata).has(".stableVersion");
  let priorVersion = hasStableVersion ? doc(metadata).get(".stableVersion") : "";
  if (priorVersion === "") {
    const hasReleaseVersion = doc(metadata).has(".releaseVersion");
    if (hasReleaseVersion) {
      priorVersion = doc(metadata).get(".releaseVersion");
    }
  }
  if (priorVersion === "") {
    io.fail("guard version policy: stable metadata is missing stableVersion/releaseVersion");
  }

  const currentOrder = version.compare(currentVersion, priorVersion);
  const priorParsed = version.parse(priorVersion);
  const currentParsed = version.parse(currentVersion);
  const sameMinorLineage = currentParsed.major === priorParsed.major &&
    currentParsed.minor === priorParsed.minor;

  if (currentOrder === "lt") {
    io.fail(`guard version policy: version regressed below prior stable ${priorVersion}`);
  }
  if (currentOrder === "eq") {
    io.fail(`guard version policy: version matches prior stable ${priorVersion}`);
  }

  if (hash === priorHash) {
    if (!sameMinorLineage) {
      io.fail(
        `guard version policy: unchanged guard.version.hash requires a patch-only bump above ${priorVersion}`,
      );
    }
    io.print(
      `guard version policy: hash unchanged -> patch bump ok (${priorVersion} -> ${currentVersion})`,
    );
  } else {
    if (sameMinorLineage) {
      io.fail(
        `guard version policy: changed guard.version.hash requires a minor-or-higher bump above ${priorVersion}`,
      );
    }
    io.print(
      `guard version policy: hash changed -> minor-or-higher bump ok (${priorVersion} -> ${currentVersion})`,
    );
  }
}

if (mode === "version-hash") {
  io.print(await currentHash());
  Deno.exit(0);
}

await versionPolicy();
if (mode === "version-check") {
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
  ".runseal/wrappers/release.ts",
  ".runseal/wrappers/release-env.ts",
]);

io.print("==> deno check release metadata");
await bin("deno").run([
  "check",
  ".forgejo/scripts/release/metadata/beta.ts",
  ".forgejo/scripts/release/metadata/stable.ts",
  ".forgejo/scripts/release/tests/metadata.ts",
]);

io.print("==> deno test release metadata");
await bin("deno").run(["test", ".forgejo/scripts/release/tests/metadata.ts"]);

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

io.print("==> shell syntax");
for (
  const [command, script] of [
    ["sh", "manage.sh"],
    ["sh", ".forgejo/scripts/release/assets/checksums.sh"],
    ["sh", ".forgejo/scripts/release/assets/package.sh"],
    ["sh", ".forgejo/scripts/release/assets/skill.sh"],
    ["sh", ".forgejo/scripts/release/assets/verify.sh"],
    ["bash", ".forgejo/scripts/release/r2/check.sh"],
    ["bash", ".forgejo/scripts/release/r2/absent.sh"],
    ["bash", ".forgejo/scripts/release/r2/publish.sh"],
    ["bash", ".forgejo/scripts/release/r2/summary.sh"],
    ["bash", ".forgejo/scripts/release/r2/verify.sh"],
    ["sh", ".forgejo/scripts/release/smoke/smoke.sh"],
    ["sh", ".forgejo/scripts/release/tests/guard.sh"],
  ]
) {
  await bin(command).run(["-n", script]);
}

io.print("==> release invariants");
await bin("sh").run([".forgejo/scripts/release/tests/guard.sh"]);
