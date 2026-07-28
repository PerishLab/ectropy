#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../../../.." && pwd)
VERSION=${1:-}
CHANNEL=${2:-stable}
PUBLIC_URL=${ECTROPY_RELEASES_PUBLIC_URL:-https://releases.ectropy.perish.uk}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

clean="$tmpdir/clean"
fault="$tmpdir/fault"
invalid="$tmpdir/invalid"
mkdir -p "$clean" "$fault" "$invalid"
printf 'fn main() {}\n' >"$clean/sample.rs"
printf 'fn main() {}\n// denied\n' >"$fault/sample.rs"
printf '[limit\n' >"$invalid/ectropy.toml"

assert_binary() {
  bin="$1"
  "$bin" --version | grep -F "${VERSION#v}" >/dev/null
  out=$("$bin" "$clean")
  [ "$out" = clean ] || {
    printf '%s\n' "smoke: expected clean, got: $out" >&2
    exit 1
  }
  if "$bin" "$fault" >/dev/null 2>&1; then
    printf '%s\n' "smoke: comment fault exited zero" >&2
    exit 1
  fi
  set +e
  "$bin" "$invalid" >/dev/null 2>&1
  code=$?
  set -e
  [ "$code" -eq 2 ] || {
    printf '%s\n' "smoke: malformed config exited $code instead of 2" >&2
    exit 1
  }
}

smoke_manager() {
  manager="$1"
  label="$2"
  export ECTROPY_INSTALL_ROOT="$tmpdir/$label/install"
  export ECTROPY_LOCAL_BIN_DIR="$tmpdir/$label/bin"
  mkdir -p "$ECTROPY_INSTALL_ROOT" "$ECTROPY_LOCAL_BIN_DIR"
  sh "$manager" install \
    --public-url "$PUBLIC_URL" \
    --channel "$CHANNEL" \
    --version "$VERSION" \
    --retain=false
  assert_binary "$ECTROPY_LOCAL_BIN_DIR/ectropy"
  sh "$manager" uninstall --version "$VERSION"
  [ ! -e "$ECTROPY_INSTALL_ROOT/$VERSION" ] || {
    printf '%s\n' "smoke: version uninstall left $ECTROPY_INSTALL_ROOT/$VERSION" >&2
    exit 1
  }
  [ ! -e "$ECTROPY_LOCAL_BIN_DIR/ectropy" ] || {
    printf '%s\n' "smoke: uninstall left $ECTROPY_LOCAL_BIN_DIR/ectropy" >&2
    exit 1
  }
}

snapshot="$tmpdir/manage.sh"
curl -fsSL --retry 8 --retry-all-errors --retry-delay 5 \
  "$PUBLIC_URL/$CHANNEL/versions/$VERSION/manage.sh" \
  -o "$snapshot"
chmod +x "$snapshot"
smoke_manager "$snapshot" snapshot
smoke_manager "$ROOT/manage.sh" current
