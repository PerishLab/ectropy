#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
temp=$(mktemp -d)
trap 'rm -rf "$temp"' EXIT INT TERM

fake="$temp/bin"
mkdir -p "$fake"

printf 'tampered\n' >"$temp/archive"
cat >"$temp/metadata.json" <<'EOF'
{
  "releaseVersion": "v0.1.0",
  "artifacts": {
    "linuxX64": {
      "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
    }
  }
}
EOF

cat >"$fake/curl" <<EOF
#!/usr/bin/env sh
set -eu
url=
out=
while [ \$# -gt 0 ]; do
  case "\$1" in
    -o) out=\$2; shift 2 ;;
    http*) url=\$1; shift ;;
    *) shift ;;
  esac
done
case "\$url" in
  */metadata.json) cp "$temp/metadata.json" "\$out" ;;
  *.tar.gz) cp "$temp/archive" "\$out" ;;
  *) exit 1 ;;
esac
EOF
chmod +x "$fake/curl"

install="$temp/install"
mkdir -p "$install/v0.1.0"
printf 'keep\n' >"$install/v0.1.0/sentinel"
if PATH="$fake:$PATH" \
  ECTROPY_INSTALL_ROOT="$install" \
  ECTROPY_LOCAL_BIN_DIR="$temp/local-bin" \
  sh "$ROOT/manage.sh" install \
    --public-url https://example.invalid \
    --channel stable \
    --version v0.1.0 \
    --retain=true >/dev/null 2>&1; then
  printf '%s\n' 'tampered archive was accepted' >&2
  exit 1
fi
[ -f "$install/v0.1.0/sentinel" ] || {
  printf '%s\n' 'failed checksum replaced the installed version' >&2
  exit 1
}

cat >"$fake/aws" <<'EOF'
#!/usr/bin/env sh
if [ "${AWS_STUB_STATUS:-1}" -eq 0 ]; then
  exit 0
fi
printf '%s\n' 'An error occurred (404) when calling HeadObject: Not Found' >&2
exit 1
EOF
chmod +x "$fake/aws"

if PATH="$fake:$PATH" \
  AWS_STUB_STATUS=0 \
  ECTROPY_RELEASES_S3_AK=ak \
  ECTROPY_RELEASES_S3_SK=sk \
  ECTROPY_RELEASES_S3_BUCKET=bucket \
  ECTROPY_RELEASES_S3_URL=https://example.invalid \
  RELEASE_CHANNEL=stable \
  RELEASE_VERSION=v0.1.0 \
  bash "$ROOT/.forgejo/scripts/release/r2/absent.sh" >/dev/null 2>&1; then
  printf '%s\n' 'immutable overwrite was accepted' >&2
  exit 1
fi

PATH="$fake:$PATH" \
  AWS_STUB_STATUS=1 \
  ECTROPY_RELEASES_S3_AK=ak \
  ECTROPY_RELEASES_S3_SK=sk \
  ECTROPY_RELEASES_S3_BUCKET=bucket \
  ECTROPY_RELEASES_S3_URL=https://example.invalid \
  RELEASE_CHANNEL=stable \
  RELEASE_VERSION=v0.1.0 \
  bash "$ROOT/.forgejo/scripts/release/r2/absent.sh"
