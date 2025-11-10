use api_spec_drift_monitor_poc::{load_openapi_spec, build_api_validator, HttpMethod};
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    println!("=== API Spec Drift Monitor - Test Suite ===\n");

    // Load spec and build validators
    let spec_path = Path::new("test-api-spec.yaml");
    
    let spec = match load_openapi_spec(spec_path) {
        Ok(spec) => {
            println!("✓ Loaded spec: {} v{}", spec.info.title, spec.info.version);
            spec
        }
        Err(e) => {
            eprintln!("✗ Failed to load spec: {}", e);
            return;
        }
    };

    let api_validator = match build_api_validator(&spec) {
        Ok(validator) => {
            println!("✓ Built API validator successfully\n");
            validator
        }
        Err(e) => {
            eprintln!("✗ Failed to build validator: {}", e);
            return;
        }
    };

    println!("{}\n", "=".repeat(60));

    // Test 1: GET /users with query parameters
    test_get_users(&api_validator);

    println!("\n{}\n", "=".repeat(60));

    // Test 2: POST /users with request body
    test_post_users(&api_validator);

    println!("\n{}\n", "=".repeat(60));

    // Test 3: GET /users/{userId} with path parameter
    test_get_user(&api_validator);

    println!("\n{}\n", "=".repeat(60));

    // Test 4: PUT /users/{userId} with optional request body
    test_put_user(&api_validator);

    println!("\n{}\n", "=".repeat(60));

    // Test 5: DELETE /users/{userId} with required header
    test_delete_user(&api_validator);

    println!("\n{}", "=".repeat(60));
    println!("\n✅ All tests completed!");
}

fn test_get_users(api_validator: &api_spec_drift_monitor_poc::ApiValidator) {
    println!("Test 1: GET /users (List users with query parameters)");
    println!("{}", "-".repeat(60));

    match api_validator.find_operation("/users", HttpMethod::GET) {
        Ok((operation, _)) => {
            println!("✓ Found operation for GET /users");

            // Test 1a: Valid query parameters
            println!("\n  Test 1a: Valid query parameters (limit=10, offset=0)");
            let mut valid_query = HashMap::new();
            valid_query.insert("limit".to_string(), json!(10));
            valid_query.insert("offset".to_string(), json!(0));
            
            match operation.parameters.validate_query(&valid_query) {
                Ok(_) => println!("    ✓ Query parameter validation passed"),
                Err(e) => println!("    ✗ Query parameter validation failed: {}", e),
            }

            // Test 1b: Invalid query parameter (limit too high)
            println!("\n  Test 1b: Invalid query parameter (limit=200, max is 100)");
            let mut invalid_query = HashMap::new();
            invalid_query.insert("limit".to_string(), json!(200));
            
            match operation.parameters.validate_query(&invalid_query) {
                Ok(_) => println!("    ✗ Query parameter validation passed (should have failed)"),
                Err(e) => println!("    ✓ Query parameter validation failed (expected): {}", e),
            }

            // Test 1c: Valid 200 response
            println!("\n  Test 1c: Valid 200 response");
            let valid_response = json!({
                "users": [
                    {
                        "id": 1,
                        "username": "alice",
                        "email": "alice@example.com",
                        "age": 30
                    },
                    {
                        "id": 2,
                        "username": "bob",
                        "email": "bob@example.com"
                    }
                ],
                "total": 2
            });
            
            match operation.responses.validate(200, Some(&valid_response)) {
                Ok(_) => println!("    ✓ Response validation passed"),
                Err(e) => println!("    ✗ Response validation failed: {}", e),
            }

            // Test 1d: Invalid response (missing required field)
            println!("\n  Test 1d: Invalid response (missing 'users' field)");
            let invalid_response = json!({
                "total": 2
            });
            
            match operation.responses.validate(200, Some(&invalid_response)) {
                Ok(_) => println!("    ✗ Response validation passed (should have failed)"),
                Err(e) => println!("    ✓ Response validation failed (expected): {}", e),
            }
        }
        Err(e) => println!("✗ Failed to find operation: {}", e),
    }
}

fn test_post_users(api_validator: &api_spec_drift_monitor_poc::ApiValidator) {
    println!("Test 2: POST /users (Create user with request body)");
    println!("{}", "-".repeat(60));

    match api_validator.find_operation("/users", HttpMethod::POST) {
        Ok((operation, _)) => {
            println!("✓ Found operation for POST /users");

            // Test 2a: Valid request body
            println!("\n  Test 2a: Valid request body");
            let valid_request = json!({
                "username": "charlie",
                "email": "charlie@example.com",
                "password": "securepassword123",
                "age": 25
            });
            
            if let Some(validator) = &operation.request_body {
                match validator.validate(Some(&valid_request)) {
                    Ok(_) => println!("    ✓ Request body validation passed"),
                    Err(e) => println!("    ✗ Request body validation failed: {}", e),
                }
            }

            // Test 2b: Invalid request body (missing required field)
            println!("\n  Test 2b: Invalid request (missing 'password' field)");
            let invalid_request = json!({
                "username": "charlie",
                "email": "charlie@example.com"
            });
            
            if let Some(validator) = &operation.request_body {
                match validator.validate(Some(&invalid_request)) {
                    Ok(_) => println!("    ✗ Request body validation passed (should have failed)"),
                    Err(e) => println!("    ✓ Request body validation failed (expected): {}", e),
                }
            }

            // Test 2c: Invalid request body (username too short)
            println!("\n  Test 2c: Invalid request (username too short, min is 3)");
            let invalid_username = json!({
                "username": "ab",
                "email": "charlie@example.com",
                "password": "securepassword123"
            });
            
            if let Some(validator) = &operation.request_body {
                match validator.validate(Some(&invalid_username)) {
                    Ok(_) => println!("    ✗ Request body validation passed (should have failed)"),
                    Err(e) => println!("    ✓ Request body validation failed (expected): {}", e),
                }
            }

            // Test 2d: Required header validation
            println!("\n  Test 2d: Valid header (x-api-key)");
            let mut valid_headers = HashMap::new();
            valid_headers.insert("x-api-key".to_string(), json!("my-api-key-123"));
            
            match operation.parameters.validate_headers(&valid_headers) {
                Ok(_) => println!("    ✓ Header validation passed"),
                Err(e) => println!("    ✗ Header validation failed: {}", e),
            }

            // Test 2e: Missing required header
            println!("\n  Test 2e: Missing required header (x-api-key)");
            let empty_headers = HashMap::new();
            
            match operation.parameters.validate_headers(&empty_headers) {
                Ok(_) => println!("    ✗ Header validation passed (should have failed)"),
                Err(e) => println!("    ✓ Header validation failed (expected): {}", e),
            }

            // Test 2f: Valid 201 response
            println!("\n  Test 2f: Valid 201 response");
            let valid_response = json!({
                "id": 3,
                "username": "charlie",
                "email": "charlie@example.com",
                "createdAt": "2024-01-01T12:00:00Z"
            });
            
            match operation.responses.validate(201, Some(&valid_response)) {
                Ok(_) => println!("    ✓ Response validation passed"),
                Err(e) => println!("    ✗ Response validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to find operation: {}", e),
    }
}

fn test_get_user(api_validator: &api_spec_drift_monitor_poc::ApiValidator) {
    println!("Test 3: GET /users/{{userId}} (Get user by ID with path parameter)");
    println!("{}", "-".repeat(60));

    match api_validator.find_operation("/users/123", HttpMethod::GET) {
        Ok((operation, params)) => {
            println!("✓ Found operation for GET /users/123");
            
            println!("\n  Path parameters extracted:");
            for (key, value) in params.iter() {
                println!("    {} = {}", key, value);
            }

            // Test 3a: Valid path parameter
            println!("\n  Test 3a: Valid path parameter (userId=123)");
            let mut valid_path = HashMap::new();
            valid_path.insert("userId".to_string(), json!(123));
            
            match operation.parameters.validate_path(&valid_path) {
                Ok(_) => println!("    ✓ Path parameter validation passed"),
                Err(e) => println!("    ✗ Path parameter validation failed: {}", e),
            }

            // Test 3b: Invalid path parameter (userId=0, minimum is 1)
            println!("\n  Test 3b: Invalid path parameter (userId=0, minimum is 1)");
            let mut invalid_path = HashMap::new();
            invalid_path.insert("userId".to_string(), json!(0));
            
            match operation.parameters.validate_path(&invalid_path) {
                Ok(_) => println!("    ✗ Path parameter validation passed (should have failed)"),
                Err(e) => println!("    ✓ Path parameter validation failed (expected): {}", e),
            }

            // Test 3c: Valid 200 response
            println!("\n  Test 3c: Valid 200 response");
            let valid_response = json!({
                "id": 123,
                "username": "alice",
                "email": "alice@example.com",
                "age": 30,
                "createdAt": "2024-01-01T10:00:00Z"
            });
            
            match operation.responses.validate(200, Some(&valid_response)) {
                Ok(_) => println!("    ✓ Response validation passed"),
                Err(e) => println!("    ✗ Response validation failed: {}", e),
            }

            // Test 3d: Valid 404 response
            println!("\n  Test 3d: Valid 404 response");
            let not_found_response = json!({
                "error": "User not found"
            });
            
            match operation.responses.validate(404, Some(&not_found_response)) {
                Ok(_) => println!("    ✓ 404 response validation passed"),
                Err(e) => println!("    ✗ 404 response validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to find operation: {}", e),
    }
}

fn test_put_user(api_validator: &api_spec_drift_monitor_poc::ApiValidator) {
    println!("Test 4: PUT /users/{{userId}} (Update user with optional body)");
    println!("{}", "-".repeat(60));

    match api_validator.find_operation("/users/456", HttpMethod::PUT) {
        Ok((operation, _)) => {
            println!("✓ Found operation for PUT /users/456");

            // Test 4a: Valid request body with partial update
            println!("\n  Test 4a: Valid request body (partial update)");
            let valid_request = json!({
                "username": "alice_updated",
                "age": 31
            });
            
            if let Some(validator) = &operation.request_body {
                match validator.validate(Some(&valid_request)) {
                    Ok(_) => println!("    ✓ Request body validation passed"),
                    Err(e) => println!("    ✗ Request body validation failed: {}", e),
                }
            }

            // Test 4b: No request body (should be valid since it's not required)
            println!("\n  Test 4b: No request body (should be valid)");
            if let Some(validator) = &operation.request_body {
                match validator.validate(None) {
                    Ok(_) => println!("    ✓ No body validation passed"),
                    Err(e) => println!("    ✗ No body validation failed: {}", e),
                }
            }

            // Test 4c: Invalid request body (age too young)
            println!("\n  Test 4c: Invalid request (age=10, minimum is 13)");
            let invalid_request = json!({
                "age": 10
            });
            
            if let Some(validator) = &operation.request_body {
                match validator.validate(Some(&invalid_request)) {
                    Ok(_) => println!("    ✗ Request body validation passed (should have failed)"),
                    Err(e) => println!("    ✓ Request body validation failed (expected): {}", e),
                }
            }

            // Test 4d: Valid 200 response
            println!("\n  Test 4d: Valid 200 response");
            let valid_response = json!({
                "id": 456,
                "username": "alice_updated",
                "email": "alice@example.com",
                "age": 31
            });
            
            match operation.responses.validate(200, Some(&valid_response)) {
                Ok(_) => println!("    ✓ Response validation passed"),
                Err(e) => println!("    ✗ Response validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to find operation: {}", e),
    }
}

fn test_delete_user(api_validator: &api_spec_drift_monitor_poc::ApiValidator) {
    println!("Test 5: DELETE /users/{{userId}} (Delete user)");
    println!("{}", "-".repeat(60));

    match api_validator.find_operation("/users/789", HttpMethod::DELETE) {
        Ok((operation, _)) => {
            println!("✓ Found operation for DELETE /users/789");

            // Test 5a: Valid header
            println!("\n  Test 5a: Valid required header (x-api-key)");
            let mut valid_headers = HashMap::new();
            valid_headers.insert("x-api-key".to_string(), json!("delete-key-xyz"));
            
            match operation.parameters.validate_headers(&valid_headers) {
                Ok(_) => println!("    ✓ Header validation passed"),
                Err(e) => println!("    ✗ Header validation failed: {}", e),
            }

            // Test 5b: Missing required header
            println!("\n  Test 5b: Missing required header");
            let empty_headers = HashMap::new();
            
            match operation.parameters.validate_headers(&empty_headers) {
                Ok(_) => println!("    ✗ Header validation passed (should have failed)"),
                Err(e) => println!("    ✓ Header validation failed (expected): {}", e),
            }

            // Test 5c: Valid 204 response
            println!("\n  Test 5c: Valid 204 response");
            let success_response = json!({
                "success": true
            });
            match operation.responses.validate(204, Some(&success_response)) {
                Ok(_) => println!("    ✓ 204 response validation passed"),
                Err(e) => println!("    ✗ 204 response validation failed: {}", e),
            }

            // Test 5d: Valid 404 response
            println!("\n  Test 5d: Valid 404 response");
            let not_found = json!({
                "error": "User not found"
            });
            
            match operation.responses.validate(404, Some(&not_found)) {
                Ok(_) => println!("    ✓ 404 response validation passed"),
                Err(e) => println!("    ✗ 404 response validation failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to find operation: {}", e),
    }
}
