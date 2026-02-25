#!/usr/bin/env bash
set -euo pipefail

# build-payload.sh
# Bundle all saved Docker images into a single compressed payload

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
IMAGES_DIR="${BUILD_DIR}/images"
PAYLOAD_FILE="${BUILD_DIR}/payload.tar.gz"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if images directory exists
if [ ! -d "${IMAGES_DIR}" ]; then
    log_error "Images directory not found: ${IMAGES_DIR}"
    log_error "Please run ./scripts/airgapped/save-images.sh first"
    exit 1
fi

# Check if manifest exists
if [ ! -f "${IMAGES_DIR}/manifest.json" ]; then
    log_error "Manifest file not found"
    log_error "Please run ./scripts/airgapped/save-images.sh first"
    exit 1
fi

log_info "Building payload from saved images..."
log_info "Source: ${IMAGES_DIR}"
log_info "Output: ${PAYLOAD_FILE}"
echo ""

# Create payload by tarring and compressing all images
log_info "Compressing images into payload..."
log_info "This may take a few minutes..."

cd "${BUILD_DIR}"
if tar czf payload.tar.gz -C images .; then
    log_info "✓ Payload created successfully"
else
    log_error "Failed to create payload"
    exit 1
fi
cd "${PROJECT_ROOT}"

# Calculate checksum
log_info "Calculating checksum..."
if command -v sha256sum &> /dev/null; then
    CHECKSUM=$(sha256sum "${PAYLOAD_FILE}" | cut -d' ' -f1)
    echo "${CHECKSUM}  payload.tar.gz" > "${BUILD_DIR}/payload.sha256"
elif command -v shasum &> /dev/null; then
    CHECKSUM=$(shasum -a 256 "${PAYLOAD_FILE}" | cut -d' ' -f1)
    echo "${CHECKSUM}  payload.tar.gz" > "${BUILD_DIR}/payload.sha256"
else
    log_error "No SHA256 tool found"
    exit 1
fi

# Summary
PAYLOAD_SIZE=$(du -h "${PAYLOAD_FILE}" | cut -f1)

log_info ""
log_info "=========================================="
log_info "Payload built successfully!"
log_info "=========================================="
log_info "File: ${PAYLOAD_FILE}"
log_info "Size: ${PAYLOAD_SIZE}"
log_info "SHA256: ${CHECKSUM}"
log_info ""
log_info "Next step: Run ./scripts/airgapped/build-single-binary.sh"
