//! Extraction of bundled plugins and agents at runtime.
//!
//! When installed via `cargo install`, the binary is a single file.
//! Plugins and agents are embedded via `rust-embed` and extracted to a
//! persistent data directory on first run.

use crate::server::{BundledAgents, BundledPlugins};
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Get the persistent data directory for extracted bundled assets.
/// Uses XDG-style paths: `~/.local/share/squid/bundled/`
fn get_bundled_data_dir() -> io::Result<PathBuf> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| io::Error::other("could not determine local data directory"))?;
    let dir = base.join("squid").join("bundled");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Compute SHA-256 hash of a byte slice (used for content comparison).
fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Extract all files from `BundledPlugins` to a target directory.
/// Only writes files that don't exist or have different content (SHA-256 comparison).
fn extract_plugins_dir(target_dir: &Path) -> io::Result<usize> {
    extract_embedded_dir::<BundledPlugins>(target_dir)
}

/// Extract all files from `BundledAgents` to a target directory.
fn extract_agents_dir(target_dir: &Path) -> io::Result<usize> {
    extract_embedded_dir::<BundledAgents>(target_dir)
}

/// Generic extraction helper for any rust-embed type.
fn extract_embedded_dir<E>(target_dir: &Path) -> io::Result<usize>
where
    E: rust_embed::RustEmbed,
{
    let mut count = 0;

    for filename in E::iter() {
        let dest_path = target_dir.join(filename.as_ref());

        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = E::get(filename.as_ref())
            .ok_or_else(|| io::Error::other(format!("embedded file not found: {filename}")))?;

        let embedded_hash = sha256(&content.data);

        let needs_write = if dest_path.exists() {
            match fs::read(&dest_path) {
                Ok(existing) => sha256(&existing) != embedded_hash,
                Err(e) => {
                    warn!(
                        "Cannot read existing bundled file {:?}: {e}, overwriting",
                        dest_path
                    );
                    true
                }
            }
        } else {
            true
        };

        if needs_write {
            fs::write(&dest_path, content.data.as_ref())?;
            debug!("Extracted bundled file: {:?}", dest_path);
            count += 1;
        }
    }

    Ok(count)
}

/// Extract bundled plugins to the data directory.
/// Returns the directory containing the extracted plugins.
pub fn extract_bundled_plugins() -> io::Result<(PathBuf, usize)> {
    let target_dir = get_bundled_data_dir()?.join("plugins");
    fs::create_dir_all(&target_dir)?;

    let count = extract_plugins_dir(&target_dir)?;
    if count > 0 {
        info!("Extracted {} bundled plugin(s) to {:?}", count, target_dir);
    }
    Ok((target_dir, count))
}

/// Extract bundled agents to the data directory.
/// Returns the directory containing the extracted agents.
pub fn extract_bundled_agents() -> io::Result<(PathBuf, usize)> {
    let target_dir = get_bundled_data_dir()?.join("agents");
    fs::create_dir_all(&target_dir)?;

    let count = extract_agents_dir(&target_dir)?;
    if count > 0 {
        info!("Extracted {} bundled agent(s) to {:?}", count, target_dir);
    }
    Ok((target_dir, count))
}

/// Initialize bundled assets. Called once during server startup.
/// Logs a warning instead of failing if extraction encounters issues.
pub fn init_bundled_assets() {
    match extract_bundled_plugins() {
        Ok((_plugins_dir, plugins_count)) => match extract_bundled_agents() {
            Ok((_agents_dir, agents_count)) => {
                if plugins_count == 0 && agents_count == 0 {
                    debug!("Bundled assets already up to date");
                } else {
                    info!(
                        "Bundled assets: {} plugin(s), {} agent(s)",
                        plugins_count, agents_count
                    );
                }
            }
            Err(e) => warn!("Failed to extract bundled agents: {e}"),
        },
        Err(e) => warn!("Failed to extract bundled plugins: {e}"),
    }
}

/// Remove all extracted bundled assets.
/// Useful for cleaning up after `cargo uninstall` or resetting state.
#[allow(dead_code)]
pub fn cleanup_bundled_assets() -> io::Result<()> {
    let data_dir = get_bundled_data_dir()?;
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir)?;
        info!("Cleaned up bundled assets from {:?}", data_dir);
    }
    Ok(())
}

/// Check if a path looks like it's inside a cargo target directory.
/// Used to detect development builds vs installed binaries.
fn is_in_target_dir(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "target")
}

/// Get the bundled plugins directory.
/// For `cargo install`: uses the extracted data directory.
/// For dev builds (`cargo run`): falls back to `target/` copy.
pub fn get_bundled_plugins_dir() -> Option<PathBuf> {
    if let Ok(data_dir) = get_bundled_data_dir() {
        let plugins_dir = data_dir.join("plugins");
        if plugins_dir.exists() {
            return Some(plugins_dir);
        }
    }

    // Dev build fallback
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let dev_plugins = parent.join("plugins");
            if dev_plugins.exists() && is_in_target_dir(&dev_plugins) {
                debug!("Using dev bundled plugins dir: {:?}", dev_plugins);
                return Some(dev_plugins);
            }
        }
    }

    None
}

/// Get the bundled agents directory.
/// For `cargo install`: uses the extracted data directory.
/// For dev builds (`cargo run`): falls back to `target/` copy.
pub fn get_bundled_agents_dir() -> Option<PathBuf> {
    if let Ok(data_dir) = get_bundled_data_dir() {
        let agents_dir = data_dir.join("agents");
        if agents_dir.exists() {
            return Some(agents_dir);
        }
    }

    // Dev build fallback
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let dev_agents = parent.join("agents");
            if dev_agents.exists() && is_in_target_dir(&dev_agents) {
                debug!("Using dev bundled agents dir: {:?}", dev_agents);
                return Some(dev_agents);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_target_dir() {
        assert!(is_in_target_dir(Path::new(
            "/home/user/project/target/debug/plugins"
        )));
        // Use forward slashes for cross-platform compatibility
        assert!(is_in_target_dir(Path::new(
            "C:/Users/user/target/release/agents"
        )));
        assert!(!is_in_target_dir(Path::new(
            "/home/user/.local/share/squid/bundled/plugins"
        )));
    }

    #[test]
    fn test_sha256_different_inputs() {
        let a = sha256(b"hello");
        let b = sha256(b"world");
        assert_ne!(a, b);
    }

    #[test]
    fn test_sha256_same_inputs() {
        let a = sha256(b"hello");
        let b = sha256(b"hello");
        assert_eq!(a, b);
    }
}
