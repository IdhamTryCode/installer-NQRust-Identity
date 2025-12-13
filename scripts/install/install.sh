#!/usr/bin/env bash
set -euo pipefail

REPO="NexusQuantum/installer-NQRust-Analytics"
BIN="nqrust-analytics"

ARCH_RAW="$(uname -m)"
case "${ARCH_RAW}" in
  x86_64|amd64)
    ARCH="amd64"
    ;;
  aarch64|arm64)
    echo "[ERROR] arm64 builds are not published yet. Please use an x86_64 machine for now." >&2
    exit 1
    ;;
  *)
    echo "[ERROR] Unsupported architecture: ${ARCH_RAW}" >&2
    exit 1
    ;;
esac

BASE="https://github.com/${REPO}/releases/latest/download"
DEB_NAME="${BIN}_${ARCH}.deb"
DEB_URL="${BASE}/${DEB_NAME}"
SUMS_URL="${BASE}/SHA256SUMS"
TMPDIR="$(mktemp -d)"
cleanup() { rm -rf "${TMPDIR}"; }
trap cleanup EXIT

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "[ERROR] Missing required command: $1" >&2
    exit 1
  }
}

need_cmd curl
need_cmd sha256sum
need_cmd sudo

fetch() {
  local url="$1" out="$2"
  echo "[INFO] Downloading ${url}"
  curl -fL --retry 3 --retry-delay 1 -o "${out}" "${url}"
}

fetch "${DEB_URL}" "${TMPDIR}/${DEB_NAME}"
fetch "${SUMS_URL}" "${TMPDIR}/SHA256SUMS"

echo "[INFO] Verifying checksum"
if ! (cd "${TMPDIR}" && grep "${DEB_NAME}" SHA256SUMS | sha256sum -c -); then
  echo "[ERROR] Checksum verification failed" >&2
  exit 1
fi

echo "[INFO] Installing package"
if command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update -y
  sudo apt-get install -y "${TMPDIR}/${DEB_NAME}"
else
  sudo dpkg -i "${TMPDIR}/${DEB_NAME}"
  sudo apt-get install -f -y || true
fi

echo "[INFO] Installed. Run: ${BIN} install"
