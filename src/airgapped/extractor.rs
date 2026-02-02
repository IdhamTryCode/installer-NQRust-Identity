// airgapped/extractor.rs
// Payload extraction logic with streaming for memory efficiency

use color_eyre::{eyre::eyre, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use flate2::read::GzDecoder;
use tar::Archive;
use indicatif::{ProgressBar, ProgressStyle};

use super::PAYLOAD_MARKER;

/// Check if a file contains the payload marker
pub fn has_payload_marker(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();
    
    // Read last 10 MB to search for marker (marker should be near the middle)
    let search_size = 10_000_000u64.min(file_size);
    let search_start = if file_size > search_size {
        file_size - search_size
    } else {
        0
    };
    
    file.seek(SeekFrom::Start(search_start))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Search for marker in buffer
    Ok(buffer.windows(PAYLOAD_MARKER.len())
        .any(|window| window == PAYLOAD_MARKER))
}

/// Find the position of the payload marker in the file
fn find_marker_position(file: &mut File) -> Result<u64> {
    let file_size = file.metadata()?.len();
    
    // Use a sliding window approach to find marker
    // Start from a reasonable position (binary is ~10 MB)
    let start_pos = 5_000_000u64.min(file_size / 2);
    file.seek(SeekFrom::Start(start_pos))?;
    
    let marker_len = PAYLOAD_MARKER.len();
    let mut buffer = vec![0u8; 8192]; // 8 KB buffer
    let mut window = Vec::new();
    let mut current_pos = start_pos;
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            return Err(eyre!("Payload marker not found in binary"));
        }
        
        window.extend_from_slice(&buffer[..bytes_read]);
        
        // Search for marker in window
        if let Some(pos) = window.windows(marker_len)
            .position(|w| w == PAYLOAD_MARKER) {
            return Ok(current_pos + pos as u64);
        }
        
        // Keep last marker_len bytes for next iteration
        if window.len() > marker_len {
            let keep_from = window.len() - marker_len;
            current_pos += keep_from as u64;
            window = window[keep_from..].to_vec();
        }
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
    
    println!("  Payload size: {:.2} GB", payload_size as f64 / 1_073_741_824.0);
    
    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.keep(); // Use keep() instead of into_path()
    
    // Setup progress bar
    let pb = ProgressBar::new(payload_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    // Extract tar.gz payload with streaming
    println!("  Extracting...");
    
    // Wrap file reader with progress tracking
    let reader = ProgressReader::new(exe_file, pb.clone());
    
    // Decompress gzip
    let decoder = GzDecoder::new(reader);
    
    // Extract tar archive
    let mut archive = Archive::new(decoder);
    archive.unpack(&temp_path)?;
    
    pb.finish_with_message("Extraction complete");
    
    Ok(temp_path)
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
