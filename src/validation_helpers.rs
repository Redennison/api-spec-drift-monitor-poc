use crate::drift_types::DriftType;
use crate::error::ValidationError;
use jsonschema::{Registry, Validator};
use serde_json::Value;

/// Builds a JSON Schema validator with registry for $ref resolution
pub fn build_validator(
    schema: &Value,
    registry: &Registry,
    error_context: &str,
) -> Result<Validator, ValidationError> {
    jsonschema::options()
        .with_registry(registry.clone())
        .with_base_uri("urn:oas:spec".to_string())
        .build(schema)
        .map_err(|e| {
            ValidationError::SchemaCompilationError(format!(
                "Failed to compile schema for {}: {}",
                error_context, e
            ))
        })
}

/// Formats drift error message
pub fn format_drift_error(drift_type: DriftType, location: &str, message: &str) -> String {
    format!("[{}] at {} - {}", drift_type.as_str(), location, message)
}

/// Formats instance path from JSON Schema validation error
pub fn format_instance_location(instance_path: &str, prefix: &str) -> String {
    if instance_path.is_empty() {
        prefix.to_string()
    } else {
        format!("{}{}", prefix, instance_path)
    }
}
