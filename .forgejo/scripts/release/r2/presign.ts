const EXPIRES = 3600;
const ARCHIVES: Record<string, string> = {
  put_linux: "ectropy-x86_64-unknown-linux-gnu.tar.gz",
  put_darwin: "ectropy-aarch64-apple-darwin.tar.gz",
  put_windows: "ectropy-x86_64-pc-windows-msvc.zip",
};

function fail(message: string): never {
  console.error(`[presign] ${message}`);
  Deno.exit(1);
}

function need(name: string): string {
  const value = (Deno.env.get(name) ?? "").trim();
  if (value === "") {
    fail(`${name} is required`);
  }
  return value;
}

const encoder = new TextEncoder();

function hex(bytes: Uint8Array): string {
  return [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
}

async function digest(value: string): Promise<string> {
  const bytes = await crypto.subtle.digest("SHA-256", encoder.encode(value));
  return hex(new Uint8Array(bytes));
}

async function hmac(key: Uint8Array, value: string): Promise<Uint8Array> {
  const imported = await crypto.subtle.importKey(
    "raw",
    key as BufferSource,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return new Uint8Array(
    await crypto.subtle.sign("HMAC", imported, encoder.encode(value)),
  );
}

function escape(value: string, keepSlash: boolean): string {
  return [...value].map((ch) => {
    if (/[A-Za-z0-9\-._~]/.test(ch) || (ch === "/" && keepSlash)) {
      return ch;
    }
    return [...encoder.encode(ch)]
      .map((b) => `%${b.toString(16).toUpperCase().padStart(2, "0")}`)
      .join("");
  }).join("");
}

async function presign(
  endpoint: URL,
  bucket: string,
  key: string,
  ak: string,
  sk: string,
  when: Date,
): Promise<string> {
  const stamp = when.toISOString().replace(/[-:]|\.\d{3}/g, "");
  const day = stamp.slice(0, 8);
  const scope = `${day}/auto/s3/aws4_request`;
  const path = escape(`/${bucket}/${key}`, true);
  const signed = "cache-control;host";
  const query = [
    ["X-Amz-Algorithm", "AWS4-HMAC-SHA256"],
    ["X-Amz-Credential", `${ak}/${scope}`],
    ["X-Amz-Date", stamp],
    ["X-Amz-Expires", String(EXPIRES)],
    ["X-Amz-SignedHeaders", signed],
  ].map(([name, value]) => `${escape(name, false)}=${escape(value, false)}`)
    .sort().join("&");
  const headers = `cache-control:no-store\nhost:${endpoint.host}\n`;
  const request = ["PUT", path, query, headers, signed, "UNSIGNED-PAYLOAD"]
    .join("\n");
  const material = ["AWS4-HMAC-SHA256", stamp, scope, await digest(request)]
    .join("\n");
  let key4: Uint8Array = encoder.encode(`AWS4${sk}`);
  for (const part of [day, "auto", "s3", "aws4_request"]) {
    key4 = await hmac(key4, part);
  }
  const signature = hex(await hmac(key4, material));
  return `${endpoint.origin}${path}?${query}&X-Amz-Signature=${signature}`;
}

async function output(name: string, value: string): Promise<void> {
  const path = Deno.env.get("GITHUB_OUTPUT");
  if (path) {
    await Deno.writeTextFile(path, `${name}=${value}\n`, { append: true });
  }
}

async function main(): Promise<void> {
  const ak = need("ECTROPY_RELEASES_S3_AK");
  const sk = need("ECTROPY_RELEASES_S3_SK");
  const bucket = need("ECTROPY_RELEASES_S3_BUCKET");
  const endpoint = new URL(need("ECTROPY_RELEASES_S3_URL"));
  const channel = need("RELEASE_CHANNEL");
  const version = need("RELEASE_VERSION");
  const when = new Date();
  for (const [name, archive] of Object.entries(ARCHIVES)) {
    const key = `${channel}/versions/${version}/${archive}`;
    await output(name, await presign(endpoint, bucket, key, ak, sk, when));
    console.log(`[presign] ${key} (${EXPIRES}s)`);
  }
}

await main();
