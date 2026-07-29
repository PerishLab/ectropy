#!/usr/bin/env sh
set -eu

MODE=${1:-}
RELEASE_VERSION=${2:-}
ARTIFACT_DIR=${3:-}

[ -n "$MODE" ] || { printf '%s\n' 'missing mode' >&2; exit 1; }
[ -n "$RELEASE_VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }
[ -n "$ARTIFACT_DIR" ] || { printf '%s\n' 'missing artifact dir' >&2; exit 1; }
[ -d "$ARTIFACT_DIR" ] || { printf 'artifact dir missing: %s\n' "$ARTIFACT_DIR" >&2; exit 1; }

ASSETS="ectropy-x86_64-unknown-linux-gnu.tar.gz ectropy-aarch64-apple-darwin.tar.gz ectropy-skill.tar.gz ectropy-x86_64-pc-windows-msvc.zip"

require_file() {
  [ -f "$ARTIFACT_DIR/$1" ] || { printf 'missing artifact: %s\n' "$1" >&2; exit 1; }
}

require_checksum_entry() {
  awk -v name="$1" 'NR > 1 && $NF == name { found = 1 } END { exit(found ? 0 : 1) }' \
    "$ARTIFACT_DIR/checksums.txt"
}

ensure_tar_contains() {
  tar tzf "$ARTIFACT_DIR/$1" | grep -Fxq "$2" || {
    printf 'missing %s in %s\n' "$2" "$1" >&2
    exit 1
  }
}

ensure_zip_contains() {
  unzip -Z1 "$ARTIFACT_DIR/$1" | grep -Fxq "$2" || {
    printf 'missing %s in %s\n' "$2" "$1" >&2
    exit 1
  }
}

check_archive_members() {
  ensure_tar_contains "ectropy-x86_64-unknown-linux-gnu.tar.gz" "ectropy"
  ensure_tar_contains "ectropy-aarch64-apple-darwin.tar.gz" "ectropy"
  ensure_tar_contains "ectropy-skill.tar.gz" "ectropy/SKILL.md"
  ensure_tar_contains "ectropy-skill.tar.gz" "ectropy/metadata.json"
  ensure_zip_contains "ectropy-x86_64-pc-windows-msvc.zip" "ectropy.exe"
}

verify_checksums() {
  tail -n +2 "$ARTIFACT_DIR/checksums.txt" |
    while read -r digest name; do
      [ -n "$digest" ] || continue
      [ -n "$name" ] || { printf '%s\n' 'malformed checksum entry' >&2; exit 1; }
      actual=$(
        if command -v sha256sum >/dev/null 2>&1; then
          sha256sum "$ARTIFACT_DIR/$name" | awk '{print $1}'
        else
          shasum -a 256 "$ARTIFACT_DIR/$name" | awk '{print $1}'
        fi
      )
      [ "$actual" = "$digest" ] || {
        printf 'checksum mismatch for %s: %s != %s\n' "$name" "$actual" "$digest" >&2
        exit 1
      }
    done
}

case "$MODE" in
  accept)
    require_file checksums.txt
    version_line=$(sed -n 's/^VERSION: *//p' "$ARTIFACT_DIR/checksums.txt" | head -n 1)
    [ "$version_line" = "$RELEASE_VERSION" ] || {
      printf 'version mismatch: expected %s got %s\n' "$RELEASE_VERSION" "$version_line" >&2
      exit 1
    }
    for asset in $ASSETS; do
      require_file "$asset"
      require_checksum_entry "$asset" || {
        printf 'missing checksum entry: %s\n' "$asset" >&2
        exit 1
      }
    done
    ;;
  verify)
    verify_checksums
    check_archive_members
    ;;
  *)
    printf 'unknown mode: %s\n' "$MODE" >&2
    exit 1
    ;;
esac
