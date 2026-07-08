use log::debug;
use serde_json::Value;

/// Runtime JSON schema validator for plugin inputs and outputs.
pub struct SchemaValidator {
    schema: jsonschema::Validator,
}

impl SchemaValidator {
    /// Compiles a JSON schema into a reusable validator.
    pub fn new(schema: &Value) -> Result<Self, String> {
        let compiled_schema = jsonschema::validator_for(schema)
            .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;

        Ok(Self {
            schema: compiled_schema,
        })
    }

    /// Validates data and returns detailed error messages on failure.
    pub fn validate(&self, data: &Value) -> Result<(), Vec<String>> {
        if self.schema.is_valid(data) {
            debug!("Schema validation passed");
            Ok(())
        } else {
            let error_messages: Vec<String> = self
                .schema
                .iter_errors(data)
                .map(|e| format!("{}: {}", e.instance_path(), e))
                .collect();

            debug!("Schema validation failed: {:?}", error_messages);
            Err(error_messages)
        }
    }
}
