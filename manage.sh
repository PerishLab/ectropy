#!/usr/bin/env sh
set -eu

COMMAND=${1:-install}
[ $# -gt 0 ] && shift || true

CHANNEL=${ECTROPY_CHANNEL:-stable}
VERSION=${ECTROPY_VERSION:-}
PUBLIC_URL=${ECTROPY_RELEASES_PUBLIC_URL:-https://releases.ectropy.perish.uk}
INSTALL_ROOT=${ECTROPY_INSTALL_ROOT:-"$HOME/.local/share/ectropy"}
LOCAL_BIN_DIR=${ECTROPY_LOCAL_BIN_DIR:-"$HOME/.local/bin"}
RETAIN=${ECTROPY_RETAIN:-}

while [ $# -gt 0 ]; do
  case "$1" in
    --channel)
      CHANNEL=${2:-}
      [ -n "$CHANNEL" ] || { echo "--channel requires a value" >&2; exit 1; }
      shift 2
      ;;
    --channel=*)
      CHANNEL=${1#--channel=}
      shift
      ;;
    --version)
      VERSION=${2:-}
      [ -n "$VERSION" ] || { echo "--version requires a value" >&2; exit 1; }
      shift 2
      ;;
    --version=*)
      VERSION=${1#--version=}
      shift
      ;;
    --public-url)
      PUBLIC_URL=${2:-}
      [ -n "$PUBLIC_URL" ] || { echo "--public-url requires a value" >&2; exit 1; }
      shift 2
      ;;
    --public-url=*)
      PUBLIC_URL=${1#--public-url=}
      shift
      ;;
    --install-root)
      INSTALL_ROOT=${2:-}
      [ -n "$INSTALL_ROOT" ] || { echo "--install-root requires a value" >&2; exit 1; }
      shift 2
      ;;
    --install-root=*)
      INSTALL_ROOT=${1#--install-root=}
      shift
      ;;
    --bin-dir)
      LOCAL_BIN_DIR=${2:-}
      [ -n "$LOCAL_BIN_DIR" ] || { echo "--bin-dir requires a value" >&2; exit 1; }
      shift 2
      ;;
    --bin-dir=*)
      LOCAL_BIN_DIR=${1#--bin-dir=}
      shift
      ;;
    --retain)
      RETAIN=true
      shift
      ;;
    --retain=*)
      RETAIN=${1#--retain=}
      shift
      ;;
    -h|--help|help)
      cat <<'EOF'
ectropy manager

Usage:
  manage.sh install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.sh uninstall [--version vX.Y.Z]

install leaves exactly one version on disk. Earlier versions are removed once
the new binary is linked and answers --version, and each removal is named.
Rolling back is install --version <older>, which fetches that version again;
released artifacts are immutable and always retrievable. Pass --retain to keep
what is already there.

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the ectropy shim/link

Environment:
  ECTROPY_RELEASES_PUBLIC_URL
  ECTROPY_CHANNEL
  ECTROPY_VERSION
  ECTROPY_INSTALL_ROOT
  ECTROPY_LOCAL_BIN_DIR
  ECTROPY_RETAIN
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

need_public_url() {
  PUBLIC_URL=${PUBLIC_URL%/}
}

normalize_bool() {
  case "$1" in
    true|1|yes|y|on) printf '%s' true ;;
    false|0|no|n|off) printf '%s' false ;;
    *) echo "invalid --retain value: $1" >&2; exit 1 ;;
  esac
}

normalize_version() {
  printf 'v%s' "$(printf '%s' "$1" | sed 's/^v//')"
}

platform_archive() {
  os=$(uname -s)
  arch=$(uname -m)
  case "$os:$arch" in
    Linux:x86_64|Linux:amd64) echo "ectropy-x86_64-unknown-linux-gnu.tar.gz" ;;
    Darwin:arm64|Darwin:aarch64) echo "ectropy-aarch64-apple-darwin.tar.gz" ;;
    *) echo "unsupported platform: $os $arch" >&2; exit 1 ;;
  esac
}

latest_version() {
  metadata="$1"
  sed -n 's/.*"releaseVersion"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$metadata" | head -n 1
}

object_string() {
  object="$1"
  field="$2"
  metadata="$3"
  awk -v object="\"$object\"" -v field="\"$field\"" '
    $0 ~ object "[[:space:]]*:" { inside = 1 }
    inside && $0 ~ field "[[:space:]]*:" {
      line = $0
      sub(".*" field "[[:space:]]*:[[:space:]]*\"", "", line)
      sub("\".*", "", line)
      print line
      exit
    }
    inside && /}/ { exit }
  ' "$metadata"
}

base_version() {
  printf '%s' "$1" | sed 's/^v//; s/-.*//'
}

version_at_least() {
  left=$(base_version "$1")
  right=$(base_version "$2")
  old_ifs=$IFS
  IFS=.
  set -- $left
  left_major=${1:-0}
  left_minor=${2:-0}
  left_patch=${3:-0}
  set -- $right
  right_major=${1:-0}
  right_minor=${2:-0}
  right_patch=${3:-0}
  IFS=$old_ifs
  [ "$left_major" -gt "$right_major" ] ||
    { [ "$left_major" -eq "$right_major" ] &&
      { [ "$left_minor" -gt "$right_minor" ] ||
        { [ "$left_minor" -eq "$right_minor" ] && [ "$left_patch" -ge "$right_patch" ]; }; }; }
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

old_versions() {
  current="$1"
  [ -d "$INSTALL_ROOT" ] || return 0
  for found in "$INSTALL_ROOT"/*; do
    [ -d "$found" ] || continue
    name=$(basename "$found")
    [ "$name" != "$current" ] || continue
    printf '%s\n' "$name"
  done
}

retain_old_versions() {
  old="$1"
  if [ -z "$old" ]; then
    printf '%s' true
    return
  fi
  if [ -n "$RETAIN" ]; then
    normalize_bool "$RETAIN"
    return
  fi
  printf '%s' false
}

install_ectropy() {
  need_public_url
  tmpdir=$(mktemp -d)
  stage=
  cleanup() {
    rm -rf "$tmpdir"
    [ -z "$stage" ] || rm -rf "$stage"
  }
  trap cleanup EXIT INT TERM

  resolved_latest=false
  if [ -z "$VERSION" ]; then
    curl -fsSL --retry 8 --retry-all-errors --retry-delay 5 \
      "$PUBLIC_URL/$CHANNEL/latest/metadata.json" -o "$tmpdir/metadata.json"
    VERSION=$(latest_version "$tmpdir/metadata.json")
    [ -n "$VERSION" ] || { echo "failed to resolve latest ectropy version" >&2; exit 1; }
    resolved_latest=true
  fi
  VERSION=$(normalize_version "$VERSION")

  if [ "$CHANNEL" = beta ] && [ "$resolved_latest" = true ]; then
    if curl -fsSL --retry 3 --retry-all-errors --retry-delay 2 \
      "$PUBLIC_URL/stable/latest/metadata.json" -o "$tmpdir/stable.json"; then
      stable=$(latest_version "$tmpdir/stable.json")
      [ -z "$stable" ] || version_at_least "$VERSION" "$stable" || {
        echo "refusing beta $VERSION older than stable $stable" >&2
        exit 1
      }
    fi
  fi

  old=$(old_versions "$VERSION")
  retain=$(retain_old_versions "$old")

  archive=$(platform_archive)
  case "$archive" in
    *linux*) platform=linuxX64 ;;
    *darwin*) platform=darwinArm64 ;;
    *) echo "unsupported archive: $archive" >&2; exit 1 ;;
  esac
  metadata_url="$PUBLIC_URL/$CHANNEL/versions/$VERSION/metadata.json"
  curl -fsSL --retry 8 --retry-all-errors --retry-delay 5 \
    "$metadata_url" -o "$tmpdir/version.json"
  metadata_version=$(latest_version "$tmpdir/version.json")
  [ "$metadata_version" = "$VERSION" ] || {
    echo "metadata version mismatch: expected $VERSION got $metadata_version" >&2
    exit 1
  }
  expected=$(object_string "$platform" sha256 "$tmpdir/version.json")
  [ -n "$expected" ] || { echo "metadata missing $platform sha256" >&2; exit 1; }
  archive_url="$PUBLIC_URL/$CHANNEL/versions/$VERSION/$archive"
  curl -fsSL --retry 8 --retry-all-errors --retry-delay 5 \
    "$archive_url" -o "$tmpdir/$archive"
  actual=$(sha256 "$tmpdir/$archive")
  [ "$actual" = "$expected" ] || {
    echo "checksum mismatch for $archive: expected $expected got $actual" >&2
    exit 1
  }

  mkdir -p "$INSTALL_ROOT" "$LOCAL_BIN_DIR"
  stage="$INSTALL_ROOT/.ectropy-stage-$$"
  rm -rf "$stage"
  mkdir -p "$stage"
  tar -xzf "$tmpdir/$archive" -C "$stage"
  [ -f "$stage/ectropy" ] || { echo "archive missing ectropy" >&2; exit 1; }
  chmod +x "$stage/ectropy"
  staged_version=$("$stage/ectropy" --version)
  case "$staged_version" in
    *"${VERSION#v}"*) ;;
    *) echo "binary version mismatch: $staged_version" >&2; exit 1 ;;
  esac

  rm -rf "$INSTALL_ROOT/$VERSION"
  mv "$stage" "$INSTALL_ROOT/$VERSION"
  stage=

  link="$LOCAL_BIN_DIR/ectropy"
  next_link="$LOCAL_BIN_DIR/.ectropy-link-$$"
  rm -f "$next_link"
  ln -s "$INSTALL_ROOT/$VERSION/ectropy" "$next_link"
  mv -f "$next_link" "$link"
  "$link" --version

  if [ "$retain" = false ]; then
    printf '%s\n' "$old" | while IFS= read -r old_version; do
      [ -n "$old_version" ] || continue
      rm -rf "$INSTALL_ROOT/$old_version"
      printf 'removed old ectropy %s from %s\n' "$old_version" "$INSTALL_ROOT"
    done
  fi

  printf 'installed ectropy to %s\n' "$link"
}

remove_empty_dir() {
  dir="$1"
  if [ -d "$dir" ]; then
    rmdir "$dir" 2>/dev/null || true
  fi
}

uninstall_ectropy() {
  bin_path="$LOCAL_BIN_DIR/ectropy"
  if [ -n "$VERSION" ]; then
    VERSION=$(normalize_version "$VERSION")
    target="$INSTALL_ROOT/$VERSION/ectropy"
    if [ -L "$bin_path" ]; then
      link_target=$(readlink "$bin_path" || true)
      if [ "$link_target" = "$target" ]; then
        rm -f "$bin_path"
        printf 'removed %s\n' "$bin_path"
      fi
    fi
    rm -rf "$INSTALL_ROOT/$VERSION"
    remove_empty_dir "$INSTALL_ROOT"
    printf 'removed ectropy %s from %s\n' "$VERSION" "$INSTALL_ROOT"
    return
  fi

  rm -f "$bin_path"
  rm -rf "$INSTALL_ROOT"
  remove_empty_dir "$LOCAL_BIN_DIR"
  printf 'removed ectropy from %s and %s\n' "$INSTALL_ROOT" "$bin_path"
}

case "$COMMAND" in
  -h|--help|help)
    cat <<'EOF'
ectropy manager

Usage:
  manage.sh install [--channel stable|beta] [--version vX.Y.Z] [--retain[=true|false]]
  manage.sh uninstall [--version vX.Y.Z]

install leaves exactly one version on disk. Earlier versions are removed once
the new binary is linked and answers --version, and each removal is named.
Rolling back is install --version <older>, which fetches that version again;
released artifacts are immutable and always retrievable. Pass --retain to keep
what is already there.

Options:
  --public-url <url>     release metadata and artifact base URL
  --install-root <path>  versioned install root
  --bin-dir <path>       directory for the ectropy shim/link

Environment:
  ECTROPY_RELEASES_PUBLIC_URL
  ECTROPY_CHANNEL
  ECTROPY_VERSION
  ECTROPY_INSTALL_ROOT
  ECTROPY_LOCAL_BIN_DIR
  ECTROPY_RETAIN
EOF
    ;;
  install) install_ectropy ;;
  uninstall) uninstall_ectropy ;;
  *)
    echo "unknown command: $COMMAND" >&2
    exit 1
    ;;
esac
