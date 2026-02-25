#!/usr/bin/env bash
# build-docker-airgapped-bundle.sh — Build Docker offline bundle for a given distro
set -euo pipefail

DISTRO="${1:-ubuntu24.04}"
OUTPUT_DIR="build/docker-packages"

echo "🐳 Building Docker airgapped bundle for ${DISTRO}..."

chmod +x scripts/airgapped/download-docker-packages.sh
./scripts/airgapped/download-docker-packages.sh "${DISTRO}" "${OUTPUT_DIR}/${DISTRO}"

# Create tarball
echo "📦 Creating bundle tarball..."
BUNDLE_FILE="build/docker-airgapped-${DISTRO}.tar.gz"
tar -C "${OUTPUT_DIR}" -czf "${BUNDLE_FILE}" "${DISTRO}"

SIZE=$(stat -c%s "${BUNDLE_FILE}")
echo "✅ Bundle created: ${BUNDLE_FILE} ($(numfmt --to=iec-i --suffix=B ${SIZE}))"
