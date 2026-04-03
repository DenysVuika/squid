//! Template rendering module with secure context variables
//!
//! This module provides template rendering capabilities using the Tera template engine.
//! It automatically populates a context with secure, privacy-safe system information
//! that can be used in agent prompts via variable substitution.
//!
//! # Example
//!
//! ```rust
//! use squid::template::TemplateRenderer;
//!
//! let renderer = TemplateRenderer::new();
//! let prompt = "You are a code reviewer on {{os}} ({{arch}}) at {{now}}.";
//! let rendered = renderer.render_string(prompt)?;
//! // Output: "You are a code reviewer on macOS (aarch64) at 2026-03-28T12:34:56+00:00."
//! ```

use chrono::Local;
use log::debug;
use sysinfo::System;
use tera::{Context, Tera};
use uuid::Uuid;

// Load persona content for template variable
const PERSONA: &str = include_str!("./assets/persona.md");

/// Template renderer with built-in secure context variables
pub struct TemplateRenderer {
    tera: Tera,
    context: Context,
}

impl TemplateRenderer {
    /// Creates a new template renderer with default secure context variables
    pub fn new() -> Self {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![]); // Disable autoescaping for markdown/code templates

        let context = Self::build_default_context();

        debug!("Template renderer initialized with secure context variables");

        Self { tera, context }
    }

    /// Builds the default context with secure, privacy-safe variables
    fn build_default_context() -> Context {
        let mut context = Context::new();

        // Base persona (available as {{persona}} in agent prompts)
        context.insert("persona", PERSONA);

        // Current timestamp in ISO 8601 format
        let now = Local::now();
        context.insert("now", &now.to_rfc3339());

        // Date and time components (useful for time-based tasks)
        context.insert("date", &now.format("%Y-%m-%d").to_string());
        context.insert("time", &now.format("%H:%M:%S").to_string());
        context.insert("year", &now.format("%Y").to_string());
        context.insert("month", &now.format("%m").to_string());
        context.insert("day", &now.format("%d").to_string());

        // Unix timestamp (sometimes useful for calculations)
        context.insert("timestamp", &now.timestamp());

        // Timezone information
        context.insert("timezone", &now.format("%Z").to_string());
        context.insert("timezone_offset", &now.format("%z").to_string());

        // Operating system information
        if let Some(os_name) = System::name() {
            context.insert("os", &os_name);
        } else {
            context.insert("os", "unknown");
        }

        if let Some(os_version) = System::os_version() {
            context.insert("os_version", &os_version);
        } else {
            context.insert("os_version", "unknown");
        }

        if let Some(kernel_version) = System::kernel_version() {
            context.insert("kernel_version", &kernel_version);
        } else {
            context.insert("kernel_version", "unknown");
        }

        // System architecture
        context.insert("arch", std::env::consts::ARCH);
        context.insert("os_family", std::env::consts::FAMILY);

        debug!("Built template context with secure variables");

        context
    }

    /// Renders a template string with the built-in context
    pub fn render_string(&self, template: &str) -> Result<String, tera::Error> {
        // Create a unique template name to avoid collisions
        let template_name = format!("inline_{}", Uuid::new_v4().simple());

        // Clone tera instance to add template temporarily
        let mut tera = self.tera.clone();
        tera.add_raw_template(&template_name, template)?;

        tera.render(&template_name, &self.context)
    }

    /// Renders a template string with custom context variables (replaces defaults)
    pub fn render_string_with_context(
        &self,
        template: &str,
        custom_context: &Context,
    ) -> Result<String, tera::Error> {
        let template_name = format!("inline_{}", Uuid::new_v4().simple());

        let mut tera = self.tera.clone();
        tera.add_raw_template(&template_name, template)?;

        // Use custom context directly (cannot easily merge tera::Context objects)
        tera.render(&template_name, custom_context)
    }

    /// Gets the current context (useful for debugging)
    pub fn get_context(&self) -> &Context {
        &self.context
    }

    /// Adds or updates a context variable
    pub fn insert<T: serde::Serialize>(&mut self, key: &str, value: &T) {
        self.context.insert(key, value);
    }

    /// Removes a context variable
    pub fn remove(&mut self, key: &str) {
        self.context.remove(key);
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_template_rendering() {
        let renderer = TemplateRenderer::new();
        let template = "Hello, world!";
        let result = renderer.render_string(template).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_datetime_variables() {
        let renderer = TemplateRenderer::new();
        let template = "Current time: {{ now }}";
        let result = renderer.render_string(template).unwrap();
        assert!(result.contains("Current time:"));
        assert!(result.contains("T")); // ISO 8601 contains 'T'
    }

    #[test]
    fn test_os_variables() {
        let renderer = TemplateRenderer::new();
        let template = "OS: {{ os }}, Arch: {{ arch }}";
        let result = renderer.render_string(template).unwrap();
        assert!(result.contains("OS:"));
        assert!(result.contains("Arch:"));
        assert!(!result.contains("unknown") || System::name().is_none());
    }

    #[test]
    fn test_date_components() {
        let renderer = TemplateRenderer::new();
        let template = "Date: {{ year }}-{{ month }}-{{ day }}";
        let result = renderer.render_string(template).unwrap();
        assert!(result.contains("Date:"));
        assert!(result.len() > 6); // Should have actual date values
    }

    #[test]
    fn test_custom_context_override() {
        let renderer = TemplateRenderer::new();
        let mut custom = Context::new();
        custom.insert("os", "CustomOS");

        let template = "OS: {{ os }}";
        let result = renderer
            .render_string_with_context(template, &custom)
            .unwrap();
        assert_eq!(result, "OS: CustomOS");
    }

    #[test]
    fn test_custom_context_addition() {
        let renderer = TemplateRenderer::new();
        let mut custom = Context::new();
        custom.insert("custom_var", "custom_value");

        let template = "Custom: {{ custom_var }}";
        let result = renderer
            .render_string_with_context(template, &custom)
            .unwrap();
        assert_eq!(result, "Custom: custom_value");
    }

    #[test]
    fn test_markdown_formatting() {
        let renderer = TemplateRenderer::new();
        let template = "# Greeting\n\nHello, **{{ os }}**!\n\nCurrent time: `{{ now }}`";
        let result = renderer.render_string(template).unwrap();
        assert!(result.contains("# Greeting"));
        assert!(result.contains("**"));
        assert!(result.contains("`"));
    }

    #[test]
    fn test_insert_and_remove() {
        let mut renderer = TemplateRenderer::new();
        renderer.insert("test_key", &"test_value");

        let template = "Value: {{ test_key }}";
        let result = renderer.render_string(template).unwrap();
        assert_eq!(result, "Value: test_value");

        renderer.remove("test_key");
        let result = renderer.render_string(template);
        assert!(result.is_err()); // Should fail because variable doesn't exist
    }

    #[test]
    fn test_multiple_templates() {
        let renderer = TemplateRenderer::new();

        let template1 = "OS: {{ os }}";
        let result1 = renderer.render_string(template1).unwrap();

        let template2 = "Arch: {{ arch }}";
        let result2 = renderer.render_string(template2).unwrap();

        assert!(result1.contains("OS:"));
        assert!(result2.contains("Arch:"));
    }

    #[test]
    fn test_agent_prompt_rendering() {
        let renderer = TemplateRenderer::new();

        // Simulate an agent prompt with template variables
        let agent_prompt = "You are an expert code reviewer working on {{os}} ({{arch}}) at {{now}}. Focus on security vulnerabilities, performance issues, code quality, and maintainability.";

        let result = renderer.render_string(agent_prompt).unwrap();

        // Verify template variables were replaced
        assert!(!result.contains("{{"));
        assert!(!result.contains("}}"));
        assert!(result.contains("You are an expert code reviewer working on"));
        assert!(result.contains("Focus on security"));

        // Verify specific variables were substituted
        assert!(!result.contains("{{os}}"));
        assert!(!result.contains("{{arch}}"));
        assert!(!result.contains("{{now}}"));
    }

    #[test]
    fn test_all_datetime_variables() {
        let renderer = TemplateRenderer::new();

        let template = "Date: {{date}}, Time: {{time}}, Year: {{year}}, TZ: {{timezone}}";
        let result = renderer.render_string(template).unwrap();

        assert!(result.contains("Date:"));
        assert!(result.contains("Time:"));
        assert!(result.contains("Year:"));
        assert!(result.contains("TZ:"));
        assert!(!result.contains("{{"));
    }

    #[test]
    fn test_persona_variable() {
        let renderer = TemplateRenderer::new();

        let template = "{{persona}}\n\nAdditional instructions: Be concise.";
        let result = renderer.render_string(template).unwrap();

        // Verify persona content is included
        assert!(result.contains("squid"));
        assert!(result.contains("AI assistant"));
        assert!(result.contains("Additional instructions: Be concise."));
    }
}
