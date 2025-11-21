use crate::api_validator::{ApiValidator, HttpMethod, OperationValidator};
use crate::error::ValidationError;
use crate::spec::reference_resolver::ResolveReference;
use jsonschema::{Registry, Resource};
use openapiv3::OpenAPI;
use serde_json::{self, Value};
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::str::FromStr;

/// Converts a schema reference to JSON Value
fn schema_to_json(schema_ref: &impl serde::Serialize, context: &str) -> Result<Value, ValidationError> {
    serde_json::to_value(schema_ref).map_err(|e| {
        ValidationError::SchemaCompilationError(format!(
            "Failed to convert {} schema to JSON: {}",
            context, e
        ))
    })
}

/// Extracts JSON schema from application/json content
fn extract_json_schema(
    content: &openapiv3::Content,
    context: &str
) -> Result<Value, ValidationError> {
    let media_type = content.get("application/json")
        .ok_or_else(|| ValidationError::SchemaCompilationError(
            format!("{} must have application/json content", context)
        ))?;
    
    let schema_ref = media_type.schema.as_ref()
        .ok_or_else(|| ValidationError::SchemaCompilationError(
            format!("{} schema is missing", context)
        ))?;
    
    schema_to_json(schema_ref, context)
}

/// Builds JSON Schema registry from OpenAPI components section
fn build_registry(spec: &OpenAPI) -> Result<Registry, ValidationError> {
    let spec_json_val = serde_json::to_value(spec).map_err(|e| {
        ValidationError::SchemaCompilationError(format!("Failed to serialize spec to JSON: {}", e))
    })?;
    
    let components_json = spec_json_val.get("components")
        .ok_or_else(|| ValidationError::SchemaCompilationError("No components section in spec".to_string()))?
        .clone();
    
    let wrapped_components = serde_json::json!({
        "components": components_json
    });
    
    let components_resource = Resource::from_contents(wrapped_components)
        .map_err(|e| ValidationError::SchemaCompilationError(format!("Failed to create resource: {}", e)))?;
    
    Registry::try_new("urn:oas:spec", components_resource)
        .map_err(|e| ValidationError::SchemaCompilationError(format!("Failed to create registry: {}", e)))
}

/// Build an ApiValidator from a parsed OpenAPI specification
pub fn build_api_validator(spec: &OpenAPI) -> Result<ApiValidator, ValidationError> {
    let mut api_validator = ApiValidator::new();
    let registry = build_registry(spec)?;

    let total_operations: usize = spec.paths.paths.values()
        .filter_map(|path_item_ref| path_item_ref.as_item())
        .map(|path_item| path_item.iter().count())
        .sum();

    if total_operations == 0 {
        println!("--- ✅ No operations found to build. ---");
        return Ok(api_validator);
    }

    let mut completed_operations = 0;

    for (path, path_item_ref) in &spec.paths.paths {
        let path_item = match path_item_ref {
            openapiv3::ReferenceOr::Item(item) => item,
            openapiv3::ReferenceOr::Reference { reference } => {
                eprintln!("\nWARNING: Skipping path. Path references ($ref) are not yet supported: {}", reference);
                continue; 
            }
        };

        // Collect all operations for this path into a HashMap
        let mut operations_map = HashMap::new();
        
        for (method_str, operation) in path_item.iter() {
            let method = HttpMethod::from_str(method_str).map_err(|_| {
                ValidationError::SchemaCompilationError(format!(
                    "Unknown HTTP method: {}",
                    method_str
                ))
            })?;

            let validator = build_operation_validator(spec, &registry, operation)?;
            operations_map.insert(method, validator);
            
            completed_operations += 1;
            let percentage = (completed_operations as f64 / total_operations as f64) * 100.0;
            print!(
                "\r--- 🛠️ Building API Validator: {:.0}% complete ({}/{}) ---",
                percentage, completed_operations, total_operations
            );
            stdout().flush().unwrap_or(()); 
        }
        
        // Insert all operations for this path at once
        api_validator.add_path_operations(path, operations_map)?;
    }

    println!();
    println!("--- ✅ Build Complete ---");
    Ok(api_validator)
}

/// Build an OperationValidator from an OpenAPI operation
fn build_operation_validator(
    spec: &OpenAPI,
    registry: &Registry,
    operation: &openapiv3::Operation,
) -> Result<OperationValidator, ValidationError> {
    let parameters_validator =
        build_parameters_validator(spec, registry, &operation.parameters)?;

    let request_body_validator = if let Some(request_body) = &operation.request_body {
        Some(build_request_body_validator(
            spec,
            registry,
            request_body,
        )?)
    } else {
        None
    };

    let response_validator =
        build_response_validator(spec, registry, &operation.responses)?;

    Ok(OperationValidator::new(
        request_body_validator,
        response_validator,
        parameters_validator,
    ))
}

/// Build a RequestBodyValidator from an OpenAPI RequestBody
fn build_request_body_validator(
    spec: &OpenAPI,
    registry: &Registry,
    request_body_ref: &openapiv3::ReferenceOr<openapiv3::RequestBody>,
) -> Result<crate::validators::RequestBodyValidator, ValidationError> {
    let request_body = request_body_ref.resolve(spec)?;
    let schema_json = extract_json_schema(&request_body.content, "request body")?;
    let required = request_body.required;

    crate::validators::RequestBodyValidator::new(&schema_json, required, registry)
}

/// Build a ResponseValidator from OpenAPI Responses
fn build_response_validator(
    spec: &OpenAPI,
    registry: &Registry,
    responses: &openapiv3::Responses,
) -> Result<crate::validators::ResponseValidator, ValidationError> {
    let mut response_validator = crate::validators::ResponseValidator::new();

    for (status_code_str, response_ref) in &responses.responses {
        let status_code = match status_code_str {
            openapiv3::StatusCode::Code(code) => *code,
            openapiv3::StatusCode::Range(_) => continue,
        };

        let response = response_ref.resolve(spec)?;

        if !response.content.is_empty() {
            if let Ok(schema_json) = extract_json_schema(&response.content, "response") {
                response_validator.add_response(status_code, &schema_json, registry)?;
            }
        }
    }

    if let Some(default_response_ref) = &responses.default {
        let default_response = default_response_ref.resolve(spec)?;

        if !default_response.content.is_empty() {
            if let Ok(schema_json) = extract_json_schema(&default_response.content, "default response") {
                response_validator.set_default(&schema_json, registry)?;
            }
        }
    }

    Ok(response_validator)
}

/// Build a ParametersValidator from OpenAPI Parameters
fn build_parameters_validator(
    spec: &OpenAPI,
    registry: &Registry,
    parameters: &[openapiv3::ReferenceOr<openapiv3::Parameter>],
) -> Result<crate::validators::ParametersValidator, ValidationError> {
    let mut params_validator = crate::validators::ParametersValidator::new();

    for parameter_ref in parameters {
        let parameter = parameter_ref.resolve(spec)?;

        let parameter_data = match parameter {
            openapiv3::Parameter::Query { parameter_data, .. } 
            | openapiv3::Parameter::Path { parameter_data, .. } => parameter_data,
            openapiv3::Parameter::Header { .. } | openapiv3::Parameter::Cookie { .. } => continue,
        };

        let schema_ref = match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(s) => s,
            _ => return Err(ValidationError::SchemaCompilationError(
                "Content-based parameters not supported".to_string()
            )),
        };

        let name = parameter_data.name.clone();
        let required = parameter_data.required;

        let schema_json = schema_to_json(schema_ref, "parameter")?;

        let param_validator = crate::validators::ParameterValidator::new(
            name,
            required,
            &schema_json,
            registry,
        )?;

        match parameter {
            openapiv3::Parameter::Query { .. } => params_validator.add_query_parameter(param_validator),
            openapiv3::Parameter::Header { .. } => params_validator.add_header_parameter(param_validator),
            openapiv3::Parameter::Path { .. } => params_validator.add_path_parameter(param_validator),
            openapiv3::Parameter::Cookie { .. } => {}
        }
    }

    Ok(params_validator)
}