use crate::error::ValidationError;
use openapiv3::OpenAPI;
use std::fs::File;
use std::path::Path;

/// Loads an OpenAPI specification from a YAML file
pub fn load_openapi_spec(path: &Path) -> Result<OpenAPI, ValidationError> {
    let file = File::open(path).map_err(|e| {
        ValidationError::SchemaCompilationError(format!("Failed to open spec file: {}", e))
    })?;

    let spec: OpenAPI = serde_yaml::from_reader(file).map_err(|e| {
        ValidationError::SchemaCompilationError(format!("Failed to parse OpenAPI spec: {}", e))
    })?;

    Ok(spec)
}
