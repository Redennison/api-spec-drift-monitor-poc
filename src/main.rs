use api_spec_drift_monitor_poc::{RequestBodyValidator, ResponseValidator, ParameterValidator, ParametersValidator};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    // Example: Create a simple schema for a user registration endpoint
    let schema = json!({
        "type": "object",
        "required": ["username", "email"],
        "properties": {
            "username": {
                "type": "string",
                "minLength": 3
            },
            "email": {
                "type": "string",
                "format": "email"
            },
            "age": {
                "type": "integer",
                "minimum": 18
            }
        }
    });

    // Create validator (required=true means body must be present)
    let validator = RequestBodyValidator::new(&schema, true)
        .expect("Failed to compile schema");

    println!("=== Request Body Validation Examples ===\n");

    // Example 1: Valid request body
    let valid_body = json!({
        "username": "johndoe",
        "email": "john@example.com",
        "age": 25
    });

    println!("Test 1: Valid request body");
    match validator.validate(Some(&valid_body)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Example 2: Invalid request body (missing required field)
    let invalid_body = json!({
        "username": "johndoe"
        // missing required "email" field
    });

    println!("Test 2: Missing required field");
    match validator.validate(Some(&invalid_body)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Example 3: Type mismatch
    let type_mismatch = json!({
        "username": "johndoe",
        "email": "john@example.com",
        "age": "not a number"  // should be integer
    });

    println!("Test 3: Type mismatch");
    match validator.validate(Some(&type_mismatch)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Example 4: Missing body when required
    println!("Test 4: Missing required body");
    match validator.validate(None) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    println!("\n=== Response Validation Examples ===\n");

    // Create a response validator
    let mut response_validator = ResponseValidator::new();

    // Define schema for 200 OK response
    let success_schema = json!({
        "type": "object",
        "required": ["id", "username"],
        "properties": {
            "id": {"type": "integer"},
            "username": {"type": "string"},
            "email": {"type": "string"}
        }
    });

    // Define schema for 404 Not Found response
    let error_schema = json!({
        "type": "object",
        "required": ["error", "message"],
        "properties": {
            "error": {"type": "string"},
            "message": {"type": "string"}
        }
    });

    // Add response schemas
    response_validator.add_response(200, &success_schema).unwrap();
    response_validator.add_response(404, &error_schema).unwrap();

    // Test 1: Valid 200 response
    let success_response = json!({
        "id": 123,
        "username": "johndoe",
        "email": "john@example.com"
    });

    println!("Test 1: Valid 200 OK response");
    match response_validator.validate(200, Some(&success_response)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 2: Valid 404 response
    let error_response = json!({
        "error": "NOT_FOUND",
        "message": "User not found"
    });

    println!("Test 2: Valid 404 Not Found response");
    match response_validator.validate(404, Some(&error_response)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 3: Invalid 200 response (missing required field)
    let invalid_response = json!({
        "id": 123
        // missing required "username"
    });

    println!("Test 3: Invalid 200 response (missing required field)");
    match response_validator.validate(200, Some(&invalid_response)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 4: Status code with no schema defined
    println!("Test 4: Status code with no schema (500)");
    match response_validator.validate(500, Some(&error_response)) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    println!("\n=== Parameter Validation Examples ===\n");

    // Create a parameters validator for an endpoint like GET /users/{id}?limit=10
    let mut params_validator = ParametersValidator::new();

    // Add path parameter: id (required, integer)
    let id_schema = json!({"type": "integer", "minimum": 1});
    let id_validator = ParameterValidator::new("id".to_string(), true, &id_schema).unwrap();
    params_validator.add_path_parameter(id_validator);

    // Add query parameter: limit (optional, integer)
    let limit_schema = json!({"type": "integer", "minimum": 1, "maximum": 100});
    let limit_validator = ParameterValidator::new("limit".to_string(), false, &limit_schema).unwrap();
    params_validator.add_query_parameter(limit_validator);

    // Add header parameter: authorization (required, string)
    let auth_schema = json!({"type": "string", "minLength": 1});
    let auth_validator = ParameterValidator::new("authorization".to_string(), true, &auth_schema).unwrap();
    params_validator.add_header_parameter(auth_validator);

    // Test 1: Valid path parameters
    let mut path_params = HashMap::new();
    path_params.insert("id".to_string(), json!(123));

    println!("Test 1: Valid path parameter (id=123)");
    match params_validator.validate_path(&path_params) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 2: Invalid path parameter (id must be positive)
    let mut invalid_path_params = HashMap::new();
    invalid_path_params.insert("id".to_string(), json!(-5));

    println!("Test 2: Invalid path parameter (id=-5, must be >= 1)");
    match params_validator.validate_path(&invalid_path_params) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 3: Valid query parameters
    let mut query_params = HashMap::new();
    query_params.insert("limit".to_string(), json!(50));

    println!("Test 3: Valid query parameter (limit=50)");
    match params_validator.validate_query(&query_params) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 4: Missing required header
    let empty_headers = HashMap::new();

    println!("Test 4: Missing required header (authorization)");
    match params_validator.validate_headers(&empty_headers) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }

    // Test 5: Valid headers
    let mut valid_headers = HashMap::new();
    valid_headers.insert("authorization".to_string(), json!("Bearer token123"));

    println!("Test 5: Valid header (authorization present)");
    match params_validator.validate_headers(&valid_headers) {
        Ok(_) => println!("✓ Validation passed\n"),
        Err(e) => println!("✗ Validation failed: {}\n", e),
    }
}
