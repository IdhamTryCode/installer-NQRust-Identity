# Airgapped Installation Guide

Complete guide for building and using the NQRust Analytics airgapped installer.

## Overview

The airgapped installer is a **self-extracting single binary** (~3-4 GB) that contains:
- NQRust Analytics installer binary
- All Docker images (analytics-engine, analytics-ui, analytics-service, ibis, qdrant, postgres)
- No internet connection required for installation

---

## Building the Airgapped Binary

### Prerequisites

**On the build machine (with internet):**
- Docker and Docker Compose installed
- Rust toolchain installed
- GitHub Container Registry access (for pulling images)
- ~10 GB free disk space

### Build Steps

1. **Clone the repository:**
   ```bash
   git clone https://github.com/NexusQuantum/installer-NQRust-Analytics.git
   cd installer-NQRust-Analytics
   git checkout airgapped-single-binary
   ```

2. **Login to GitHub Container Registry:**
   ```bash
   docker login ghcr.io
   # Username: your-github-username
   # Password: your-personal-access-token
   ```

3. **Run the build script:**
   ```bash
   chmod +x scripts/airgapped/build-single-binary.sh
   ./scripts/airgapped/build-single-binary.sh
   ```

   This will:
   - Build the Rust binary (`cargo build --release`)
   - Pull all 6 Docker images from registries
   - Save images to compressed tar.gz files
   - Bundle everything into a single payload
   - Create self-extracting binary: `nqrust-analytics-airgapped`

4. **Verify the build:**
   ```bash
   # Check file size (should be ~2.5-3.5 GB)
   ls -lh nqrust-analytics-airgapped
   
   # Verify checksum
   sha256sum -c nqrust-analytics-airgapped.sha256
   ```

### Build Output

```
nqrust-analytics-airgapped         (~3.2 GB) - Self-extracting binary
nqrust-analytics-airgapped.sha256  - Checksum file
build/
├── images/                        - Individual image tar.gz files
│   ├── analytics-engine.tar.gz
│   ├── analytics-ui.tar.gz
│   ├── analytics-service.tar.gz
│   ├── analytics-engine-ibis.tar.gz
│   ├── qdrant.tar.gz
│   ├── postgres.tar.gz
│   ├── manifest.json
│   └── SHA256SUMS
└── payload.tar.gz                 (~2.5 GB) - Combined payload
```

---

## Transferring to Airgapped Environment

### Transfer Methods

**Option 1: USB Drive**
```bash
# Copy to USB
cp nqrust-analytics-airgapped /media/usb/
cp nqrust-analytics-airgapped.sha256 /media/usb/

# On airgapped machine
cp /media/usb/nqrust-analytics-airgapped ~/
cp /media/usb/nqrust-analytics-airgapped.sha256 ~/
```

**Option 2: SCP (if airgapped network has internal connectivity)**
```bash
scp nqrust-analytics-airgapped user@airgapped-host:~/
scp nqrust-analytics-airgapped.sha256 user@airgapped-host:~/
```

**Option 3: Physical Media**
- Burn to DVD/Blu-ray
- Use external HDD/SSD

---

## Docker Airgapped Installer (Optional)

If the airgapped VM **does not have Docker** yet, use this installer to install the full Docker stack offline:

- **Docker CE** (engine + CLI)
- **Docker Compose v2** (`docker compose` plugin)
- **Docker Buildx** (BuildKit)
- **containerd**

### Building the Docker Airgapped Bundle (machine with internet)

**Prerequisites:** Docker and `curl`/`gpg` on the build machine.

1. **Download packages for your target distro:**
   ```bash
   chmod +x scripts/airgapped/download-docker-packages.sh
   ./scripts/airgapped/download-docker-packages.sh ubuntu24.04
   ```
   Supported: `ubuntu24.04`, `ubuntu22.04`, `debian12`.

2. **Or build a full bundle (multiple distros + tarball):**
   ```bash
   chmod +x scripts/airgapped/build-docker-airgapped-bundle.sh
   ./scripts/airgapped/build-docker-airgapped-bundle.sh ubuntu24.04
   # Or: ./scripts/airgapped/build-docker-airgapped-bundle.sh "ubuntu24.04 ubuntu22.04"
   ```

3. **Output:**
   - `build/docker-packages/<distro>/` — folder with `.deb` files and `install-docker-offline.sh`
   - Or `build/docker-airgapped-YYYYMMDD.tar.gz` — tarball to transfer

### Installing Docker on the Airgapped VM

1. Transfer the folder (or tarball) to the VM (USB/SCP).
2. If you transferred a tarball:
   ```bash
   tar xzf docker-airgapped-*.tar.gz
   cd docker-packages/ubuntu24.04   # or your distro
   ```
3. Run the installer:
   ```bash
   chmod +x install-docker-offline.sh
   sudo ./install-docker-offline.sh
   ```
4. **Add your user to the `docker` group** (required for NQRust Analytics installer to access the daemon):
   ```bash
   sudo usermod -aG docker $USER
   ```
   Then log out and back in.

**Note:** The VM must match the distro you downloaded (e.g. Ubuntu 24.04 for `ubuntu24.04`). The script uses Docker on the build machine to download all required `.deb` packages and dependencies.

---

## Installation on Airgapped Machine

### Prerequisites

**On the airgapped machine (Docker stack required by NQRust Analytics):**
- **Docker** (engine + CLI)
- **Docker Compose v2** (`docker compose` — Compose v2 plugin)
- **Docker Buildx** (BuildKit)
- **Access to Docker daemon:** user in `docker` group or use `sudo`
- ~10 GB free disk space (for extraction and Docker images)
- Linux OS (tested on Ubuntu 20.04+, Debian 11+)

If Docker is not installed, use the [Docker Airgapped Installer](#docker-airgapped-installer-optional) above; it installs Docker CE, Compose v2, and Buildx.

### Installation Steps

1. **Verify the binary:**
   ```bash
   sha256sum -c nqrust-analytics-airgapped.sha256
   ```
   Should output: `nqrust-analytics-airgapped: OK`

2. **Make executable:**
   ```bash
   chmod +x nqrust-analytics-airgapped
   ```

3. **Run the installer:**
   ```bash
   ./nqrust-analytics-airgapped install
   ```

### What Happens During Installation

```
1. Airgapped mode detected
   ├─ Check if Docker images already loaded
   └─ If not loaded, proceed to extraction

2. Extract embedded Docker images
   ├─ Locate payload marker in binary
   ├─ Stream extract to /tmp/nqrust-*
   ├─ Show progress bar
   └─ Extract ~2.5 GB payload

3. Load images to Docker
   ├─ Load analytics-engine.tar.gz
   ├─ Load analytics-ui.tar.gz
   ├─ Load analytics-service.tar.gz
   ├─ Load analytics-engine-ibis.tar.gz
   ├─ Load qdrant.tar.gz
   └─ Load postgres.tar.gz

4. Cleanup temporary files
   └─ Delete /tmp/nqrust-* directory

5. Run normal installer TUI
   ├─ Generate .env file
   ├─ Select AI provider config
   └─ Deploy with docker compose up
```

### Expected Output

```
🔒 Airgapped mode detected
📦 Extracting embedded Docker images...
  Locating payload...
  Payload size: 2.45 GB
  Extracting...
  [########################################] 2.45 GB/2.45 GB (00:02)
  Extraction complete

🐳 Loading images to Docker...
  Loading 6 Docker images...
  [1/6] ghcr.io/nexusquantum/analytics-engine:latest
    Loading ghcr.io/nexusquantum/analytics-engine:latest...
  [2/6] ghcr.io/nexusquantum/analytics-ui:latest
    Loading ghcr.io/nexusquantum/analytics-ui:latest...
  ...
  ✓ All images loaded successfully

🧹 Cleaning up temporary files...
✓ Airgapped setup complete!

[Normal installer TUI starts...]
```

---

## Verification

### Verify Docker Images Loaded

```bash
docker images | grep -E 'nexusquantum|qdrant|postgres'
```

Expected output:
```
ghcr.io/nexusquantum/analytics-engine       latest    ...
ghcr.io/nexusquantum/analytics-ui           latest    ...
ghcr.io/nexusquantum/analytics-service      latest    ...
ghcr.io/nexusquantum/analytics-engine-ibis  latest    ...
qdrant/qdrant                               v1.11.0   ...
postgres                                    15        ...
```

### Verify Services Running

```bash
docker compose ps
```

All services should be in `Up` state.

### Access the Application

```bash
curl http://localhost:3000
```

Or open in browser: `http://localhost:3000`

---

## Troubleshooting

### Issue: "Docker is not installed or not in PATH"

**Solution:**
```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Start Docker daemon
sudo systemctl start docker
sudo systemctl enable docker
```

### Issue: "Docker daemon is not running"

**Solution:**
```bash
sudo systemctl start docker
```

### Issue: "Payload marker not found in binary"

**Cause:** Binary is corrupted or not the airgapped version

**Solution:**
- Verify checksum: `sha256sum -c nqrust-analytics-airgapped.sha256`
- Re-transfer from build machine
- Ensure you're using the airgapped binary, not the regular one

### Issue: "No space left on device"

**Cause:** Insufficient disk space for extraction

**Solution:**
```bash
# Check available space
df -h /tmp

# Need at least 10 GB free
# Clean up or mount larger partition to /tmp
```

### Issue: Images already loaded, but want to re-extract

**Solution:**
```bash
# Remove existing images
docker rmi ghcr.io/nexusquantum/analytics-engine:latest
docker rmi ghcr.io/nexusquantum/analytics-ui:latest
# ... (remove all 6 images)

# Run installer again
./nqrust-analytics-airgapped install
```

---

## Advanced Usage

### Skip Auto-Detection (Force Extraction)

If images are already loaded but you want to force re-extraction:

```bash
# Remove all images first
docker rmi $(docker images -q 'ghcr.io/nexusquantum/*')
docker rmi qdrant/qdrant:v1.11.0
docker rmi postgres:15

# Then run installer
./nqrust-analytics-airgapped install
```

### Manual Extraction (for debugging)

```bash
# Extract payload only (without loading to Docker)
# This requires modifying the binary or using a hex editor
# Not recommended for normal use
```

---

## FAQ

**Q: How large is the airgapped binary?**  
A: ~2.5-3.5 GB (varies based on Docker image sizes)

**Q: Can I use this on ARM64?**  
A: Currently only `linux/amd64` is supported. ARM64 support can be added if needed.

**Q: How do I update to a newer version?**  
A: Build a new airgapped binary from the updated repository and transfer it to the airgapped machine.

**Q: Does this work on Windows?**  
A: No, currently Linux only. Windows support would require significant changes.

**Q: Can I customize which images are included?**  
A: Yes, edit `scripts/airgapped/save-images.sh` and `src/airgapped/docker.rs` to modify the image list.

**Q: What if I only have 4 GB RAM?**  
A: Should work fine. The extractor uses streaming with 8 KB buffers, so memory usage is minimal.

---

## Security Considerations

1. **Verify checksums** before running the binary
2. **Use secure transfer** methods (encrypted USB, SCP with keys)
3. **Scan for malware** if transferring via removable media
4. **Keep build machine secure** - it has access to your GitHub credentials

---

## Support

For issues or questions:
- GitHub Issues: [NexusQuantum/installer-NQRust-Analytics](https://github.com/NexusQuantum/installer-NQRust-Analytics/issues)
- Email: idhammultazam7@gmail.com
