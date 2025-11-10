use crate::api_validator::{ApiValidator, HttpMethod, OperationValidator};
use crate::error::ValidationError;
use openapiv3::OpenAPI;

/// Build an ApiValidator from a parsed OpenAPI specification
/// 
/// This function iterates through all paths and operations in the spec,
/// creates validators for each operation, and populates an ApiValidator
/// that can be used to validate requests and responses.
/// 
/// # Arguments
/// * `spec` - The parsed OpenAPI specification
/// 
/// # Returns
/// A fully populated ApiValidator ready to validate requests/responses
/// 
/// # Errors
/// Returns error if:
/// - Schema conversion fails
/// - Validator construction fails
/// - Route insertion fails (e.g., duplicate or invalid paths)
pub fn build_api_validator(spec: &OpenAPI) -> Result<ApiValidator, ValidationError> {
    let mut api_validator = ApiValidator::new();

    // Iterate through all paths in the spec
    for (path, path_item_ref) in &spec.paths.paths {
        // Resolve ReferenceOr to get the actual PathItem
        let path_item = match path_item_ref {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { reference } => {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Path references ($ref) are not yet supported: {}",
                    reference
                )));
            }
        };

        // Iterate over all operations in the path item
        for (method_str, operation) in path_item.iter() {
            // Convert method string to our HttpMethod enum
            let method = HttpMethod::from_str(method_str).ok_or_else(|| {
                ValidationError::SchemaCompilationError(format!(
                    "Unknown HTTP method: {}",
                    method_str
                ))
            })?;

            let validator = build_operation_validator(operation)?;
            api_validator.add_operation(path, method, validator)?;
        }
    }

    Ok(api_validator)
}

/// Build an OperationValidator from an OpenAPI operation
/// 
/// Extracts and builds validators for:
/// - Request body (if present)
/// - Response bodies for all status codes
/// - Parameters (path, query, header)
/// 
/// # Arguments
/// * `operation` - The OpenAPI operation to build validators for
/// 
/// # Errors
/// Returns error if validator construction fails
fn build_operation_validator(
    operation: &openapiv3::Operation,
) -> Result<OperationValidator, ValidationError> {
    // Build request body validator (if operation has a request body)
    let request_body_validator = if let Some(request_body) = &operation.request_body {
        Some(build_request_body_validator(request_body)?)
    } else {
        None
    };

    // Build response validator for all responses
    let response_validator = build_response_validator(&operation.responses)?;

    // Build parameters validator
    let parameters_validator = build_parameters_validator(&operation.parameters)?;

    // Construct and return the OperationValidator
    Ok(OperationValidator::new(
        request_body_validator,
        response_validator,
        parameters_validator,
    ))
}

/// Build a RequestBodyValidator from an OpenAPI RequestBody
/// 
/// Extracts the JSON schema from the request body and creates a validator.
/// 
/// # Arguments
/// * `request_body` - The OpenAPI request body definition
/// 
/// # Errors
/// Returns error if schema extraction or validator construction fails
fn build_request_body_validator(
    request_body_ref: &openapiv3::ReferenceOr<openapiv3::RequestBody>,
) -> Result<crate::request_validator::RequestBodyValidator, ValidationError> {
    // Resolve ReferenceOr to get the actual RequestBody
    let request_body = match request_body_ref {
        openapiv3::ReferenceOr::Item(item) => item,
        openapiv3::ReferenceOr::Reference { reference } => {
            return Err(ValidationError::SchemaCompilationError(format!(
                "Request body references ($ref) are not yet supported: {}",
                reference
            )));
        }
    };

    // Get the content for application/json
    let media_type = request_body
        .content
        .get("application/json")
        .ok_or_else(|| {
            ValidationError::SchemaCompilationError(
                "Request body must have application/json content".to_string()
            )
        })?;

    // Get the schema from the media type
    let schema_ref = media_type.schema.as_ref().ok_or_else(|| {
        ValidationError::SchemaCompilationError(
            "Request body application/json content must have a schema".to_string()
        )
    })?;

    // Resolve schema reference
    let schema = match schema_ref {
        openapiv3::ReferenceOr::Item(schema) => schema,
        openapiv3::ReferenceOr::Reference { reference } => {
            return Err(ValidationError::SchemaCompilationError(format!(
                "Schema references ($ref) are not yet supported: {}",
                reference
            )));
        }
    };

    // Convert OpenAPI schema to JSON value for jsonschema crate
    let schema_json = serde_json::to_value(schema).map_err(|e| {
        ValidationError::SchemaCompilationError(format!(
            "Failed to convert schema to JSON: {}",
            e
        ))
    })?;

    // Get the required flag (defaults to false if not specified)
    let required = request_body.required;

    // Create and return the RequestBodyValidator
    crate::request_validator::RequestBodyValidator::new(&schema_json, required)
}

/// Build a ResponseValidator from OpenAPI Responses
/// 
/// Iterates through all response status codes and creates validators for each.
/// 
/// # Arguments
/// * `responses` - The OpenAPI responses definition
/// 
/// # Errors
/// Returns error if schema extraction or validator construction fails
fn build_response_validator(
    responses: &openapiv3::Responses,
) -> Result<crate::response_validator::ResponseValidator, ValidationError> {
    let mut response_validator = crate::response_validator::ResponseValidator::new();

    // Iterate through all status code responses
    for (status_code_str, response_ref) in &responses.responses {
        // Parse status code (e.g., "200", "404")
        let status_code = match status_code_str {
            openapiv3::StatusCode::Code(code) => *code,
            openapiv3::StatusCode::Range(_) => {
                // Skip wildcard patterns like "2XX", "4XX" for now
                continue;
            }
        };

        // Resolve ReferenceOr to get the actual Response
        let response = match response_ref {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { reference } => {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Response references ($ref) are not yet supported: {}",
                    reference
                )));
            }
        };

        // Try to get application/json content
        if let Some(media_type) = response.content.get("application/json") {
            // Extract schema if present
            if let Some(schema_ref) = &media_type.schema {
                // Resolve schema reference
                let schema = match schema_ref {
                    openapiv3::ReferenceOr::Item(schema) => schema,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return Err(ValidationError::SchemaCompilationError(format!(
                            "Schema references ($ref) are not yet supported: {}",
                            reference
                        )));
                    }
                };

                // Convert schema to JSON
                let schema_json = serde_json::to_value(schema).map_err(|e| {
                    ValidationError::SchemaCompilationError(format!(
                        "Failed to convert response schema to JSON: {}",
                        e
                    ))
                })?;

                // Add to response validator
                response_validator.add_response(status_code, &schema_json)?;
            }
        }
        // If no application/json or no schema, skip this response
    }

    // Handle default response if present
    if let Some(default_response_ref) = &responses.default {
        // Resolve ReferenceOr
        let default_response = match default_response_ref {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { reference } => {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Default response references ($ref) are not yet supported: {}",
                    reference
                )));
            }
        };

        // Try to get application/json content
        if let Some(media_type) = default_response.content.get("application/json") {
            if let Some(schema_ref) = &media_type.schema {
                let schema = match schema_ref {
                    openapiv3::ReferenceOr::Item(schema) => schema,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return Err(ValidationError::SchemaCompilationError(format!(
                            "Schema references ($ref) are not yet supported: {}",
                            reference
                        )));
                    }
                };

                let schema_json = serde_json::to_value(schema).map_err(|e| {
                    ValidationError::SchemaCompilationError(format!(
                        "Failed to convert default response schema to JSON: {}",
                        e
                    ))
                })?;

                response_validator.set_default(&schema_json)?;
            }
        }
    }

    Ok(response_validator)
}

/// Build a ParametersValidator from OpenAPI Parameters
/// 
/// Groups parameters by location (path, query, header) and creates validators.
/// 
/// # Arguments
/// * `parameters` - List of OpenAPI parameters
/// 
/// # Errors
/// Returns error if schema extraction or validator construction fails
fn build_parameters_validator(
    parameters: &[openapiv3::ReferenceOr<openapiv3::Parameter>],
) -> Result<crate::parameter_validator::ParametersValidator, ValidationError> {
    let mut params_validator = crate::parameter_validator::ParametersValidator::new();

    // Iterate through all parameters
    for parameter_ref in parameters {
        // Resolve ReferenceOr to get the actual Parameter
        let parameter = match parameter_ref {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { reference } => {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Parameter references ($ref) are not yet supported: {}",
                    reference
                )));
            }
        };

        // Extract parameter data based on parameter type
        let (name, required, schema_ref) = match parameter {
            openapiv3::Parameter::Query { parameter_data, .. } => {
                let schema = match &parameter_data.format {
                    openapiv3::ParameterSchemaOrContent::Schema(schema) => schema,
                    openapiv3::ParameterSchemaOrContent::Content(_) => {
                        return Err(ValidationError::SchemaCompilationError(
                            "Content-based parameters are not yet supported".to_string()
                        ));
                    }
                };
                (
                    parameter_data.name.clone(),
                    parameter_data.required,
                    schema,
                )
            }
            openapiv3::Parameter::Header { parameter_data, .. } => {
                let schema = match &parameter_data.format {
                    openapiv3::ParameterSchemaOrContent::Schema(schema) => schema,
                    openapiv3::ParameterSchemaOrContent::Content(_) => {
                        return Err(ValidationError::SchemaCompilationError(
                            "Content-based parameters are not yet supported".to_string()
                        ));
                    }
                };
                (
                    parameter_data.name.clone(),
                    parameter_data.required,
                    schema,
                )
            }
            openapiv3::Parameter::Path { parameter_data, .. } => {
                let schema = match &parameter_data.format {
                    openapiv3::ParameterSchemaOrContent::Schema(schema) => schema,
                    openapiv3::ParameterSchemaOrContent::Content(_) => {
                        return Err(ValidationError::SchemaCompilationError(
                            "Content-based parameters are not yet supported".to_string()
                        ));
                    }
                };
                (
                    parameter_data.name.clone(),
                    parameter_data.required,
                    schema,
                )
            }
            openapiv3::Parameter::Cookie { .. } => {
                // Skip cookie parameters as we decided not to support them
                continue;
            }
        };

        // Resolve schema reference
        let schema = match schema_ref {
            openapiv3::ReferenceOr::Item(schema) => schema,
            openapiv3::ReferenceOr::Reference { reference } => {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Parameter schema references ($ref) are not yet supported: {}",
                    reference
                )));
            }
        };

        // Convert schema to JSON
        let schema_json = serde_json::to_value(schema).map_err(|e| {
            ValidationError::SchemaCompilationError(format!(
                "Failed to convert parameter schema to JSON: {}",
                e
            ))
        })?;

        // Create ParameterValidator
        let param_validator =
            crate::parameter_validator::ParameterValidator::new(name, required, &schema_json)?;

        // Add to appropriate location
        match parameter {
            openapiv3::Parameter::Query { .. } => {
                params_validator.add_query_parameter(param_validator);
            }
            openapiv3::Parameter::Header { .. } => {
                params_validator.add_header_parameter(param_validator);
            }
            openapiv3::Parameter::Path { .. } => {
                params_validator.add_path_parameter(param_validator);
            }
            openapiv3::Parameter::Cookie { .. } => {
                // Already skipped above
            }
        }
    }

    Ok(params_validator)
}

