//! JSON schema validation for structured AI responses

use crate::ai::{AiError, AiResult};
use serde_json::Value;
use std::collections::HashMap;

/// Schema validator for AI responses
pub struct SchemaValidator {
    schemas: HashMap<String, Value>,
}

impl SchemaValidator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// Register a schema with a name
    ///
    /// # Errors
    ///
    /// Returns an error if the schema is invalid
    pub fn register_schema(&mut self, name: &str, schema: Value) -> AiResult<()> {
        // Basic validation that it's a valid JSON schema
        if !schema.is_object() {
            return Err(AiError::SchemaValidation(
                "Schema must be an object".to_string(),
            ));
        }

        self.schemas.insert(name.to_string(), schema);
        Ok(())
    }

    /// Validate a value against a named schema
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The schema doesn't exist
    /// - The value doesn't match the schema
    pub fn validate(&self, schema_name: &str, value: &Value) -> AiResult<()> {
        let schema = self.schemas.get(schema_name).ok_or_else(|| {
            AiError::SchemaValidation(format!("Schema '{schema_name}' not found"))
        })?;

        // For now, just do basic type checking
        // TODO: Implement full JSON schema validation
        self.validate_against_schema(value, schema)
    }

    /// Validate a value against a schema
    #[allow(clippy::only_used_in_recursion)]
    fn validate_against_schema(&self, value: &Value, schema: &Value) -> AiResult<()> {
        // Basic type validation
        if let Some(schema_type) = schema.get("type").and_then(Value::as_str) {
            match schema_type {
                "object" => {
                    if !value.is_object() {
                        return Err(AiError::SchemaValidation("Expected object".to_string()));
                    }

                    // Validate required properties
                    if let Some(required) = schema.get("required").and_then(Value::as_array) {
                        let obj = value.as_object().ok_or_else(|| {
                            AiError::SchemaValidation("Expected object".to_string())
                        })?;
                        for req in required {
                            if let Some(req_name) = req.as_str() {
                                if !obj.contains_key(req_name) {
                                    return Err(AiError::SchemaValidation(format!(
                                        "Missing required property: {req_name}"
                                    )));
                                }
                            }
                        }
                    }

                    // Validate properties
                    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                        let obj = value.as_object().ok_or_else(|| {
                            AiError::SchemaValidation("Expected object".to_string())
                        })?;
                        for (key, val) in obj {
                            if let Some(prop_schema) = properties.get(key) {
                                self.validate_against_schema(val, prop_schema)?;
                            }
                        }
                    }
                }
                "array" => {
                    if !value.is_array() {
                        return Err(AiError::SchemaValidation("Expected array".to_string()));
                    }

                    // Validate items
                    if let Some(items_schema) = schema.get("items") {
                        let arr = value.as_array().ok_or_else(|| {
                            AiError::SchemaValidation("Expected array".to_string())
                        })?;
                        for item in arr {
                            self.validate_against_schema(item, items_schema)?;
                        }
                    }
                }
                "string" => {
                    if !value.is_string() {
                        return Err(AiError::SchemaValidation("Expected string".to_string()));
                    }
                }
                "number" => {
                    if !value.is_number() {
                        return Err(AiError::SchemaValidation("Expected number".to_string()));
                    }
                }
                "boolean" => {
                    if !value.is_boolean() {
                        return Err(AiError::SchemaValidation("Expected boolean".to_string()));
                    }
                }
                "null" => {
                    if !value.is_null() {
                        return Err(AiError::SchemaValidation("Expected null".to_string()));
                    }
                }
                _ => {
                    return Err(AiError::SchemaValidation(format!(
                        "Unknown type: {schema_type}"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get a list of registered schema names
    #[must_use]
    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Common schemas for structured responses
pub mod schemas {
    use serde_json::json;

    /// Schema for moderation responses
    #[must_use]
    pub fn moderation_response() -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["safe", "issues", "severity", "recommendation"],
            "properties": {
                "safe": {
                    "type": "boolean"
                },
                "issues": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                },
                "severity": {
                    "type": "string",
                    "enum": ["none", "low", "medium", "high"]
                },
                "recommendation": {
                    "type": "string",
                    "enum": ["allow", "review", "block"]
                }
            }
        })
    }

    /// Schema for code analysis responses
    #[must_use]
    pub fn code_analysis() -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["summary", "complexity", "suggestions"],
            "properties": {
                "summary": {
                    "type": "string"
                },
                "complexity": {
                    "type": "string",
                    "enum": ["simple", "moderate", "complex"]
                },
                "issues": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["line", "severity", "message"],
                        "properties": {
                            "line": {
                                "type": "number"
                            },
                            "severity": {
                                "type": "string",
                                "enum": ["info", "warning", "error"]
                            },
                            "message": {
                                "type": "string"
                            }
                        }
                    }
                },
                "suggestions": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_validation() {
        let mut validator = SchemaValidator::new();

        // Register a simple schema
        let schema = json!({
            "type": "object",
            "required": ["name", "age"],
            "properties": {
                "name": {
                    "type": "string"
                },
                "age": {
                    "type": "number"
                }
            }
        });

        validator
            .register_schema("person", schema)
            .expect("Failed to register schema");

        // Valid object
        let valid = json!({
            "name": "John",
            "age": 30
        });
        assert!(validator.validate("person", &valid).is_ok());

        // Missing required field
        let invalid = json!({
            "name": "John"
        });
        assert!(validator.validate("person", &invalid).is_err());

        // Wrong type
        let invalid_type = json!({
            "name": "John",
            "age": "thirty"
        });
        assert!(validator.validate("person", &invalid_type).is_err());
    }
}
