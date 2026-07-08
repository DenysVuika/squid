use crate::manager::{PluginManager, PluginSystemConfig};
use crate::registry::PluginRegistry;
use crate::runtime::PluginRuntime;
use crate::validator::SchemaValidator;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    dir.push(format!(
        "squid_plugins_{prefix}_{}_{}",
        std::process::id(),
        ts
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

fn write_plugin(dir: &Path, plugin_id: &str, description: &str, result_expr: &str) {
    let plugin_dir = dir.join(format!("{plugin_id}_dir"));
    fs::create_dir_all(&plugin_dir).expect("failed to create plugin dir");

    let metadata = json!({
        "id": plugin_id,
        "title": format!("{plugin_id} title"),
        "description": description,
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
        serde_json::to_string_pretty(&metadata).expect("serialize metadata"),
    )
    .expect("write plugin metadata");

    let js_code = format!(
        "function execute(context, input) {{ return {{ result: {} }}; }}\nglobalThis.execute = execute;",
        result_expr
    );
    fs::write(plugin_dir.join("index.js"), js_code).expect("write plugin code");
}

#[test]
fn runtime_executes_plugin_and_blocks_eval() {
    let runtime = PluginRuntime::with_memory_limit(64).expect("create runtime");

    let js_code = r#"
        function execute(context, input) {
            return { result: "Hello, " + input.name };
        }
        globalThis.execute = execute;
    "#;
    runtime.load_plugin(js_code).expect("load plugin code");

    let result = runtime
        .execute_sync(json!({ "name": "World" }))
        .expect("execute plugin");
    assert_eq!(result["result"], "Hello, World");

    let runtime = PluginRuntime::with_memory_limit(64).expect("create runtime");
    let js_code = r#"
        function execute(context, input) {
            try {
                eval("1+1");
                return { result: "not blocked" };
            } catch (e) {
                return { result: "blocked" };
            }
        }
        globalThis.execute = execute;
    "#;
    runtime.load_plugin(js_code).expect("load plugin code");

    let result = runtime.execute_sync(json!({})).expect("execute plugin");
    assert_eq!(result["result"], "blocked");
}

#[test]
fn schema_validator_accepts_valid_and_rejects_invalid_data() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "count": { "type": "number" }
        },
        "required": ["name"]
    });

    let validator = SchemaValidator::new(&schema).expect("compile schema");

    assert!(
        validator
            .validate(&json!({ "name": "ok", "count": 1 }))
            .is_ok()
    );

    let errors = validator
        .validate(&json!({ "count": 1 }))
        .expect_err("missing required property should fail");
    assert!(!errors.is_empty());
}

#[test]
fn registry_prefers_workspace_over_global_for_same_plugin_id() {
    let root = create_temp_dir("registry_precedence");
    let global_dir = root.join("global_plugins");
    let workspace_dir = root.join("workspace_plugins");

    fs::create_dir_all(&global_dir).expect("create global dir");
    fs::create_dir_all(&workspace_dir).expect("create workspace dir");

    write_plugin(&global_dir, "shared", "global version", "'from-global'");
    write_plugin(
        &workspace_dir,
        "shared",
        "workspace version",
        "'from-workspace'",
    );

    let mut registry = PluginRegistry::new();
    registry.add_global_directory(global_dir.clone());
    registry.add_workspace_directory(workspace_dir.clone());

    let loaded = registry.discover_plugins().expect("discover plugins");
    assert_eq!(loaded, 2);

    let metadata = registry.get_plugin("shared").expect("plugin should exist");
    assert_eq!(metadata.description, "workspace version");
    assert!(!metadata.is_global);

    fs::remove_dir_all(&root).expect("cleanup temp tree");
}

#[test]
fn manager_initializes_and_executes_plugin_from_bundled_dir() {
    let root = create_temp_dir("manager_exec");
    let bundled_dir = root.join("bundled_plugins");
    fs::create_dir_all(&bundled_dir).expect("create bundled dir");

    write_plugin(&bundled_dir, "echo", "echo plugin", "input.message");

    let config = PluginSystemConfig {
        enabled: true,
        load_global: false,
        load_workspace: false,
        load_bundled: true,
        working_dir: root.clone(),
        default_timeout_seconds: 5,
        max_memory_mb: 64,
        bundled_plugins_dir: Some(bundled_dir),
        ignore_file_name: ".squidignore".to_string(),
    };

    let manager = PluginManager::new(Arc::new(config));
    manager.initialize().expect("initialize manager");

    assert_eq!(manager.plugin_count(), 1);
    assert!(manager.is_plugin_tool("plugin:echo"));

    let tools = manager.get_plugin_tools().expect("get tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "plugin:echo");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    let output = rt
        .block_on(manager.execute_plugin_tool("plugin:echo", &json!({ "message": "hello" })))
        .expect("execute plugin");

    assert_eq!(output["result"], "hello");

    fs::remove_dir_all(&root).expect("cleanup temp tree");
}
