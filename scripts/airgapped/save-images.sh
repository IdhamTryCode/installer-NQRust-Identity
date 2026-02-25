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

IMAGES=(
  "ghcr.io/nexusquantum/nqrust-identity:${IDENTITY_TAG}"
  "postgres:16-alpine"
)

MANIFEST_ENTRIES="[]"

for IMAGE in "${IMAGES[@]}"; do
  echo "Pulling ${IMAGE}..."
  docker pull "${IMAGE}"

  # Sanitize filename
  SAFE_NAME=$(echo "${IMAGE}" | sed 's|/|_|g; s|:|_|g')
  OUTPUT="${IMAGES_DIR}/${SAFE_NAME}.tar.gz"

  echo "Saving ${IMAGE} → ${OUTPUT}..."
  docker save "${IMAGE}" | gzip -9 > "${OUTPUT}"

  SIZE=$(stat -c%s "${OUTPUT}")
  CHECKSUM=$(sha256sum "${OUTPUT}" | cut -d' ' -f1)

  echo "${CHECKSUM}  ${SAFE_NAME}.tar.gz" >> "${IMAGES_DIR}/SHA256SUMS"

  MANIFEST_ENTRIES=$(echo "$MANIFEST_ENTRIES" | jq \
    --arg name "$IMAGE" \
    --arg file "${SAFE_NAME}.tar.gz" \
    --arg size "$SIZE" \
    --arg sha256 "$CHECKSUM" \
    '. + [{"name": $name, "file": $file, "size": $size, "sha256": $sha256}]')
done

# Write manifest
echo "{\"images\": ${MANIFEST_ENTRIES}, \"identity_tag\": \"${IDENTITY_TAG}\"}" \
  | jq . > "${IMAGES_DIR}/manifest.json"

echo "✅ Images saved to ${IMAGES_DIR}/"
cat "${IMAGES_DIR}/manifest.json" | jq -r '.images[] | "  - \(.name) (\(.size | tonumber / 1048576 | floor)MB)"'
