#!/usr/bin/env bash
# download-docker-packages.sh — Download Docker .deb packages for offline installation
set -euo pipefail

DISTRO="${1:-ubuntu24.04}"
OUTPUT_DIR="${2:-build/docker-packages/${DISTRO}}"

mkdir -p "${OUTPUT_DIR}"

# Determine Ubuntu/Debian codename
case "${DISTRO}" in
  ubuntu24.04) CODENAME="noble" ;;
  ubuntu22.04) CODENAME="jammy" ;;
  ubuntu20.04) CODENAME="focal" ;;
  debian12)    CODENAME="bookworm" ;;
  *)
    echo "❌ Unsupported distro: ${DISTRO}"
    echo "   Supported: ubuntu24.04, ubuntu22.04, ubuntu20.04, debian12"
    exit 1 ;;
esac

echo "📦 Downloading Docker packages for ${DISTRO} (${CODENAME})..."

# Add Docker GPG key and repo temporarily, then download packages
docker run --rm \
  -v "${PWD}/${OUTPUT_DIR}:/output" \
  "${DISTRO/-/:}" bash -c "
    set -e
    DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq ca-certificates curl gnupg

    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/$(. /etc/os-release && echo \$ID)/gpg \
      | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
    chmod a+r /etc/apt/keyrings/docker.gpg

    echo \"deb [arch=\$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
      https://download.docker.com/linux/\$(. /etc/os-release && echo \$ID) \
      ${CODENAME} stable\" \
      > /etc/apt/sources.list.d/docker.list

    apt-get update -qq

    # Download packages without installing
    cd /output
    apt-get download -y \
      docker-ce \
      docker-ce-cli \
      containerd.io \
      docker-buildx-plugin \
      docker-compose-plugin
    echo 'Done'
  "

# Copy install script
cp scripts/airgapped/install-docker-offline.sh "${OUTPUT_DIR}/"
chmod +x "${OUTPUT_DIR}/install-docker-offline.sh"

echo "✅ Docker packages downloaded to ${OUTPUT_DIR}/"
ls -lh "${OUTPUT_DIR}/"
