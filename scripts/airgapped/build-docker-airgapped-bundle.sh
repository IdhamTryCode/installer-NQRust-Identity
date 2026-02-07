#!/usr/bin/env bash
set -euo pipefail

# build-docker-airgapped-bundle.sh
# Build Docker airgapped installer: download .deb for selected distros and create
# a tarball ready to transfer to airgapped VM.
# Run on a machine WITH internet and Docker.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
DOCKER_PACKAGES_DIR="${BUILD_DIR}/docker-packages"

# Distros to bundle: space-separated, e.g. "ubuntu24.04 ubuntu22.04"
DISTROS="${1:-ubuntu24.04}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step()  { echo -e "${BLUE}[STEP]${NC} $1"; }

echo ""
echo "=========================================="
echo "  Docker Airgapped Bundle Builder"
echo "=========================================="
echo ""
log_info "Distros: ${DISTROS}"
log_info "Output dir: ${DOCKER_PACKAGES_DIR}"
echo ""

for distro in ${DISTROS}; do
    log_step "Building bundle for: ${distro}"
    "${SCRIPT_DIR}/download-docker-packages.sh" "${distro}"
    echo ""
done

# Create tarball for easy transfer
BUNDLE_NAME="docker-airgapped-$(date +%Y%m%d).tar.gz"
BUNDLE_PATH="${BUILD_DIR}/${BUNDLE_NAME}"
log_step "Creating tarball: ${BUNDLE_PATH}"
tar czf "${BUNDLE_PATH}" -C "${BUILD_DIR}" docker-packages
BUNDLE_SIZE=$(du -h "${BUNDLE_PATH}" | cut -f1)

log_info ""
log_info "=========================================="
log_info "Docker airgapped bundle ready"
log_info "=========================================="
log_info "Tarball: ${BUNDLE_PATH}"
log_info "Size: ${BUNDLE_SIZE}"
log_info ""
log_info "Transfer to airgapped VM (USB/SCP), then:"
log_info "  tar xzf docker-airgapped-*.tar.gz"
log_info "  cd docker-packages/<distro>   # e.g. ubuntu24.04"
log_info "  sudo ./install-docker-offline.sh"
log_info ""
