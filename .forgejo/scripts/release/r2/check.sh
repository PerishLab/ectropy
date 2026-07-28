#!/usr/bin/env bash
set -euo pipefail

for name in ECTROPY_RELEASES_S3_AK ECTROPY_RELEASES_S3_SK ECTROPY_RELEASES_S3_BUCKET ECTROPY_RELEASES_S3_URL ECTROPY_RELEASES_PUBLIC_URL RELEASE_CHANNEL R2_ACCESS_PROBE_NAME; do
  if [ -z "${!name:-}" ]; then
    echo "$name is required" >&2
    exit 1
  fi
done

temp=${ECTROPY_RELEASE_TEMP:-${RUNNER_TEMP:-.local/tmp}}
run_id=${CI_RUN_ID:-local}
sha=${CI_COMMIT:-unknown}
mkdir -p "$temp"
probe_file="$temp/ectropy-r2-access.txt"
probe_key="$RELEASE_CHANNEL/.ci-access-check/$R2_ACCESS_PROBE_NAME.txt"
printf 'run=%s\nsha=%s\nchannel=%s\n' "$run_id" "$sha" "$RELEASE_CHANNEL" > "$probe_file"

AWS_ACCESS_KEY_ID="$ECTROPY_RELEASES_S3_AK" \
AWS_SECRET_ACCESS_KEY="$ECTROPY_RELEASES_S3_SK" \
AWS_DEFAULT_REGION=auto \
AWS_EC2_METADATA_DISABLED=true \
aws --endpoint-url "${ECTROPY_RELEASES_S3_URL%/}" s3api put-object \
  --bucket "$ECTROPY_RELEASES_S3_BUCKET" \
  --key "$probe_key" \
  --body "$probe_file" \
  --content-type "text/plain; charset=utf-8" \
  --cache-control "no-store" \
  --no-cli-pager >/dev/null
