#!/usr/bin/env bash
# download-docker-packages.sh — Download Docker .deb packages for offline installation
# NOTE: This script must run on the matching Ubuntu/Debian host, not inside docker run.
set -euo pipefail

DISTRO="${1:-ubuntu24.04}"
OUTPUT_DIR="${2:-build/docker-packages/${DISTRO}}"

mkdir -p "${OUTPUT_DIR}"

echo "📦 Downloading Docker packages for ${DISTRO}..."

# Determine codename
case "${DISTRO}" in
  ubuntu24.04) CODENAME="noble";   OS_ID="ubuntu" ;;
  ubuntu22.04) CODENAME="jammy";   OS_ID="ubuntu" ;;
  ubuntu20.04) CODENAME="focal";   OS_ID="ubuntu" ;;
  debian12)    CODENAME="bookworm"; OS_ID="debian" ;;
  *)
    echo "❌ Unsupported distro: ${DISTRO}"
    echo "   Supported: ubuntu24.04, ubuntu22.04, ubuntu20.04, debian12"
    exit 1 ;;
esac

# Add Docker's official GPG key
install -m 0755 -d /etc/apt/keyrings
curl -fsSL "https://download.docker.com/linux/${OS_ID}/gpg" \
  | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

# Add Docker repository
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
https://download.docker.com/linux/${OS_ID} ${CODENAME} stable" \
  | tee /etc/apt/sources.list.d/docker.list > /dev/null

apt-get update -qq

# Download packages without installing
echo "Downloading to ${OUTPUT_DIR}..."
cd "${OUTPUT_DIR}"
apt-get download \
  docker-ce \
  docker-ce-cli \
  containerd.io \
  docker-buildx-plugin \
  docker-compose-plugin

echo "✅ Docker packages downloaded to ${OUTPUT_DIR}/"
ls -lh "${OUTPUT_DIR}/"
