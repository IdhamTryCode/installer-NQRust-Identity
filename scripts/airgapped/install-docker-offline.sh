#!/usr/bin/env bash
# install-docker-offline.sh — Install Docker from local .deb packages (no internet required)
# Run this script from the directory containing the .deb package files.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "$(id -u)" != "0" ]; then
  echo "❌ This script must be run as root (sudo ./install-docker-offline.sh)"
  exit 1
fi

echo "🐳 Installing Docker from offline packages..."
echo "   Package directory: ${SCRIPT_DIR}"

# Check for .deb files
DEB_COUNT=$(ls "${SCRIPT_DIR}"/*.deb 2>/dev/null | wc -l)
if [ "${DEB_COUNT}" -eq 0 ]; then
  echo "❌ No .deb files found in ${SCRIPT_DIR}"
  exit 1
fi

echo "   Found ${DEB_COUNT} package(s)"

# Install all packages
dpkg -i "${SCRIPT_DIR}"/*.deb || true

# Fix any dependency issues
apt-get install -f -y 2>/dev/null || true

# Enable and start Docker
systemctl enable docker
systemctl start docker

# Verify installation
if docker --version && docker compose version; then
  echo ""
  echo "✅ Docker installed successfully!"
  echo ""
  echo "📌 Next steps:"
  echo "   1. Add your user to the docker group:  sudo usermod -aG docker \$USER"
  echo "   2. Log out and log back in"
  echo "   3. Verify:  docker run hello-world"
else
  echo "❌ Docker installation may have failed. Check the output above."
  exit 1
fi
