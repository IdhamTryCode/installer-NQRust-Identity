// airgapped/mod.rs
// Main module for airgapped installer functionality

pub mod extractor;
pub mod docker;

use color_eyre::Result;

/// Marker string that separates binary code from embedded payload
pub const PAYLOAD_MARKER: &[u8] = b"__NQRUST_PAYLOAD__";

/// Check if the current binary has an embedded payload
pub fn is_airgapped_binary() -> Result<bool> {
    let exe_path = std::env::current_exe()?;
    let file_size = std::fs::metadata(&exe_path)?.len();
    
    // If binary is larger than expected (~50 MB threshold), likely has payload
    // Normal binary is ~10 MB, with payload it's 2.5+ GB
    if file_size > 50_000_000 {
        // Verify by checking for marker
        extractor::has_payload_marker(&exe_path)
    } else {
        Ok(false)
    }
}

/// Check if Docker images are already loaded locally
pub fn images_already_loaded() -> Result<bool> {
    docker::check_all_images_exist()
}

/// Main setup function for airgapped installation
/// Extracts payload and loads Docker images
pub async fn setup() -> Result<()> {
    println!("\n🔒 Airgapped mode detected");
    
    // Check if images already loaded
    if images_already_loaded()? {
        println!("✓ Docker images already loaded, skipping extraction");
        return Ok(());
    }
    
    println!("📦 Extracting embedded Docker images...");
    
    // Extract payload to temporary directory
    let temp_dir = extractor::extract_payload()?;
    
    println!("🐳 Loading images to Docker...");
    
    // Load all images to Docker
    docker::load_all_images(&temp_dir)?;
    
    println!("🧹 Cleaning up temporary files...");
    
    // Cleanup temp directory
    std::fs::remove_dir_all(&temp_dir)?;
    
    println!("✓ Airgapped setup complete!\n");
    
    Ok(())
}
