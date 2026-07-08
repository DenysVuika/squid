use log::debug;
use rquickjs::{Context, Function, Runtime};
use serde_json::Value;
use std::sync::Arc;

use crate::context::PluginContext;

/// Minimal QuickJS wrapper for loading and running plugin scripts.
pub struct PluginRuntime {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
    plugin_context: Option<Arc<PluginContext>>,
}

impl PluginRuntime {
    /// Creates a runtime with memory and stack limits.
    pub fn with_memory_limit(memory_mb: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime = Runtime::new()?;
        runtime.set_memory_limit(memory_mb * 1024 * 1024);
        runtime.set_max_stack_size(1024 * 1024);

        let context = Context::full(&runtime)?;

        Ok(Self {
            runtime,
            context,
            plugin_context: None,
        })
    }

    /// Attaches host context APIs for plugin execution.
    pub fn set_context(&mut self, ctx: Arc<PluginContext>) {
        self.plugin_context = Some(ctx);
    }

    /// Loads plugin JavaScript source into the runtime.
    pub fn load_plugin(&self, js_code: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.context.with(|ctx| {
            Self::apply_sandbox(&ctx)?;

            let _: () = ctx
                .eval(js_code.as_bytes())
                .map_err(|e| format!("Failed to execute plugin code: {}", e))?;

            Ok::<_, Box<dyn std::error::Error>>(())
        })?;

        Ok(())
    }

    /// Executes global `execute(context, input)` and returns JSON value output.
    pub fn execute_sync(&self, input: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let input_json = serde_json::to_string(&input)?;
        let plugin_ctx = self.plugin_context.clone();

        let result_str = self.context.with(|ctx| {
            if let Some(ref pctx) = plugin_ctx {
                Self::inject_context_apis(&ctx, Arc::clone(pctx))?;
            }

            let globals = ctx.globals();
            globals.set("__input_json_str", input_json.as_str())?;
            let json_obj: rquickjs::Object = ctx
                .eval(b"JSON.parse(__input_json_str)")
                .map_err(|e| format!("Failed to parse input JSON: {}", e))?;

            let execute_fn: Function = globals
                .get("execute")
                .map_err(|_| "Plugin must define a global 'execute' function")?;

            let context_obj: rquickjs::Value = globals
                .get("__pluginContext")
                .unwrap_or_else(|_| ctx.eval("({})".as_bytes()).unwrap());

            let result_js: rquickjs::Value = execute_fn
                .call((context_obj, json_obj))
                .map_err(|e| format!("Plugin execution failed: {}", e))?;

            let json_stringify: Function = ctx
                .eval("JSON.stringify".as_bytes())
                .map_err(|e| format!("Failed to get JSON.stringify: {}", e))?;

            let result_str: String = json_stringify
                .call((result_js,))
                .map_err(|e| format!("Failed to stringify result: {}", e))?;

            Ok::<_, Box<dyn std::error::Error>>(result_str)
        })?;

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

        let result_value: Value = serde_json::from_str(&result_str)?;

        Ok(result_value)
    }

    /// Applies basic sandbox restrictions and installs `console.log` capture.
    fn apply_sandbox(ctx: &rquickjs::Ctx<'_>) -> Result<(), Box<dyn std::error::Error>> {
        let _: () = ctx
            .eval(
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
            "#
                .as_bytes(),
            )
            .map_err(|e| format!("Failed to inject console: {}", e))?;

        let _: () = ctx
            .eval(
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
            "#
                .as_bytes(),
            )
            .map_err(|e| format!("Failed to apply sandbox restrictions: {}", e))?;

        debug!("Applied sandbox restrictions to QuickJS context");

        Ok(())
    }

    /// Injects host-backed context APIs into the JavaScript global scope.
    fn inject_context_apis(
        ctx: &rquickjs::Ctx<'_>,
        plugin_context: Arc<PluginContext>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let globals = ctx.globals();

        let pctx_read = Arc::clone(&plugin_context);
        let read_fn = rquickjs::Function::new(ctx.clone(), move |path: String| -> String {
            match pctx_read.read_file(&path) {
                Ok(content) => content,
                Err(e) => panic!("{}", e),
            }
        })?;
        globals.set("__rustReadFile", read_fn)?;

        let pctx_write = Arc::clone(&plugin_context);
        let write_fn =
            rquickjs::Function::new(ctx.clone(), move |path: String, content: String| -> bool {
                match pctx_write.write_file(&path, &content) {
                    Ok(_) => true,
                    Err(e) => panic!("{}", e),
                }
            })?;
        globals.set("__rustWriteFile", write_fn)?;

        let pctx_http = Arc::clone(&plugin_context);
        let http_fn = rquickjs::Function::new(
            ctx.clone(),
            move |url: String, timeout: Option<u64>| -> String {
                let result =
                    tokio::runtime::Handle::current().block_on(pctx_http.http_get(&url, timeout));
                match result {
                    Ok(content) => content,
                    Err(e) => panic!("{}", e),
                }
            },
        )?;
        globals.set("__rustHttpGet", http_fn)?;

        let context_code = format!(
            r#"
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
        "#,
            plugin_context
                .project_dir()
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
        );

        let _: () = ctx
            .eval(context_code.as_bytes())
            .map_err(|e| format!("Failed to inject context: {}", e))?;

        Ok(())
    }
}
