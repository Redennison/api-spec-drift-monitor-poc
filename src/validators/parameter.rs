use crate::drift_types::{map_to_drift_type, DriftType, ValidationContext};
use crate::error::ValidationError;
use crate::validation_helpers::{build_validator, format_drift_error};
use jsonschema::{Registry, Validator};
use serde_json::Value;
use std::collections::HashMap;

/// Validator for a single parameter
#[derive(Debug)]
pub struct ParameterValidator {
    name: String,
    required: bool,
    validator: Validator,
}

impl ParameterValidator {
    /// Creates validator with registry for $ref resolution
    pub fn new(
        name: String,
        required: bool,
        schema: &Value,
        registry: &Registry,
    ) -> Result<Self, ValidationError> {
        let validator = build_validator(schema, registry, &format!("parameter '{}'", name))?;
        Ok(Self {
            name,
            required,
            validator,
        })
    }

    /// Validate a parameter value
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        if self.validator.is_valid(value) {
            Ok(())
        } else {
            let drift_errors: Vec<String> = self
                .validator
                .iter_errors(value)
                .filter_map(|e| {
                    map_to_drift_type(&e.kind, ValidationContext::Parameter).map(|drift_type| {
                        let location = if e.instance_path.to_string().is_empty() {
                            self.name.clone()
                        } else {
                            format!("{}[{}]", self.name, e.instance_path)
                        };
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

    /// Get the parameter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if the parameter is required
    pub fn is_required(&self) -> bool {
        self.required
    }
}

/// Validator for all parameters of an operation
#[derive(Default, Debug)]
pub struct ParametersValidator {
    /// Path parameters (e.g., /users/{id})
    path: Vec<ParameterValidator>,
    /// Query parameters (e.g., ?page=1&limit=10)
    query: Vec<ParameterValidator>,
    /// Header parameters
    header: Vec<ParameterValidator>,
}

impl ParametersValidator {
    /// Create a new empty ParametersValidator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a path parameter validator
    pub fn add_path_parameter(&mut self, validator: ParameterValidator) {
        self.path.push(validator);
    }

    /// Add a query parameter validator
    pub fn add_query_parameter(&mut self, validator: ParameterValidator) {
        self.query.push(validator);
    }

    /// Add a header parameter validator
    pub fn add_header_parameter(&mut self, validator: ParameterValidator) {
        self.header.push(validator);
    }

    /// Validate path parameters
    pub fn validate_path(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.path, params, "path")
    }

    /// Validate query parameters
    pub fn validate_query(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.query, params, "query")
    }

    /// Validate header parameters
    pub fn validate_headers(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.header, params, "header")
    }

    /// Internal helper to validate a set of parameters
    fn validate_parameters(
        &self,
        validators: &[ParameterValidator],
        params: &HashMap<String, Value>,
        _location: &str,
    ) -> Result<(), ValidationError> {
        for validator in validators {
            match params.get(validator.name()) {
                Some(value) => {
                    validator.validate(value)?;
                }
                None => {
                    if validator.is_required() {
                        let drift_error = format_drift_error(
                            DriftType::ParameterMissingRequired,
                            validator.name(),
                            &format!("Required parameter '{}' is missing", validator.name())
                        );
                        return Err(ValidationError::ValidationFailed(drift_error));
                    }
                }
            }
        }
        Ok(())
    }
}
