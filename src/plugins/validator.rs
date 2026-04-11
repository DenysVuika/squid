use log::debug;
use serde_json::Value;

/// Validates JSON data against a JSON schema
pub struct SchemaValidator {
    schema: jsonschema::Validator,
}

impl SchemaValidator {
    /// Create a new validator from a JSON schema
    pub fn new(schema: &Value) -> Result<Self, String> {
        let compiled_schema = jsonschema::validator_for(schema)
            .map_err(|e| format!("Failed to compile JSON schema: {}", e))?;

        Ok(Self {
            schema: compiled_schema,
        })
    }

    /// Validate data against the schema
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

    /// Check if data is valid (returns bool instead of Result)
    pub fn is_valid(&self, data: &Value) -> bool {
        self.schema.is_valid(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_data() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name"]
        });

        let data = json!({
            "name": "Alice",
            "age": 30
        });

        let validator = SchemaValidator::new(&schema).unwrap();
        assert!(validator.validate(&data).is_ok());
    }

    #[test]
    fn test_invalid_data_missing_required() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });

        let data = json!({
            "age": 30
        });

        let validator = SchemaValidator::new(&schema).unwrap();
        let result = validator.validate(&data);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_invalid_data_wrong_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": { "type": "number" }
            }
        });

        let data = json!({
            "age": "not a number"
        });

        let validator = SchemaValidator::new(&schema).unwrap();
        assert!(validator.validate(&data).is_err());
    }

    #[test]
    fn test_is_valid() {
        let schema = json!({
            "type": "string"
        });

        let validator = SchemaValidator::new(&schema).unwrap();

        assert!(validator.is_valid(&json!("hello")));
        assert!(!validator.is_valid(&json!(123)));
    }
}
