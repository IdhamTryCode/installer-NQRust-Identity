// airgapped/extractor.rs
// Payload extraction logic with streaming for memory efficiency

use color_eyre::{Result, eyre::eyre};
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use tar::Archive;

use super::PAYLOAD_MARKER;

const GZIP_MAGIC: [u8; 3] = [0x1f, 0x8b, 0x08];

/// Check if a file contains the payload marker.
/// Layout is [binary ~10MB][marker][payload.tar.gz], so marker is right after the binary.
pub fn has_payload_marker(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    Ok(find_marker_position(&mut file).is_ok())
}

/// Find the position of the payload marker in the file
fn find_marker_position(file: &mut File) -> Result<u64> {
    let marker_len = PAYLOAD_MARKER.len();
    let signature_len = marker_len + GZIP_MAGIC.len();
    let mut buffer = vec![0u8; 64 * 1024];
    let mut window = Vec::new();
    let mut current_pos = 0u64;

    file.seek(SeekFrom::Start(0))?;

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            return Err(eyre!("Payload marker not found in binary"));
        }

        window.extend_from_slice(&buffer[..bytes_read]);

        let mut search_from = 0usize;
        while let Some(rel_pos) = window[search_from..]
            .windows(signature_len)
            .position(|w| w[..marker_len] == *PAYLOAD_MARKER && w[marker_len..] == GZIP_MAGIC)
        {
            let pos = search_from + rel_pos;
            let marker_pos = current_pos + pos as u64;
            let payload_start = marker_pos + marker_len as u64;

            if payload_looks_valid(file, payload_start)? {
                return Ok(marker_pos);
            }

            search_from = pos + 1;
        }

        // Keep overlap bytes so marker/signature spanning chunks is still matched.
        if window.len() > signature_len {
            let keep_from = window.len() - (signature_len - 1);
            current_pos += keep_from as u64;
            window = window[keep_from..].to_vec();
        }
    }
}

fn payload_looks_valid(file: &File, payload_start: u64) -> Result<bool> {
    let mut probe = file.try_clone()?;
    probe.seek(SeekFrom::Start(payload_start))?;

    let decoder = GzDecoder::new(probe);
    let mut archive = Archive::new(decoder);

    match archive.entries() {
        Ok(mut entries) => match entries.next() {
            Some(Ok(_)) => Ok(true),
            _ => Ok(false),
        },
        Err(_) => Ok(false),
    }
}

/// Extract the embedded payload to a temporary directory  
pub fn extract_payload() -> Result<std::path::PathBuf> {
    let exe_path = std::env::current_exe()?;
    let mut exe_file = File::open(&exe_path)?;

    // Find marker position
    println!("  Locating payload...");
    let marker_pos = find_marker_position(&mut exe_file)?;

    // Seek to start of payload (after marker)
    let payload_start = marker_pos + PAYLOAD_MARKER.len() as u64;
    exe_file.seek(SeekFrom::Start(payload_start))?;

    // Get payload size
    let file_size = exe_file.metadata()?.len();
    let payload_size = file_size - payload_start;

    println!(
        "  Payload size: {:.2} GB",
        payload_size as f64 / 1_073_741_824.0
    );

    // Verify payload integrity with quick checksum
    println!("  Verifying payload integrity...");
    let payload_checksum = verify_payload_integrity(&mut exe_file, payload_start, payload_size)?;
    println!("  ✓ Payload checksum: {}...", &payload_checksum[..16]);

    // Reset to payload start for extraction
    exe_file.seek(SeekFrom::Start(payload_start))?;

    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.keep(); // Use keep() instead of into_path()

    // Setup progress bar
    let pb = ProgressBar::new(payload_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Extract tar.gz payload with streaming
    println!("  Extracting...");

    // Wrap file reader with progress tracking
    let reader = ProgressReader::new(exe_file, pb.clone());

    // Decompress gzip
    let decoder = GzDecoder::new(reader);

    // Extract tar archive
    let mut archive = Archive::new(decoder);
    archive.unpack(&temp_path).map_err(|e| {
        eyre!(
            "Failed to extract payload: {}\n\n\
             Troubleshooting:\n\
             - Payload may be corrupted during transfer\n\
             - Verify binary checksum: sha256sum -c nqrust-analytics-airgapped.sha256\n\
             - Check disk space: df -h /tmp\n\
             - Re-download or re-transfer the binary\n\
             Original error: {}",
            e,
            e
        )
    })?;

    pb.finish_with_message("Extraction complete");

    Ok(temp_path)
}

/// Verify payload integrity with SHA256 checksum
fn verify_payload_integrity(file: &mut File, start: u64, size: u64) -> Result<String> {
    use sha2::{Digest, Sha256};

    file.seek(SeekFrom::Start(start))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 8192];
    let mut remaining = size;

    while remaining > 0 {
        let to_read = (buffer.len() as u64).min(remaining) as usize;
        let n = file.read(&mut buffer[..to_read])?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        remaining -= n as u64;
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Wrapper to track read progress
struct ProgressReader<R> {
    inner: R,
    progress: ProgressBar,
}

impl<R> ProgressReader<R> {
    fn new(inner: R, progress: ProgressBar) -> Self {
        Self { inner, progress }
    }
}

impl<R: Read> Read for ProgressReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.progress.inc(n as u64);
        Ok(n)
    }
}
