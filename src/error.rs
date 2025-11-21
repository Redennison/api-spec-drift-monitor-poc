use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Request body is required but was not provided")]
    RequestBodyMissing,

    #[error("No schema defined for status code {0}")]
    NoSchemaForStatusCode(u16),

    #[error("Failed to compile JSON schema: {0}")]
    SchemaCompilationError(String),
}
