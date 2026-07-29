#!/usr/bin/env bash
set -euo pipefail

for name in ECTROPY_RELEASES_S3_AK ECTROPY_RELEASES_S3_SK ECTROPY_RELEASES_S3_BUCKET ECTROPY_RELEASES_S3_URL RELEASE_CHANNEL RELEASE_VERSION; do
  if [ -z "${!name:-}" ]; then
    echo "$name is required" >&2
    exit 1
  fi
done

prefix="$RELEASE_CHANNEL/versions/$RELEASE_VERSION"
for name in \
  ectropy-x86_64-unknown-linux-gnu.tar.gz \
  ectropy-aarch64-apple-darwin.tar.gz \
  ectropy-skill.tar.gz \
  ectropy-x86_64-pc-windows-msvc.zip \
  checksums.txt \
  manage.sh \
  manage.ps1 \
  metadata.json; do
  if held=$(AWS_ACCESS_KEY_ID="$ECTROPY_RELEASES_S3_AK" \
    AWS_SECRET_ACCESS_KEY="$ECTROPY_RELEASES_S3_SK" \
    AWS_DEFAULT_REGION=auto \
    AWS_EC2_METADATA_DISABLED=true \
    aws --endpoint-url "${ECTROPY_RELEASES_S3_URL%/}" s3api head-object \
      --bucket "$ECTROPY_RELEASES_S3_BUCKET" \
      --key "$prefix/$name" \
      --no-cli-pager 2>&1); then
    echo "immutable object already exists: $prefix/$name" >&2
    exit 1
  fi
  case "$held" in
    *404*|*NotFound*|*"Not Found"*) ;;
    *)
      echo "cannot verify immutable object absence for $prefix/$name: $held" >&2
      exit 1
      ;;
  esac
done
