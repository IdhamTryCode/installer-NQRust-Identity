#!/usr/bin/env bash
# build-single-binary.sh — Build airgapped single binary for nqrust-identity
set -euo pipefail

FORCE_REFRESH="${1:-}"
BIN_NAME="nqrust-identity-airgapped"
IMAGES_DIR="build/images"
PAYLOAD_PATH="build/payload.tar.gz"

echo "🔨 Building airgapped single binary for nqrust-identity..."

# Step 1: Pull and save Docker images
echo ""
echo "=== Step 1/3: Saving Docker images ==="
chmod +x scripts/airgapped/save-images.sh

if [ "${FORCE_REFRESH}" == "--force-refresh" ] || [ ! -f "${IMAGES_DIR}/manifest.json" ]; then
  ./scripts/airgapped/save-images.sh "${IMAGES_DIR}"
else
  echo "ℹ Using cached images (use --force-refresh to re-pull)"
fi

# Step 2: Build payload
echo ""
echo "=== Step 2/3: Building payload ==="
chmod +x scripts/airgapped/build-payload.sh
./scripts/airgapped/build-payload.sh "${IMAGES_DIR}" "${PAYLOAD_PATH}"

PAYLOAD_SIZE=$(stat -c%s "${PAYLOAD_PATH}")
echo "Payload size: $(numfmt --to=iec-i --suffix=B ${PAYLOAD_SIZE})"

# Step 3: Build Rust binary in release mode
echo ""
echo "=== Step 3/3: Building Rust binary ==="
cargo build --release

BINARY_PATH="target/release/nqrust-identity"
BINARY_SIZE=$(stat -c%s "${BINARY_PATH}")
echo "Binary size: $(numfmt --to=iec-i --suffix=B ${BINARY_SIZE})"

# Combine binary + marker + payload
echo "📎 Embedding payload into binary..."
MARKER="__NQRUST_PAYLOAD__"

cat "${BINARY_PATH}" \
  <(printf '%s' "${MARKER}") \
  "${PAYLOAD_PATH}" \
  > "${BIN_NAME}"

chmod +x "${BIN_NAME}"

# Generate checksum
sha256sum "${BIN_NAME}" > "${BIN_NAME}.sha256"

FINAL_SIZE=$(stat -c%s "${BIN_NAME}")
echo ""
echo "✅ Airgapped binary built: ${BIN_NAME}"
echo "   Size: $(numfmt --to=iec-i --suffix=B ${FINAL_SIZE})"
echo "   SHA256: $(cat ${BIN_NAME}.sha256 | cut -d' ' -f1)"
