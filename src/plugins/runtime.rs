use log::debug;
use rquickjs::{Context, Function, Runtime};
use serde_json::Value;
use std::sync::Arc;

use crate::plugins::context::PluginContext;

/// QuickJS runtime wrapper for executing plugin code
pub struct PluginRuntime {
    #[allow(dead_code)] // Kept alive for context to reference
    runtime: Runtime,
    context: Context,
    plugin_context: Option<Arc<PluginContext>>,
}

impl PluginRuntime {
    /// Create a new sandboxed runtime
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_memory_limit(128)
    }
    
    /// Create a new sandboxed runtime with custom memory limit
    pub fn with_memory_limit(memory_mb: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        
        // Set memory limit (in bytes)
        runtime.set_memory_limit(memory_mb * 1024 * 1024);
        
        // Set max stack size (1MB)
        runtime.set_max_stack_size(1024 * 1024);
        
        let context = Context::full(&runtime)?;
        
        Ok(Self { 
            runtime, 
            context,
            plugin_context: None,
        })
    }
    
    /// Set the plugin context for this runtime
    pub fn set_context(&mut self, ctx: Arc<PluginContext>) {
        self.plugin_context = Some(ctx);
    }
    
    /// Load and execute plugin JavaScript code
    pub fn load_plugin(&self, js_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.context.with(|ctx| {
            // Apply sandbox restrictions
            Self::apply_sandbox(&ctx)?;
            
            // Execute the plugin code
            let _: () = ctx.eval(js_code.as_bytes())
                .map_err(|e| format!("Failed to execute plugin code: {}", e))?;
            
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;
        
        Ok(())
    }
    
    /// Execute the plugin's main function with input (synchronous)
    pub fn execute_sync(
        &self,
        input: Value,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let input_json = serde_json::to_string(&input)?;
        let plugin_ctx = self.plugin_context.clone();
        
        let result_str = self.context.with(|ctx| {
            // Inject context APIs if we have a context
            if let Some(ref pctx) = plugin_ctx {
                Self::inject_context_apis(&ctx, Arc::clone(pctx))?;
            }
            
            // Get globals
            let globals = ctx.globals();
            
            // Parse input JSON by setting it as a global variable (avoids escaping issues)
            globals.set("__input_json_str", input_json.as_str())?;
            let json_obj: rquickjs::Object = ctx.eval(b"JSON.parse(__input_json_str)")
                .map_err(|e| format!("Failed to parse input JSON: {}", e))?;
            
            // Get the global execute function
            let execute_fn: Function = globals
                .get("execute")
                .map_err(|_| "Plugin must define a global 'execute' function")?;
            
            // Get context object
            let context_obj: rquickjs::Value = globals.get("__pluginContext")
                .unwrap_or_else(|_| ctx.eval("({})".as_bytes()).unwrap());
            
            // Call the execute function (synchronous now)
            let result_js: rquickjs::Value = execute_fn
                .call((context_obj, json_obj))
                .map_err(|e| format!("Plugin execution failed: {}", e))?;
            
            // Convert result to JSON string
            let json_stringify: Function = ctx.eval("JSON.stringify".as_bytes())
                .map_err(|e| format!("Failed to get JSON.stringify: {}", e))?;
            
            let result_str: String = json_stringify
                .call((result_js,))
                .map_err(|e| format!("Failed to stringify result: {}", e))?;
            
            Ok::<_, Box<dyn std::error::Error>>(result_str)
        })?;
        
        // Extract and log any console.log messages
        if let Some(ref pctx) = self.plugin_context {
            self.context.with(|ctx| {
                let globals = ctx.globals();
                if let Ok(logs_array) = globals.get::<_, rquickjs::Array>("__pluginLogs") {
                    for i in 0..logs_array.len() {
                        if let Ok(log_msg) = logs_array.get::<String>(i) {
                            pctx.log(&log_msg);
                        }
                    }
                }
                Ok::<_, Box<dyn std::error::Error>>(())
            })?;
        }
        
        // Parse result back to serde_json::Value
        let result_value: Value = serde_json::from_str(&result_str)?;
        
        Ok(result_value)
    }
    
    /// Apply sandbox restrictions to prevent unsafe operations
    fn apply_sandbox(ctx: &rquickjs::Ctx<'_>) -> Result<(), Box<dyn std::error::Error>> {
        // Set up console.log to collect messages in an array
        let _: () = ctx.eval(
            r#"
            globalThis.__pluginLogs = [];
            globalThis.console = {
                log: function(...args) {
                    const message = args.map(arg => {
                        if (typeof arg === 'object') {
                            try { return JSON.stringify(arg); }
                            catch (e) { return String(arg); }
                        }
                        return String(arg);
                    }).join(' ');
                    globalThis.__pluginLogs.push(message);
                }
            };
            "#.as_bytes(),
        )
        .map_err(|e| format!("Failed to inject console: {}", e))?;
        
        // Disable eval and Function constructor
        let _: () = ctx.eval(
            r#"
            Object.defineProperty(globalThis, 'eval', { 
                value: undefined,
                writable: false,
                configurable: false
            });
            Object.defineProperty(globalThis, 'Function', { 
                value: undefined,
                writable: false,
                configurable: false
            });
            "#.as_bytes(),
        )
        .map_err(|e| format!("Failed to apply sandbox restrictions: {}", e))?;
        
        debug!("Applied sandbox restrictions to QuickJS context");
        
        Ok(())
    }
    
    /// Inject context APIs using JavaScript wrapper that calls Rust implementations
    fn inject_context_apis(
        ctx: &rquickjs::Ctx<'_>,
        plugin_context: Arc<PluginContext>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let globals = ctx.globals();
        
        // Create Rust-backed functions that panic on error (converted to JS exceptions)
        let pctx_read = Arc::clone(&plugin_context);
        let read_fn = rquickjs::Function::new(ctx.clone(), move |path: String| -> String {
            match pctx_read.read_file(&path) {
                Ok(content) => content,
                Err(e) => panic!("{}", e)
            }
        })?;
        globals.set("__rustReadFile", read_fn)?;
        
        let pctx_write = Arc::clone(&plugin_context);
        let write_fn = rquickjs::Function::new(ctx.clone(), move |path: String, content: String| -> bool {
            match pctx_write.write_file(&path, &content) {
                Ok(_) => true,
                Err(e) => panic!("{}", e)
            }
        })?;
        globals.set("__rustWriteFile", write_fn)?;
        
        let pctx_http = Arc::clone(&plugin_context);
        let http_fn = rquickjs::Function::new(ctx.clone(), move |url: String, timeout: Option<u64>| -> String {
            // http_get is async - block on it since we're in spawn_blocking
            let result = tokio::runtime::Handle::current().block_on(
                pctx_http.http_get(&url, timeout)
            );
            match result {
                Ok(content) => content,
                Err(e) => panic!("{}", e)
            }
        })?;
        globals.set("__rustHttpGet", http_fn)?;
        
        // Create JavaScript wrapper object with error handling
        let context_code = format!(r#"
            globalThis.__pluginContext = {{
                log: function(msg) {{
                    console.log('[Plugin] ' + msg);
                }},
                config: {{
                    projectDir: '{}'
                }},
                readFile: function(path) {{
                    try {{
                        return globalThis.__rustReadFile(path);
                    }} catch (e) {{
                        throw new Error('readFile failed: ' + e.message);
                    }}
                }},
                writeFile: function(path, content) {{
                    try {{
                        return globalThis.__rustWriteFile(path, content);
                    }} catch (e) {{
                        throw new Error('writeFile failed: ' + e.message);
                    }}
                }},
                httpGet: function(url, timeout) {{
                    try {{
                        return globalThis.__rustHttpGet(url, timeout || 5000);
                    }} catch (e) {{
                        throw new Error('httpGet failed: ' + e.message);
                    }}
                }}
            }};
        "#, plugin_context.project_dir().replace('\\', "\\\\").replace('\'', "\\'"));
        
        let _: () = ctx.eval(context_code.as_bytes())
            .map_err(|e| format!("Failed to inject context: {}", e))?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_simple_plugin_execution() {
        let runtime = PluginRuntime::new().unwrap();
        
        let js_code = r#"
            function execute(context, input) {
                return { result: "Hello, " + input.name };
            }
            globalThis.execute = execute;
        "#;
        
        runtime.load_plugin(js_code).unwrap();
        
        let input = json!({ "name": "World" });
        let result = runtime.execute_sync(input).unwrap();
        
        assert_eq!(result["result"], "Hello, World");
    }
    
    #[test]
    fn test_sandbox_blocks_eval() {
        let runtime = PluginRuntime::new().unwrap();
        
        let js_code = r#"
            function execute(context, input) {
                try {
                    eval("console.log('should not work')");
                    return { error: "eval should be blocked" };
                } catch (e) {
                    return { success: true, message: "eval is blocked" };
                }
            }
            globalThis.execute = execute;
        "#;
        
        runtime.load_plugin(js_code).unwrap();
        
        let input = json!({});
        let result = runtime.execute_sync(input).unwrap();
        
        assert_eq!(result["success"], true);
    }
    
    #[test]
    fn test_plugin_logging() {
        use crate::config::Config;
        use std::sync::Arc;
        
        let mut runtime = PluginRuntime::new().unwrap();
        
        // Set up plugin context
        let config = Arc::new(Config::load());
        let context = Arc::new(PluginContext::new(
            config,
            "test-logger".to_string(),
            false,
            false,
        ));
        runtime.set_context(context);
        
        let js_code = r#"
            function execute(context, input) {
                console.log("Test log message");
                console.log("Multiple", "arguments", 123);
                console.log({ key: "value" });
                context.log("Using context.log");
                return { success: true };
            }
            globalThis.execute = execute;
        "#;
        
        runtime.load_plugin(js_code).unwrap();
        
        let input = json!({});
        let result = runtime.execute_sync(input).unwrap();
        
        assert_eq!(result["success"], true);
        // Note: Actual log output goes to the system logger and database
        // This test verifies it doesn't crash
    }
    
    #[test]
    fn test_plugin_context_readfile() {
        use crate::config::Config;
        use std::sync::Arc;

        let mut runtime = PluginRuntime::new().unwrap();

        // Set up plugin context with working_dir = "." for tests
        let mut config = Config::load();
        config.working_dir = ".".to_string();
        let config = Arc::new(config);

        let context = Arc::new(PluginContext::new(
            config,
            "test-readfile".to_string(),
            false,
            false,
        ));
        runtime.set_context(context);
        
        let js_code = r#"
            function execute(context, input) {
                // Read Cargo.toml (exists in project root)
                const content = context.readFile("Cargo.toml");
                return { 
                    success: true,
                    hasContent: content.length > 0,
                    hasPackage: content.includes("[package]")
                };
            }
            globalThis.execute = execute;
        "#;
        
        runtime.load_plugin(js_code).unwrap();
        
        let input = json!({});
        let result = runtime.execute_sync(input).unwrap();
        
        assert_eq!(result["success"], true);
        assert_eq!(result["hasContent"], true);
        assert_eq!(result["hasPackage"], true);
    }
}
