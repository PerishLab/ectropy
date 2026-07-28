import { bin, exists } from "@perish/sealkit/cmd";
import { fs } from "@perish/sealkit/fs";
import { io } from "@perish/sealkit/io";

const FILE = ".forgejo/release.env";
const EXAMPLE = ".forgejo/release.env.example";
const REQUIRED = [
  "ECTROPY_RELEASES_PUBLIC_URL",
  "ECTROPY_RELEASES_S3_AK",
  "ECTROPY_RELEASES_S3_SK",
  "ECTROPY_RELEASES_S3_BUCKET",
  "ECTROPY_RELEASES_S3_URL",
];

function usage(): void {
  io.print("Usage: runseal :release-env <command>");
  io.print("");
  io.print("Commands:");
  io.print("  init                      create repo-local .forgejo/release.env");
  io.print("  check                     validate repo-local S3/R2 release credentials");
  io.print("");
  io.print("Credentials:");
  io.print(`  ${FILE}`);
}

function rejectExtraArg(value: string | undefined, message: string): void {
  if (value !== undefined && value !== "") {
    io.fail(message);
  }
}

async function initCommand(rest: string[]): Promise<void> {
  rejectExtraArg(rest[0], "release-env: init does not accept arguments");
  await fs.dir.ensure(".local/tmp", "700");
  if (await fs.file.exists(FILE)) {
    await fs.file.chmodIfUnix(FILE, "600");
    io.print(`exists ${FILE}`);
    return;
  }
  const template = await Deno.readTextFile(EXAMPLE);
  await fs.file.writeText(FILE, template, "600");
  io.print(`created ${FILE}`);
}

function parse(text: string): Record<string, string> {
  const values: Record<string, string> = {};
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (line === "" || line.startsWith("#")) {
      continue;
    }
    const split = line.indexOf("=");
    if (split < 0) {
      io.fail(`release-env: invalid line: ${line}`);
    }
    const key = line.slice(0, split).trim();
    const value = line.slice(split + 1).trim().replace(/^["']|["']$/g, "");
    values[key] = value;
  }
  return values;
}

async function values(): Promise<Record<string, string>> {
  const parsed = parse(await Deno.readTextFile(FILE));
  for (const name of REQUIRED) {
    if (parsed[name] === undefined || parsed[name] === "") {
      io.fail(`release-env: ${name} is required in ${FILE}`);
    }
  }
  return parsed;
}

async function checkCommand(rest: string[]): Promise<void> {
  rejectExtraArg(rest[0], "release-env: check does not accept arguments");
  const aws = await bin("aws").status(["--version"], {
    stdout: "null",
    stderr: "null",
  });
  if (aws !== 0) {
    io.fail("release-env: missing required tool: aws");
  }
  await fs.dir.ensure(".local/tmp", "700");
  const sha = await bin("git").text(["rev-parse", "HEAD"]);
  await bin("bash").run([".forgejo/scripts/release/r2/check.sh"], {
    env: {
      ...(await values()),
      RELEASE_CHANNEL: "beta",
      R2_ACCESS_PROBE_NAME: "local",
      ECTROPY_RELEASE_TEMP: ".local/tmp",
      CI_RUN_ID: "local",
      CI_COMMIT: sha,
    },
  });
  io.print("release-env check: ok");
}

const [command, ...rest] = Deno.args;
if (command === undefined || command === "help" || command === "--help") {
  usage();
  Deno.exit(0);
}

switch (command) {
  case "init":
    await initCommand(rest);
    break;
  case "check":
    await checkCommand(rest);
    break;
  default:
    io.fail(`release-env: unknown command: ${command}`);
}
