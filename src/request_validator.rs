use crate::error::ValidationError;
use jsonschema::Validator;
use serde_json::Value;

/// Validator for request body against a JSON Schema
pub struct RequestBodyValidator {
    schema: Validator,
    required: bool,
}

impl RequestBodyValidator {
    /// Create a new RequestBodyValidator from a JSON Schema
    /// 
    /// # Arguments
    /// * `schema_value` - The JSON Schema as a serde_json::Value
    /// * `required` - Whether the request body is required (from OpenAPI spec)
    /// 
    /// # Errors
    /// Returns `ValidationError::SchemaCompilationError` if the schema is invalid
    pub fn new(schema_value: &Value, required: bool) -> Result<Self, ValidationError> {
        let schema = jsonschema::options()
            .build(schema_value)
            .map_err(|e| ValidationError::SchemaCompilationError(e.to_string()))?;

        Ok(Self { schema, required })
    }

    /// Validate a request body against the schema
    /// 
    /// # Arguments
    /// * `body` - Optional JSON body to validate (None if no body was provided)
    /// 
    /// # Errors
    /// Returns `ValidationError::RequestBodyMissing` if body is required but None
    /// Returns `ValidationError::ValidationFailed` if validation fails
    pub fn validate(&self, body: Option<&Value>) -> Result<(), ValidationError> {
        match body {
            None => {
                if self.required {
                    Err(ValidationError::RequestBodyMissing)
                } else {
                    Ok(())
                }
            }
            Some(value) => {
                if self.schema.is_valid(value) {
                    Ok(())
                } else {
                    // Collect all validation errors
                    let error_messages: Vec<String> = self.schema
                        .iter_errors(value)
                        .map(|e| format!("{} at {}", e, e.instance_path))
                        .collect();
                    
                    Err(ValidationError::ValidationFailed(
                        error_messages.join("; ")
                    ))
                }
            }
        }
    }
}

