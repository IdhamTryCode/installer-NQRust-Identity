#!/usr/bin/env bash
# save-images.sh — Pull and save Docker images for nqrust-identity airgapped bundle
set -euo pipefail

IMAGES_DIR="${1:-build/images}"
mkdir -p "$IMAGES_DIR"

# Resolve latest nqrust-identity tag from GitHub Releases
OWNER="NexusQuantum"
REPO="nqrust-identity"
IDENTITY_TAG=$(curl -s "https://api.github.com/repos/${OWNER}/${REPO}/releases/latest" \
  -H "Accept: application/vnd.github+json" \
  | jq -r '.tag_name // "latest"')
echo "Using nqrust-identity tag: ${IDENTITY_TAG}"

# NOTE: Output filenames MUST match REQUIRED_IMAGES in src/airgapped/docker.rs
declare -A IMAGE_FILES
IMAGE_FILES["ghcr.io/nexusquantum/nqrust-identity:${IDENTITY_TAG}"]="nqrust-identity.tar.gz"
IMAGE_FILES["postgres:16-alpine"]="postgres.tar.gz"

MANIFEST_ENTRIES="[]"

for IMAGE in "ghcr.io/nexusquantum/nqrust-identity:${IDENTITY_TAG}" "postgres:16-alpine"; do
  FILENAME="${IMAGE_FILES[$IMAGE]}"
  OUTPUT="${IMAGES_DIR}/${FILENAME}"

  echo "Pulling ${IMAGE}..."
  docker pull "${IMAGE}"

  echo "Saving ${IMAGE} → ${OUTPUT}..."
  docker save "${IMAGE}" | gzip -9 > "${OUTPUT}"

  SIZE=$(stat -c%s "${OUTPUT}")
  CHECKSUM=$(sha256sum "${OUTPUT}" | cut -d' ' -f1)

  echo "${CHECKSUM}  ${FILENAME}" >> "${IMAGES_DIR}/SHA256SUMS"

  MANIFEST_ENTRIES=$(echo "$MANIFEST_ENTRIES" | jq \
    --arg name "$IMAGE" \
    --arg file "$FILENAME" \
    --arg size "$SIZE" \
    --arg sha256 "$CHECKSUM" \
    '. + [{"name": $name, "file": $file, "size": $size, "sha256": $sha256}]')
done

# Write manifest
echo "{\"images\": ${MANIFEST_ENTRIES}, \"identity_tag\": \"${IDENTITY_TAG}\"}" \
  | jq . > "${IMAGES_DIR}/manifest.json"

echo "✅ Images saved to ${IMAGES_DIR}/"
cat "${IMAGES_DIR}/manifest.json" | jq -r '.images[] | "  - \(.name) → \(.file) (\(.size | tonumber / 1048576 | floor)MB)"'
