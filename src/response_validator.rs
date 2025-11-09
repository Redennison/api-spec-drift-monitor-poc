use crate::error::ValidationError;
use jsonschema::Validator;
use serde_json::Value;
use std::collections::HashMap;

/// Validator for response bodies against JSON Schemas based on status codes
pub struct ResponseValidator {
    /// Map from exact status codes (200, 404, etc.) to their validators
    exact: HashMap<u16, Validator>,
    /// Optional default validator (for "default" in OpenAPI spec)
    default: Option<Validator>,
}

impl ResponseValidator {
    /// Create a new empty ResponseValidator
    pub fn new() -> Self {
        Self {
            exact: HashMap::new(),
            default: None,
        }
    }

    /// Add a response schema for a specific status code
    /// 
    /// # Arguments
    /// * `status_code` - The HTTP status code (e.g., 200, 404)
    /// * `schema` - The JSON Schema for this response
    /// 
    /// # Errors
    /// Returns `ValidationError::SchemaCompilationError` if the schema is invalid
    pub fn add_response(&mut self, status_code: u16, schema: &Value) -> Result<(), ValidationError> {
        let validator = jsonschema::options()
            .build(schema)
            .map_err(|e| ValidationError::SchemaCompilationError(e.to_string()))?;
        
        self.exact.insert(status_code, validator);
        Ok(())
    }

    /// Set the default response schema (used when no exact status code matches)
    /// 
    /// # Arguments
    /// * `schema` - The JSON Schema for the default response
    /// 
    /// # Errors
    /// Returns `ValidationError::SchemaCompilationError` if the schema is invalid
    pub fn set_default(&mut self, schema: &Value) -> Result<(), ValidationError> {
        let validator = jsonschema::options()
            .build(schema)
            .map_err(|e| ValidationError::SchemaCompilationError(e.to_string()))?;
        
        self.default = Some(validator);
        Ok(())
    }

    /// Validate a response body against the appropriate schema
    /// 
    /// # Arguments
    /// * `status_code` - The HTTP status code of the response
    /// * `body` - Optional response body to validate (None for no body)
    /// 
    /// # Errors
    /// Returns `ValidationError::NoSchemaForStatusCode` if no schema matches the status code
    /// Returns `ValidationError::ValidationFailed` if validation fails
    pub fn validate(&self, status_code: u16, body: Option<&Value>) -> Result<(), ValidationError> {
        // Find the appropriate validator (exact match first, then default)
        let validator = self.exact.get(&status_code)
            .or(self.default.as_ref())
            .ok_or_else(|| ValidationError::NoSchemaForStatusCode(status_code))?;
        
        // Validate the body if present
        match body {
            Some(value) => {
                if validator.is_valid(value) {
                    Ok(())
                } else {
                    // Collect all validation errors
                    let error_messages: Vec<String> = validator
                        .iter_errors(value)
                        .map(|e| format!("{} at {}", e, e.instance_path))
                        .collect();
                    
                    Err(ValidationError::ValidationFailed(
                        error_messages.join("; ")
                    ))
                }
            }
            None => {
                // No body provided - this is valid for responses like 204 No Content
                Ok(())
            }
        }
    }
}

