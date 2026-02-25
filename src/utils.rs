use std::fs;
use std::path::{Path, PathBuf};

use color_eyre::eyre::Result;

pub const COMPOSE_TEMPLATE: &str = include_str!("../docker-compose.yaml");

#[allow(dead_code)]
pub fn find_file(filename: &str) -> bool {
    let root = project_root();
    root.join(filename).exists()
}

pub fn project_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Walk up to find a directory that contains the compose file or Cargo.toml
    let candidates = [
        "docker-compose.yaml",
        "docker-compose.yml",
        "compose.yml",
        "compose.yaml",
        "Cargo.toml",
    ];

    let mut current = start.as_path();
    loop {
        if candidates.iter().any(|name| current.join(name).exists()) {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Fallback: if running from target/*/ build dirs, hop two parents
    if start
        .to_str()
        .map(|s| s.contains("target"))
        .unwrap_or(false)
    {
        if let Some(parent) = start.parent().and_then(|p| p.parent()) {
            return parent.to_path_buf();
        }
    }

    start
}

/// Ensure docker-compose.yaml exists in the working dir.
/// If it doesn't, write the embedded template.
pub fn ensure_compose_bundle(root: &Path) -> Result<()> {
    let compose_path = root.join("docker-compose.yaml");
    if !compose_path.exists() {
        if let Some(parent) = compose_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&compose_path, COMPOSE_TEMPLATE)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file_exists() {
        assert!(
            find_file("Cargo.toml"),
            "Should find Cargo.toml in project root"
        );
    }

    #[test]
    fn test_find_file_not_exists() {
        assert!(
            !find_file("non_existent_file_xyz"),
            "Should not find non-existent file"
        );
    }
}
