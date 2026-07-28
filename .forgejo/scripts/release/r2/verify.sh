#!/usr/bin/env bash
set -euo pipefail

for name in ECTROPY_RELEASES_PUBLIC_URL RELEASE_CHANNEL RELEASE_VERSION R2_METADATA_URL; do
  if [ -z "${!name:-}" ]; then
    echo "$name is required" >&2
    exit 1
  fi
done

run_id=${CI_RUN_ID:-local}
temp=${ECTROPY_RELEASE_TEMP:-${RUNNER_TEMP:-.local/tmp}}
mkdir -p "$temp"
metadata="$temp/ectropy-release-metadata.json"
curl -fsSL "$R2_METADATA_URL?run=$run_id" -o "$metadata"

public_url="${ECTROPY_RELEASES_PUBLIC_URL%/}"
jq -e \
  --arg channel "$RELEASE_CHANNEL" \
  --arg version "$RELEASE_VERSION" \
  --arg unix "$public_url/manage.sh" \
  '
  (.channel == $channel)
  and (.releaseVersion == $version)
  and (.manage.unix == $unix)
  and (.manage.snapshots.unix.url == ($unix | sub("/manage.sh$"; "/" + $channel + "/versions/" + $version + "/manage.sh")))
  and (if .channel == "beta"
        then (.betaVersion == $version)
          and (.baseVersion | (type == "string") and (length > 0))
          and (.betaNumber | type == "number")
          and (("v" + .baseVersion + "-beta." + (.betaNumber | tostring)) == $version)
        else true end)
  and (.artifacts | to_entries | all(
    (.value.url | (type == "string") and (length > 0))
    and (.value.sha256 | test("^[0-9a-f]{64}$"))
    and (.value.size | (type == "number") and (. > 0))
  ))
  and (.manage.snapshots | to_entries | all(
    (.value.url | (type == "string") and (length > 0))
    and (.value.sha256 | test("^[0-9a-f]{64}$"))
    and (.value.size | (type == "number") and (. > 0))
  ))
  ' "$metadata" >/dev/null || {
  echo "metadata validation failed" >&2
  exit 1
}

bytes="$temp/ectropy-release-bytes"
rm -rf "$bytes"
mkdir -p "$bytes"
jq -r '(.artifacts[], .manage.snapshots[]) | [.name, .url, .sha256] | @tsv' "$metadata" |
  while IFS="$(printf '\t')" read -r name url expected; do
    curl -fsSL "$url?run=$run_id" -o "$bytes/$name"
    actual=$(sha256sum "$bytes/$name" | awk '{print $1}')
    [ "$actual" = "$expected" ] || {
      echo "public checksum mismatch for $name: $actual != $expected" >&2
      exit 1
    }
  done

for url in $(jq -r '.manage.unix, .manage.windows' "$metadata"); do
  curl -fsSI "$url" >/dev/null
done
