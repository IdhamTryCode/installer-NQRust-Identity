#!/usr/bin/env bash
set -euo pipefail

# install-docker-offline.sh
# Install Docker CE from pre-downloaded .deb packages (airgapped/offline).
# Run on the target VM. No internet required.
#
# Usage:
#   sudo ./install-docker-offline.sh [DIR]
# If DIR is omitted, uses the directory containing this script.

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info()  { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

if [ "$(id -u)" -ne 0 ]; then
    log_error "This script must be run as root (sudo ./install-docker-offline.sh)"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEB_DIR="${1:-${SCRIPT_DIR}}"

if [ ! -d "${DEB_DIR}" ]; then
    log_error "Directory not found: ${DEB_DIR}"
    exit 1
fi

DEB_COUNT=$(find "${DEB_DIR}" -maxdepth 1 -name "*.deb" 2>/dev/null | wc -l)
if [ "${DEB_COUNT}" -eq 0 ]; then
    log_error "No .deb files found in ${DEB_DIR}"
    exit 1
fi

echo ""
echo "=========================================="
echo "  Docker Offline Installer"
echo "=========================================="
echo ""
log_info "Installing from: ${DEB_DIR}"
log_info "Packages: ${DEB_COUNT} .deb files"
echo ""

# Install all .deb (run twice so second pass satisfies newly revealed deps)
log_info "Installing packages with dpkg..."
cd "${DEB_DIR}"
dpkg -i *.deb 2>/dev/null || true
dpkg -i *.deb 2>/dev/null || true

# Check for broken packages
if dpkg -s docker-ce &>/dev/null; then
    log_info "Docker CE installed successfully"
else
    log_warn "Some packages may have failed. Ensure all dependency .deb files are in this folder."
    log_warn "Re-run download-docker-packages.sh with Docker available to get full dependency set."
fi

# Start and enable Docker
log_info "Enabling and starting Docker service..."
if systemctl unmask docker.service 2>/dev/null; then true; fi
if systemctl enable docker.service 2>/dev/null; then true; fi
if systemctl start docker.service 2>/dev/null; then
    log_info "Docker daemon started"
else
    log_warn "Could not start Docker. Run manually: systemctl start docker"
fi

# Verify
if command -v docker &>/dev/null; then
    if docker info &>/dev/null 2>&1; then
        log_info "Docker is running: $(docker --version)"
    else
        log_warn "Docker installed but daemon may not be running. Try: systemctl start docker"
    fi
else
    log_warn "docker command not in PATH"
fi

if command -v docker &>/dev/null && docker compose version &>/dev/null 2>&1; then
    log_info "Docker Compose (v2): $(docker compose version --short 2>/dev/null || docker compose version)"
fi

if command -v docker &>/dev/null && docker buildx version &>/dev/null 2>&1; then
    log_info "Docker Buildx (BuildKit): $(docker buildx version 2>/dev/null | head -1)"
else
    log_warn "Docker Buildx not found. NQRust Analytics installer requires it. Ensure docker-buildx-plugin .deb was in the bundle."
fi

echo ""
log_info "=========================================="
log_info "Installation complete"
log_info "=========================================="
log_info "NQRust Analytics requires access to the Docker daemon (sudo or docker group)."
log_info "Add your user to the 'docker' group:"
log_info "  sudo usermod -aG docker \$USER"
log_info "  (then log out and back in)"
echo ""
