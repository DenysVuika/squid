use chrono::Local;
use std::env;
use sysinfo::System;

/// Configuration for environment context
#[derive(Debug, Clone)]
pub struct EnvContextConfig {
    /// Include hostname (may reveal computer/organization names)
    pub include_hostname: bool,
    /// Include working directory (may reveal username and project paths)
    pub include_working_dir: bool,
    /// Sanitize working directory to show only relative path from home
    pub sanitize_paths: bool,
}

impl Default for EnvContextConfig {
    fn default() -> Self {
        Self {
            include_hostname: false,    // Disabled by default for privacy
            include_working_dir: false, // Disabled by default for privacy
            sanitize_paths: true,       // Always sanitize if paths are shown
        }
    }
}

/// Sanitizes a path by replacing the home directory with ~
fn sanitize_path(path: &str) -> String {
    if let Ok(home) = env::var("HOME") {
        if path.starts_with(&home) {
            return path.replacen(&home, "~", 1);
        }
    }
    path.to_string()
}

/// Gathers system and environment information and formats it as a context string
///
/// # Privacy Note
/// By default, this function excludes hostname and working directory to protect
/// privacy when sending data to external LLM APIs. These can be enabled via config.
pub fn get_env_context() -> String {
    get_env_context_with_config(&EnvContextConfig::default())
}

/// Gathers system and environment information with custom configuration
pub fn get_env_context_with_config(config: &EnvContextConfig) -> String {
    let mut sys = System::new_all();
    sys.refresh_all();

    let now = Local::now();
    let utc_now = now.naive_utc();

    let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown".to_string());

    // Get CPU architecture from the first CPU if available
    let cpu_arch = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| env::consts::ARCH.to_string());

    let mut context = format!(
        r#"Use the following environment context to answer the user's query:

<env>
OS: {}
OS Version: {}
Kernel Version: {}
CPU Architecture: {}
Platform: {}
Architecture: {}
Family: {}"#,
        os_name,
        os_version,
        kernel_version,
        cpu_arch,
        env::consts::OS,
        env::consts::ARCH,
        env::consts::FAMILY,
    );

    // Optional: Add hostname (privacy concern)
    if config.include_hostname {
        if let Some(hostname) = System::host_name() {
            context.push_str(&format!("\nHostname: {}", hostname));
        }
    }

    // Optional: Add working directory (privacy concern)
    if config.include_working_dir {
        if let Ok(working_dir) = env::current_dir() {
            let dir_str = working_dir.display().to_string();
            let dir_display = if config.sanitize_paths {
                sanitize_path(&dir_str)
            } else {
                dir_str
            };
            context.push_str(&format!("\nWorking Directory: {}", dir_display));
        }
    }

    // Add time information (safe to share)
    context.push_str(&format!(
        r#"
Local Time: {}
UTC Time: {}
Unix Timestamp: {}
Timezone: {}
</env>"#,
        now.format("%Y-%m-%d %H:%M:%S%.3f %:z"),
        utc_now.format("%Y-%m-%d %H:%M:%S%.3f +00:00"),
        now.timestamp(),
        now.offset().to_string(),
    ));

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_env_context_default() {
        let context = get_env_context();
        assert!(context.contains("<env>"));
        assert!(context.contains("</env>"));
        assert!(context.contains("OS:"));
        assert!(context.contains("Unix Timestamp:"));
        // Should NOT contain sensitive info by default
        assert!(!context.contains("Hostname:"));
        assert!(!context.contains("Working Directory:"));
    }

    #[test]
    fn test_get_env_context_with_sensitive_data() {
        let config = EnvContextConfig {
            include_hostname: true,
            include_working_dir: true,
            sanitize_paths: true,
        };
        let context = get_env_context_with_config(&config);
        assert!(context.contains("<env>"));
        assert!(context.contains("OS:"));
        assert!(context.contains("Hostname:"));
        assert!(context.contains("Working Directory:"));
    }

    #[test]
    fn test_sanitize_path() {
        // This test will vary based on the actual HOME directory
        if let Ok(home) = env::var("HOME") {
            let test_path = format!("{}/projects/myapp", home);
            let sanitized = sanitize_path(&test_path);
            assert_eq!(sanitized, "~/projects/myapp");
        }
    }

    #[test]
    fn test_sanitize_path_no_home() {
        let path = "/tmp/some/path";
        let sanitized = sanitize_path(path);
        assert_eq!(sanitized, path);
    }
}
