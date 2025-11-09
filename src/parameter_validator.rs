use crate::error::ValidationError;
use jsonschema::Validator;
use serde_json::Value;
use std::collections::HashMap;

/// Validator for a single parameter
pub struct ParameterValidator {
    name: String,
    required: bool,
    validator: Validator,
}

impl ParameterValidator {
    /// Create a new ParameterValidator
    /// 
    /// # Arguments
    /// * `name` - The parameter name
    /// * `required` - Whether the parameter is required
    /// * `schema` - The JSON Schema for this parameter
    /// 
    /// # Errors
    /// Returns `ValidationError::SchemaCompilationError` if the schema is invalid
    pub fn new(name: String, required: bool, schema: &Value) -> Result<Self, ValidationError> {
        let validator = jsonschema::options()
            .build(schema)
            .map_err(|e| ValidationError::SchemaCompilationError(e.to_string()))?;
        
        Ok(Self {
            name,
            required,
            validator,
        })
    }

    /// Validate a parameter value
    /// 
    /// # Arguments
    /// * `value` - The parameter value to validate
    /// 
    /// # Errors
    /// Returns `ValidationError::ValidationFailed` if validation fails
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        if self.validator.is_valid(value) {
            Ok(())
        } else {
            let error_messages: Vec<String> = self.validator
                .iter_errors(value)
                .map(|e| format!("{} at {}", e, e.instance_path))
                .collect();
            
            Err(ValidationError::ValidationFailed(
                format!("Parameter '{}': {}", self.name, error_messages.join("; "))
            ))
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
        Self {
            path: Vec::new(),
            query: Vec::new(),
            header: Vec::new(),
        }
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
    /// 
    /// # Arguments
    /// * `params` - Map of parameter names to their values
    /// 
    /// # Errors
    /// Returns `ValidationError` if any required parameter is missing or validation fails
    pub fn validate_path(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.path, params, "path")
    }

    /// Validate query parameters
    /// 
    /// # Arguments
    /// * `params` - Map of parameter names to their values
    /// 
    /// # Errors
    /// Returns `ValidationError` if any required parameter is missing or validation fails
    pub fn validate_query(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.query, params, "query")
    }

    /// Validate header parameters
    /// 
    /// # Arguments
    /// * `params` - Map of parameter names to their values
    /// 
    /// # Errors
    /// Returns `ValidationError` if any required parameter is missing or validation fails
    pub fn validate_headers(&self, params: &HashMap<String, Value>) -> Result<(), ValidationError> {
        self.validate_parameters(&self.header, params, "header")
    }

    /// Internal helper to validate a set of parameters
    fn validate_parameters(
        &self,
        validators: &[ParameterValidator],
        params: &HashMap<String, Value>,
        location: &str,
    ) -> Result<(), ValidationError> {
        for validator in validators {
            match params.get(validator.name()) {
                Some(value) => {
                    // Parameter is present, validate it
                    validator.validate(value)?;
                }
                None => {
                    // Parameter is missing
                    if validator.is_required() {
                        return Err(ValidationError::ValidationFailed(
                            format!(
                                "Required {} parameter '{}' is missing",
                                location,
                                validator.name()
                            )
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

