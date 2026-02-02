// airgapped/docker.rs
// Docker operations for loading images in airgapped mode

use color_eyre::{eyre::eyre, Result};
use std::path::Path;
use std::process::{Command, Stdio};

/// List of required Docker images
const REQUIRED_IMAGES: &[(&str, &str)] = &[
    ("ghcr.io/nexusquantum/analytics-engine:latest", "analytics-engine.tar.gz"),
    ("ghcr.io/nexusquantum/analytics-engine-ibis:latest", "analytics-engine-ibis.tar.gz"),
    ("ghcr.io/nexusquantum/analytics-service:latest", "analytics-service.tar.gz"),
    ("ghcr.io/nexusquantum/analytics-ui:latest", "analytics-ui.tar.gz"),
    ("qdrant/qdrant:v1.11.0", "qdrant.tar.gz"),
    ("postgres:15", "postgres.tar.gz"),
];

/// Check if Docker is available
pub fn check_docker_available() -> Result<()> {
    let output = Command::new("docker")
        .arg("--version")
        .output();
    
    match output {
        Ok(_) => Ok(()),
        Err(_) => Err(eyre!("Docker is not installed or not in PATH")),
    }
}

/// Check if Docker daemon is running
pub fn check_docker_running() -> Result<()> {
    let output = Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    match output {
        Ok(status) if status.success() => Ok(()),
        _ => Err(eyre!("Docker daemon is not running")),
    }
}

/// Check if a specific Docker image exists locally
fn image_exists(image_name: &str) -> Result<bool> {
    let output = Command::new("docker")
        .args(&["images", "-q", image_name])
        .output()?;
    
    Ok(!output.stdout.is_empty())
}

/// Check if all required images are already loaded
pub fn check_all_images_exist() -> Result<bool> {
    // First check if Docker is available
    if check_docker_available().is_err() || check_docker_running().is_err() {
        return Ok(false);
    }
    
    // Check each required image
    for (image_name, _) in REQUIRED_IMAGES {
        if !image_exists(image_name)? {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Load a single Docker image from tar.gz file
fn load_image(tar_gz_path: &Path, image_name: &str) -> Result<()> {
    println!("    Loading {}...", image_name);
    
    // Use gunzip | docker load pipeline
    let gunzip = Command::new("gunzip")
        .arg("-c")
        .arg(tar_gz_path)
        .stdout(Stdio::piped())
        .spawn()?;
    
    let docker_load = Command::new("docker")
        .arg("load")
        .stdin(gunzip.stdout.ok_or_else(|| eyre!("Failed to pipe gunzip output"))?)
        .output()?;
    
    if !docker_load.status.success() {
        let error = String::from_utf8_lossy(&docker_load.stderr);
        return Err(eyre!("Failed to load image: {}", error));
    }
    
    Ok(())
}

/// Load all Docker images from extracted payload directory
pub fn load_all_images(payload_dir: &Path) -> Result<()> {
    // Pre-flight checks
    check_docker_available()?;
    check_docker_running()?;
    
    let total = REQUIRED_IMAGES.len();
    println!("  Loading {} Docker images...", total);
    
    for (idx, (image_name, filename)) in REQUIRED_IMAGES.iter().enumerate() {
        let tar_gz_path = payload_dir.join(filename);
        
        if !tar_gz_path.exists() {
            return Err(eyre!("Image file not found: {}", filename));
        }
        
        println!("  [{}/{}] {}", idx + 1, total, image_name);
        load_image(&tar_gz_path, image_name)?;
    }
    
    println!("  ✓ All images loaded successfully");
    
    Ok(())
}

/// Verify all images are loaded correctly
#[allow(dead_code)]
pub fn verify_images_loaded() -> Result<()> {
    println!("  Verifying images...");
    
    for (image_name, _) in REQUIRED_IMAGES {
        if !image_exists(image_name)? {
            return Err(eyre!("Image not found after loading: {}", image_name));
        }
    }
    
    println!("  ✓ All images verified");
    Ok(())
}
