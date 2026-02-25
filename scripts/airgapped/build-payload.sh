#!/usr/bin/env bash
# build-payload.sh — Bundle Docker image tarballs into a single payload archive
set -euo pipefail

IMAGES_DIR="${1:-build/images}"
OUTPUT_PAYLOAD="${2:-build/payload.tar.gz}"

if [ ! -f "${IMAGES_DIR}/manifest.json" ]; then
  echo "❌ No manifest.json found in ${IMAGES_DIR}. Run save-images.sh first."
  exit 1
fi

mkdir -p "$(dirname "${OUTPUT_PAYLOAD}")"

echo "📦 Creating payload archive from ${IMAGES_DIR}..."
tar -C "${IMAGES_DIR}" -czf "${OUTPUT_PAYLOAD}" .

SIZE=$(stat -c%s "${OUTPUT_PAYLOAD}")
echo "✅ Payload created: ${OUTPUT_PAYLOAD} ($(numfmt --to=iec-i --suffix=B ${SIZE}))"
