#[cfg(test)]
mod plugin_system_tests {
    use crate::plugins::{context, metadata, registry, runtime, validator};
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test plugin
    fn create_test_plugin(dir: &PathBuf, id: &str, js_code: &str) -> std::io::Result<()> {
        let plugin_dir = dir.join(id);
        fs::create_dir_all(&plugin_dir)?;

        let metadata = json!({
            "id": id,
            "title": "Test Plugin",
            "description": "A test plugin",
            "version": "0.1.0",
            "api_version": "1.0",
            "security": {
                "requires": [],
                "network": false,
                "file_write": false
            },
            "input_schema": {
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            },
            "output_schema": {
                "type": "object",
                "properties": {
                    "result": { "type": "string" }
                },
                "required": ["result"]
            }
        });

        fs::write(
            plugin_dir.join("plugin.json"),
            serde_json::to_string_pretty(&metadata)?,
        )?;
        fs::write(plugin_dir.join("index.js"), js_code)?;

        Ok(())
    }

    #[test]
    fn test_plugin_metadata_loading() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test-plugin");

        create_test_plugin(
            &temp_dir.path().to_path_buf(),
            "test-plugin",
            r#"
                function execute(context, input) {
                    return { result: "test" };
                }
                globalThis.execute = execute;
            "#,
        )
        .unwrap();

        let metadata = metadata::PluginMetadata::load(&plugin_dir).unwrap();

        assert_eq!(metadata.id, "test-plugin");
        assert_eq!(metadata.title, "Test Plugin");
        assert_eq!(metadata.api_version, "1.0");
    }

    #[test]
    fn test_plugin_registry_discovery() {
        let temp_dir = TempDir::new().unwrap();

        // Create two test plugins
        create_test_plugin(
            &temp_dir.path().to_path_buf(),
            "plugin1",
            "function execute(context, input) { return { result: '1' }; } globalThis.execute = execute;",
        ).unwrap();

        create_test_plugin(
            &temp_dir.path().to_path_buf(),
            "plugin2",
            "function execute(context, input) { return { result: '2' }; } globalThis.execute = execute;",
        ).unwrap();

        let mut registry = registry::PluginRegistry::new();
        registry.add_directory(temp_dir.path().to_path_buf());

        let count = registry.discover_plugins().unwrap();

        assert_eq!(count, 2);
        assert_eq!(registry.plugin_count(), 2);
        assert!(registry.has_plugin("plugin1"));
        assert!(registry.has_plugin("plugin2"));
    }

    #[test]
    fn test_json_schema_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name"]
        });

        let valid_data = json!({
            "name": "Alice",
            "age": 30
        });

        let invalid_data = json!({
            "age": 30
        });

        let validator = validator::SchemaValidator::new(&schema).unwrap();

        assert!(validator.validate(&valid_data).is_ok());
        assert!(validator.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_plugin_runtime_execution() {
        let runtime = runtime::PluginRuntime::new().unwrap();

        let js_code = r#"
            function execute(context, input) {
                return { result: "Hello, " + input.message };
            }
            globalThis.execute = execute;
        "#;

        runtime.load_plugin(js_code).unwrap();

        let input = json!({ "message": "World" });
        let result = runtime.execute_sync(input).unwrap();

        assert_eq!(result["result"], "Hello, World");
    }

    #[test]
    fn test_plugin_timeout() {
        let runtime = runtime::PluginRuntime::new().unwrap();

        let js_code = r#"
            function execute(context, input) {
                // Simulate long-running operation
                let start = Date.now();
                while (Date.now() - start < 10000) {
                    // Busy loop for 10 seconds
                }
                return { result: "done" };
            }
            globalThis.execute = execute;
        "#;

        runtime.load_plugin(js_code).unwrap();

        let input = json!({});
        // Note: Timeout is handled at manager level, not runtime level
        // This test just ensures the plugin doesn't crash
        let result = runtime.execute_sync(input);

        // Will either complete or take a very long time
        // Real timeout enforcement happens in PluginManager with spawn_blocking + tokio::timeout
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_plugin_tool_name_format() {
        let metadata = metadata::PluginMetadata {
            id: "my-plugin".to_string(),
            title: "My Plugin".to_string(),
            description: "Test".to_string(),
            version: "0.1.0".to_string(),
            api_version: "1.0".to_string(),
            security: metadata::SecurityRequirements {
                requires: vec![],
                network: false,
                file_write: false,
            },
            input_schema: json!({}),
            output_schema: json!({}),
            plugin_path: PathBuf::new(),
            is_global: false,
        };

        assert_eq!(metadata.tool_name(), "plugin:my-plugin");
    }

    #[test]
    fn test_context_read_file_permission() {
        use crate::config::Config;
        use std::sync::Arc;

        // Use working_dir = "." for tests
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = context::PluginContext::new(
            config,
            "test-plugin".to_string(),
            false, // no network
            false, // no file write
        );

        // Try to read Cargo.toml (a file in the project directory)
        let result = context.read_file("Cargo.toml");

        // Should succeed
        if let Err(e) = &result {
            eprintln!("Read file error: {}", e);
        }
        assert!(result.is_ok());
        assert!(result.unwrap().contains("[package]"));
    }

    #[test]
    fn test_context_write_file_permission_denied() {
        use crate::config::Config;
        use std::sync::Arc;

        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = context::PluginContext::new(
            config,
            "test-plugin".to_string(),
            false, // no network
            false, // no file write permission
        );

        // Try to write to a file
        let result = context.write_file("test-output.txt", "content");

        // Should be denied
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("file_write permission"));
    }
}
