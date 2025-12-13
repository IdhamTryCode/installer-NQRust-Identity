use std::path::PathBuf;

pub const ENV_TEMPLATE: &str = include_str!("../env_template");

pub fn find_file(filename: &str) -> bool {
    let root = project_root();
    root.join(filename).exists()
}

pub fn project_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Walk up to find a directory that contains either docker-compose files or Cargo.toml
    let candidates = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
        "Cargo.toml",
    ];

    let mut current = start.as_path();
    while let Some(dir) = current.parent().or_else(|| Some(current)) {
        if candidates.iter().any(|name| dir.join(name).exists()) {
            return dir.to_path_buf();
        }

        // If we've reached filesystem root, stop
        if dir.parent().is_none() {
            break;
        }

        current = dir.parent().unwrap_or(dir);
    }

    // Fallback: if running from target/*/ build dirs, hop two parents as before
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_file_exists() {
        // Cargo.toml is known to exist in the project root
        assert!(
            find_file("Cargo.toml"),
            "Should find Cargo.toml in project root"
        );
    }

    #[test]
    fn test_find_file_not_exists() {
        // This file should not exist
        assert!(
            !find_file("non_existent_file_xyz"),
            "Should not find non-existent file"
        );
    }
}
