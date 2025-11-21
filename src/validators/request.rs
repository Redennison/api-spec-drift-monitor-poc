use crate::drift_types::{map_to_drift_type, DriftType, ValidationContext};
use crate::error::ValidationError;
use crate::validation_helpers::{build_validator, format_drift_error, format_instance_location};
use jsonschema::{Registry, Validator};
use serde_json::Value; 

/// Validator for request body against a JSON Schema
pub struct RequestBodyValidator {
    schema: Validator,
    required: bool,
}

impl RequestBodyValidator {
    /// Creates validator with registry for $ref resolution
    pub fn new(
        schema_value: &Value, 
        required: bool,
        registry: &Registry,
    ) -> Result<Self, ValidationError> {
        let schema = build_validator(schema_value, registry, "request body")?;
        Ok(Self { schema, required })
    }

    /// Validates request body against schema
    pub fn validate(&self, body: Option<&Value>) -> Result<(), ValidationError> {
        match body {
            None => {
                if self.required {
                    let drift_error = format_drift_error(
                        DriftType::RequestBodyMissingRequired,
                        "body",
                        "Request body is required but missing"
                    );
                    Err(ValidationError::ValidationFailed(drift_error))
                } else {
                    Ok(())
                }
            }
            Some(value) => {
                if self.schema.is_valid(value) {
                    Ok(())
                } else {
                    let drift_errors: Vec<String> = self.schema
                        .iter_errors(value)
                        .filter_map(|e| {
                            map_to_drift_type(&e.kind, ValidationContext::RequestBody).map(|drift_type| {
                                let location = format_instance_location(&e.instance_path.to_string(), "body");
                                format_drift_error(drift_type, &location, &e.to_string())
                            })
                        })
                        .collect();
                    
                    if drift_errors.is_empty() {
                        Ok(())
                    } else {
                        Err(ValidationError::ValidationFailed(drift_errors.join("; ")))
                    }
                }
            }
        }
    }
}
