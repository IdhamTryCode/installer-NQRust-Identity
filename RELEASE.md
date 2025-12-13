# Release Guide

This project ships Linux release artifacts automatically when a GitHub Release is published. Follow the steps below to cut a release cleanly and reproduce what the workflow does.

> Note: the shipped binary/apt package name is `nqrust-analytics` (previously `installer-analytics`).

## Prerequisites
- Write access to the repository to publish releases.
- Updated version in `Cargo.toml` (and committed) before publishing.
- A short release summary for the GitHub Release body.

## How to Cut a Release
1. Bump the crate version: `cargo set-version <new-version>` (or edit `Cargo.toml` manually) and commit.
2. Optionally run sanity checks locally: `cargo fmt`, `cargo check`, `cargo test`, `cargo deb`.
3. Push changes to `main`.
4. In GitHub, draft a new Release with tag `v<new-version>` (create the tag in the UI if it does not exist), add notes, and click **Publish release**.
5. The workflow `.github/workflows/release.yml` triggers on the publish event and builds/upload artifacts automatically.

## What the Workflow Builds
- `nqrust-analytics-linux-amd64.tar.gz` — tarball containing the release binary.
- `nqrust-analytics-linux-amd64` — raw ELF binary for convenience.
- `nqrust-analytics_*.deb` — Debian package produced by `cargo deb` (versioned).
- `nqrust-analytics_amd64.deb` — stable alias for the latest amd64 package (used by the install script).
- `SHA256SUMS` — checksums for all artifacts.

Artifacts are attached to the GitHub Release once the job succeeds.

## Verifying Artifacts Locally
- Verify checksums:
  ```bash
  sha256sum -c SHA256SUMS
  ```
- Inspect the Debian package contents:
  ```bash
  dpkg -c nqrust-analytics_*.deb
  ```
- Install the Debian package:
  ```bash
  sudo dpkg -i nqrust-analytics_*.deb
  ```
- One-liner installer (uses the stable alias):
  ```bash
  curl -fsSL https://raw.githubusercontent.com/NexusQuantum/installer-NQRust-Analytics/main/scripts/install/install.sh | bash
  ```

## Troubleshooting
- If the workflow fails, open the Actions run for the release and inspect the failing step (build, `cargo deb`, packaging, or upload).
- Ensure `Cargo.toml` version matches the release tag; mismatches can be confusing for consumers.
