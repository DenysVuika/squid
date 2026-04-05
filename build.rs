use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

fn main() {
    let static_dir = Path::new("static");
    let web_dir = Path::new("web");
    let plugins_dir = Path::new("plugins");
    let agents_dir = Path::new("agents");

    // Tell Cargo to re-run this script when web sources or bundled assets change.
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/public");
    println!("cargo:rerun-if-changed=web/index.html");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/package-lock.json");
    println!("cargo:rerun-if-changed=web/tsconfig.json");
    println!("cargo:rerun-if-changed=web/tsconfig.app.json");
    println!("cargo:rerun-if-changed=web/tsconfig.node.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=static");
    println!("cargo:rerun-if-changed=plugins");
    println!("cargo:rerun-if-changed=agents");

    // Attempt to build the web frontend if npm is available and static/ is
    // missing or stale. The build is best-effort: when Node.js is not
    // installed (e.g. CI containers, cross-compilation, minimal dev setups)
    // we fall back to the existing stub/empty static directory so the Rust
    // build still succeeds.
    let needs_build = web_build_required(web_dir, static_dir);

    if needs_build && web_dir.join("package.json").exists() {
        if let Ok(npm) = which_npm() {
            eprintln!(
                "cargo:warning================================================================="
            );
            eprintln!("cargo:warning=Building web frontend (static/ is missing or stale)...");
            eprintln!(
                "cargo:warning================================================================="
            );

            // npm ci / npm install
            let install_status = Command::new(&npm)
                .args(["ci", "--ignore-scripts"])
                .current_dir(web_dir)
                .status();

            match install_status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    // Fall back to `npm install` if `npm ci` fails (no lockfile, etc.)
                    eprintln!("cargo:warning=npm ci exited with {s}, trying npm install...");
                    let fallback = Command::new(&npm)
                        .args(["install"])
                        .current_dir(web_dir)
                        .status();
                    if !matches!(fallback, Ok(s) if s.success()) {
                        eprintln!(
                            "cargo:warning================================================================="
                        );
                        eprintln!("cargo:warning=ERROR: npm install failed — skipping web build");
                        eprintln!("cargo:warning=Web UI will NOT be available!");
                        eprintln!(
                            "cargo:warning================================================================="
                        );
                        ensure_static_dir(static_dir);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                    eprintln!("cargo:warning=ERROR: Could not run npm: {e}");
                    eprintln!("cargo:warning=Web UI will NOT be available!");
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                    ensure_static_dir(static_dir);
                    return;
                }
            }

            // npm run build
            let build_status = Command::new(&npm)
                .args(["run", "build"])
                .current_dir(web_dir)
                .status();

            match build_status {
                Ok(s) if s.success() => {
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                    eprintln!("cargo:warning=Web frontend built successfully.");
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                }
                Ok(s) => {
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                    eprintln!("cargo:warning=WARNING: npm run build exited with {s}");
                    eprintln!("cargo:warning=Web UI may be unavailable!");
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                }
                Err(e) => {
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                    eprintln!("cargo:warning=ERROR: Could not run npm build: {e}");
                    eprintln!("cargo:warning=Web UI may be unavailable!");
                    eprintln!(
                        "cargo:warning================================================================="
                    );
                }
            }
        } else {
            eprintln!(
                "cargo:warning================================================================="
            );
            eprintln!("cargo:warning=ERROR: npm not found on system PATH!");
            eprintln!("cargo:warning=");
            eprintln!("cargo:warning=The web UI will NOT be available.");
            eprintln!("cargo:warning=");
            eprintln!("cargo:warning=To enable the web UI, install Node.js and npm:");
            eprintln!("cargo:warning=  - macOS/Linux: https://nodejs.org/");
            eprintln!("cargo:warning=  - Or use your package manager (brew, apt, etc.)");
            eprintln!("cargo:warning=");
            eprintln!("cargo:warning=Then rebuild with: cargo build --release");
            eprintln!(
                "cargo:warning================================================================="
            );
        }
    }

    ensure_static_dir(static_dir);
    copy_bundled_plugins(plugins_dir);
    copy_bundled_agents(agents_dir);
}

fn web_build_required(web_dir: &Path, static_dir: &Path) -> bool {
    // Check if the static directory has actual build artifacts
    let index_html = static_dir.join("index.html");
    if !index_html.exists() {
        return true;
    }

    let Some(static_mtime) = latest_modified(static_dir) else {
        return true;
    };

    [
        web_dir.join("src"),
        web_dir.join("public"),
        web_dir.join("index.html"),
        web_dir.join("package.json"),
        web_dir.join("package-lock.json"),
        web_dir.join("tsconfig.json"),
        web_dir.join("tsconfig.app.json"),
        web_dir.join("tsconfig.node.json"),
        web_dir.join("vite.config.ts"),
    ]
    .into_iter()
    .filter_map(|path| latest_modified(&path))
    .any(|mtime| mtime > static_mtime)
}

fn latest_modified(path: &Path) -> Option<SystemTime> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.is_file() {
        return metadata.modified().ok();
    }
    if !metadata.is_dir() {
        return None;
    }

    let mut latest = metadata.modified().ok();
    let entries = fs::read_dir(path).ok()?;
    for entry in entries.flatten() {
        if let Some(child_mtime) = latest_modified(&entry.path()) {
            latest = Some(match latest {
                Some(current) if current >= child_mtime => current,
                _ => child_mtime,
            });
        }
    }
    latest
}

/// Ensure the static directory exists so `rust-embed` does not fail at compile
/// time even when the web frontend is not built.
fn ensure_static_dir(static_dir: &Path) {
    if !static_dir.exists() {
        std::fs::create_dir_all(static_dir).expect("failed to create static/");

        // Create a .gitkeep file to preserve the directory in git
        let gitkeep_path = static_dir.join(".gitkeep");
        if !gitkeep_path.exists() {
            std::fs::write(gitkeep_path, "").ok();
        }
    }
}

/// Locate the `npm` binary on the system PATH.
fn which_npm() -> Result<String, ()> {
    let cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };

    Command::new(cmd)
        .arg("npm")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.lines().next().unwrap_or("npm").trim().to_string())
            } else {
                None
            }
        })
        .ok_or(())
}

/// Copy bundled plugins directory to the build output directory.
/// This ensures plugins are available next to the executable during development.
fn copy_bundled_plugins(plugins_dir: &Path) {
    // OUT_DIR points to target/debug/build/<crate>-<hash>/out
    // We want target/debug/plugins (or target/release/plugins)
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    // Navigate from OUT_DIR to the profile directory (debug or release)
    // OUT_DIR: /path/to/target/debug/build/squid-rs-<hash>/out
    // We want: /path/to/target/debug/plugins
    let profile_dir = Path::new(&out_dir)
        .parent() // /path/to/target/debug/build/squid-rs-<hash>
        .and_then(|p| p.parent()) // /path/to/target/debug/build
        .and_then(|p| p.parent()) // /path/to/target/debug
        .expect("Failed to resolve profile directory");

    let dest_plugins = profile_dir.join("plugins");

    if !plugins_dir.exists() {
        eprintln!(
            "cargo:warning=Plugins directory does not exist: {}",
            plugins_dir.display()
        );
        return;
    }

    // Copy plugins if source is newer than destination
    let needs_copy = !dest_plugins.exists()
        || latest_modified(plugins_dir)
            .map(|src_mtime| {
                latest_modified(&dest_plugins)
                    .map(|dst_mtime| src_mtime > dst_mtime)
                    .unwrap_or(true)
            })
            .unwrap_or(true);

    if needs_copy {
        if dest_plugins.exists() {
            let _ = fs::remove_dir_all(&dest_plugins);
        }

        if let Err(e) = copy_dir_all(plugins_dir, &dest_plugins) {
            eprintln!("cargo:warning=Failed to copy plugins directory: {}", e);
            return;
        }

        eprintln!(
            "cargo:warning=Copied bundled plugins to {}",
            dest_plugins.display()
        );
    }
}

/// Copy bundled agents directory to the build output directory.
/// This ensures default agents are available next to the executable during development.
fn copy_bundled_agents(agents_dir: &Path) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");

    let profile_dir = Path::new(&out_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .expect("Failed to resolve profile directory");

    let dest_agents = profile_dir.join("agents");

    if !agents_dir.exists() {
        // Agents directory is optional — users may provide their own
        return;
    }

    let needs_copy = !dest_agents.exists()
        || latest_modified(agents_dir)
            .map(|src_mtime| {
                latest_modified(&dest_agents)
                    .map(|dst_mtime| src_mtime > dst_mtime)
                    .unwrap_or(true)
            })
            .unwrap_or(true);

    if needs_copy {
        if dest_agents.exists() {
            let _ = fs::remove_dir_all(&dest_agents);
        }

        if let Err(e) = copy_dir_all(agents_dir, &dest_agents) {
            eprintln!("cargo:warning=Failed to copy agents directory: {}", e);
            return;
        }

        eprintln!(
            "cargo:warning=Copied bundled agents to {}",
            dest_agents.display()
        );
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
