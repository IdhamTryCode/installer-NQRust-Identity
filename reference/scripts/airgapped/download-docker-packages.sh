#!/usr/bin/env bash
set -euo pipefail

# download-docker-packages.sh
# Download Docker CE .deb packages + dependencies for offline/airgapped install.
# Includes: docker-ce, docker-ce-cli, containerd.io, docker-compose-plugin (Compose v2),
#           docker-buildx-plugin (BuildKit). Run on a machine WITH internet.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
DOCKER_PACKAGES_DIR="${BUILD_DIR}/docker-packages"

# Distro: ubuntu24.04 | ubuntu22.04 | debian12
DISTRO="${1:-ubuntu24.04}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step()  { echo -e "${BLUE}[STEP]${NC} $1"; }

case "${DISTRO}" in
    ubuntu24.04)
        OS=ubuntu
        CODENAME=noble
        ;;
    ubuntu22.04)
        OS=ubuntu
        CODENAME=jammy
        ;;
    debian12)
        OS=debian
        CODENAME=bookworm
        ;;
    *)
        log_error "Unsupported DISTRO: ${DISTRO}"
        echo "Usage: $0 [ubuntu24.04|ubuntu22.04|debian12]"
        exit 1
        ;;
esac

OUTPUT_DIR="${DOCKER_PACKAGES_DIR}/${DISTRO}"
mkdir -p "${OUTPUT_DIR}"

echo ""
echo "=========================================="
echo "  Docker Airgapped Package Downloader"
echo "=========================================="
echo ""
log_info "Distro: ${DISTRO} (${OS}/${CODENAME})"
log_info "Output: ${OUTPUT_DIR}"
echo ""

# Prerequisites
for cmd in curl gpg; do
    if ! command -v "${cmd}" &>/dev/null; then
        log_error "Required command not found: ${cmd}. Install it first."
        exit 1
    fi
done

log_step "Downloading Docker packages (and dependencies)..."

# If we're on the same OS, we can use apt-get download. Otherwise we download by URL.
# Prefer: run apt-get update and install --download-only inside a container of the target OS.
if command -v docker &>/dev/null && docker info &>/dev/null 2>&1; then
    log_info "Using Docker container to download packages for ${OS}:${CODENAME}..."
    docker run --rm \
        -v "${OUTPUT_DIR}:/out" \
        -e DEBIAN_FRONTEND=noninteractive \
        "${OS}:${CODENAME}" \
        bash -c '
            apt-get update -qq
            apt-get install -y -qq ca-certificates curl gnupg
            install -m 0755 -d /etc/apt/keyrings
            curl -fsSL "https://download.docker.com/linux/'"${OS}"'/gpg" | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
            echo "deb [arch=amd64 signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/'"${OS}"' '"${CODENAME}"' stable" > /etc/apt/sources.list.d/docker.list
            apt-get update -qq
            apt-get install -y --download-only docker-ce docker-ce-cli containerd.io docker-compose-plugin docker-buildx-plugin
            cp /var/cache/apt/archives/*.deb /out/ 2>/dev/null || true
            chmod -R a+r /out
        '
else
    log_warn "Docker daemon not available. Downloading main .deb packages by URL (no dependency resolution)."
    log_warn "For full dependency set, start Docker Desktop and run this script again."
    BASE_URL="https://download.docker.com/linux/${OS}/dists/${CODENAME}/pool/stable/amd64"
    INDEX=$(curl -sSL "${BASE_URL}/" 2>/dev/null || true)
    for pkg in containerd.io docker-ce-cli docker-ce docker-compose-plugin docker-buildx-plugin; do
        # Extract latest .deb filename for this package (e.g. docker-ce_29.1.5-1~ubuntu.24.04~noble_amd64.deb)
        PKG_DEB=$(echo "${INDEX}" | grep -oE "${pkg}_[^\"'<> ]+\.deb" | sort -V | tail -1)
        if [ -n "${PKG_DEB}" ]; then
            log_info "  Downloading ${PKG_DEB}..."
            curl -fsSL "${BASE_URL}/${PKG_DEB}" -o "${OUTPUT_DIR}/${PKG_DEB}" 2>/dev/null || {
                log_warn "  Failed to download ${PKG_DEB}"
            }
        else
            log_warn "  No package found for ${pkg}"
        fi
    done
    DEB_COUNT=$(find "${OUTPUT_DIR}" -maxdepth 1 -name "*.deb" 2>/dev/null | wc -l)
    if [ "${DEB_COUNT}" -eq 0 ]; then
        log_error "No packages downloaded. Start Docker and re-run, or check network."
        exit 1
    fi
    log_warn "Only main packages downloaded. Target VM may need same distro and existing base deps."
fi

# Copy install script into the bundle
INSTALL_SCRIPT="${SCRIPT_DIR}/install-docker-offline.sh"
if [ -f "${INSTALL_SCRIPT}" ]; then
    cp "${INSTALL_SCRIPT}" "${OUTPUT_DIR}/"
    chmod +x "${OUTPUT_DIR}/install-docker-offline.sh"
    log_info "Installed install-docker-offline.sh into bundle"
fi

# Summary
DEB_COUNT=$(find "${OUTPUT_DIR}" -maxdepth 1 -name "*.deb" 2>/dev/null | wc -l)
TOTAL_SIZE=$(du -sh "${OUTPUT_DIR}" | cut -f1)
log_info ""
log_info "=========================================="
log_info "Download complete: ${DISTRO}"
log_info "=========================================="
log_info "Output: ${OUTPUT_DIR}"
log_info "Packages: ${DEB_COUNT} .deb files"
log_info "Total size: ${TOTAL_SIZE}"
log_info ""
log_info "Next: transfer folder '${OUTPUT_DIR}' to airgapped VM, then run:"
log_info "  cd docker-packages/${DISTRO}"
log_info "  sudo ./install-docker-offline.sh"
log_info ""
