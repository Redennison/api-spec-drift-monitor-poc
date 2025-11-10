use crate::error::ValidationError;
use openapiv3::OpenAPI;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Load and parse an OpenAPI specification from a YAML file
/// 
/// # Arguments
/// * `path` - Path to the OpenAPI YAML file
/// 
/// # Errors
/// Returns error if file cannot be read or spec cannot be parsed
pub fn load_openapi_spec(path: &Path) -> Result<OpenAPI, ValidationError> {
    let file = File::open(path).map_err(|e| {
        ValidationError::SchemaCompilationError(format!("Failed to open spec file: {}", e))
    })?;

    let reader = BufReader::new(file);

    let spec: OpenAPI = serde_yaml::from_reader(reader).map_err(|e| {
        ValidationError::SchemaCompilationError(format!("Failed to parse OpenAPI spec: {}", e))
    })?;

    Ok(spec)
}

