use crate::drift_types::{map_to_drift_type, ValidationContext};
use crate::error::ValidationError;
use crate::validation_helpers::{build_validator, format_drift_error, format_instance_location};
use jsonschema::{Registry, Validator};
use serde_json::Value;
use std::collections::HashMap;

/// Validator for response bodies against JSON Schemas based on status codes
pub struct ResponseValidator {
    exact: HashMap<u16, Validator>,
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

    /// Adds response schema for a specific status code
    pub fn add_response(
        &mut self,
        status_code: u16,
        schema: &Value,
        registry: &Registry,
    ) -> Result<(), ValidationError> {
        let validator = build_validator(schema, registry, &format!("response {}", status_code))?;
        self.exact.insert(status_code, validator);
        Ok(())
    }

    /// Sets default response schema for unmatched status codes
    pub fn set_default(
        &mut self, 
        schema: &Value,
        registry: &Registry,
    ) -> Result<(), ValidationError> {
        let validator = build_validator(schema, registry, "default response")?;
        self.default = Some(validator);
        Ok(())
    }

    /// Validates response body against schema for the given status code
    pub fn validate(&self, status_code: u16, body: Option<&Value>) -> Result<(), ValidationError> {
        // Find the appropriate validator (exact match first, then default)
        let validator = self.exact.get(&status_code)
            .or(self.default.as_ref())
            .ok_or_else(|| ValidationError::NoSchemaForStatusCode(status_code))?;
        
        match body {
            Some(value) => {
                if validator.is_valid(value) {
                    Ok(())
                } else {
                    let drift_errors: Vec<String> = validator
                        .iter_errors(value)
                        .filter_map(|e| {
                            map_to_drift_type(&e.kind, ValidationContext::ResponseBody).map(|drift_type| {
                                let location = format_instance_location(&e.instance_path.to_string(), "body");
                                format_drift_error(drift_type, &location, &e.to_string())
                            })
                        })
                        .collect();
                    
                    if drift_errors.is_empty() {
                        Ok(()) // No drift-relevant errors
                    } else {
                        Err(ValidationError::ValidationFailed(drift_errors.join("; ")))
                    }
                }
            }
            None => {
                // No body provided - this is valid for responses like 204 No Content
                Ok(())
            }
        }
    }
}
