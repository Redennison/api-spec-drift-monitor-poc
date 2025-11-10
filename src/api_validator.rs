use crate::error::ValidationError;
use crate::parameter_validator::ParametersValidator;
use crate::request_validator::RequestBodyValidator;
use crate::response_validator::ResponseValidator;
use matchit::Router;
use std::collections::HashMap;

/// HTTP methods supported by OpenAPI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    TRACE,
}

impl HttpMethod {
    /// Convert from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "PATCH" => Some(HttpMethod::PATCH),
            "HEAD" => Some(HttpMethod::HEAD),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "TRACE" => Some(HttpMethod::TRACE),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::TRACE => "TRACE",
        }
    }
}

/// Validator for a single API operation (path + method combination)
/// 
/// Contains all validators needed to validate requests and responses for one operation
pub struct OperationValidator {
    /// Validator for request body (None if operation doesn't accept a body)
    pub request_body: Option<RequestBodyValidator>,
    
    /// Validator for response bodies by status code
    pub responses: ResponseValidator,
    
    /// Validators for path, query, and header parameters
    pub parameters: ParametersValidator,
}

impl OperationValidator {
    /// Create a new OperationValidator
    /// 
    /// # Arguments
    /// * `request_body` - Optional request body validator
    /// * `responses` - Response validator
    /// * `parameters` - Parameters validator
    pub fn new(
        request_body: Option<RequestBodyValidator>,
        responses: ResponseValidator,
        parameters: ParametersValidator,
    ) -> Self {
        Self {
            request_body,
            responses,
            parameters,
        }
    }
}

/// Map of HTTP methods to their operation validators
type OperationMap = HashMap<HttpMethod, OperationValidator>;

/// Top-level API validator that validates requests/responses against an OpenAPI spec
/// 
/// Uses a router to match request paths (including path parameters) and lookup
/// the appropriate operation validators
pub struct ApiValidator {
    /// Router for matching request paths to operation validators
    /// Each path maps to a HashMap of HTTP methods
    router: Router<OperationMap>,
}

impl ApiValidator {
    /// Create a new empty ApiValidator
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Add an operation validator for a specific path and HTTP method
    /// 
    /// # Arguments
    /// * `path` - The path pattern (e.g., "/users/{id}")
    /// * `method` - The HTTP method
    /// * `validator` - The operation validator
    /// 
    /// # Errors
    /// Returns error if path pattern is invalid or conflicts with existing routes
    pub fn add_operation(
        &mut self,
        path: &str,
        method: HttpMethod,
        validator: OperationValidator,
    ) -> Result<(), ValidationError> {
        // Try to find existing operations for this path
        match self.router.at_mut(path) {
            Ok(matched) => {
                // Path exists, add method to existing map
                matched.value.insert(method, validator);
                Ok(())
            }
            Err(_) => {
                // Path doesn't exist, create new map and insert
                let mut operations = HashMap::new();
                operations.insert(method, validator);
                
                self.router.insert(path, operations).map_err(|e| {
                    ValidationError::SchemaCompilationError(format!(
                        "Failed to add route '{}': {}",
                        path, e
                    ))
                })?;
                
                Ok(())
            }
        }
    }

    /// Find the operation validator for a given path and method
    /// 
    /// # Arguments
    /// * `path` - The actual request path (e.g., "/users/123")
    /// * `method` - The HTTP method
    /// 
    /// # Returns
    /// Returns the matched operation validator and extracted path parameters
    /// 
    /// # Errors
    /// Returns error if no matching route or method is found
    pub fn find_operation<'a>(
        &'a self,
        path: &'a str,
        method: HttpMethod,
    ) -> Result<(&'a OperationValidator, matchit::Params<'a, 'a>), ValidationError> {
        // Match the path
        let matched = self.router.at(path).map_err(|_| {
            ValidationError::ValidationFailed(format!("No route found for path: {}", path))
        })?;

        // Find the operation for this method
        let operation = matched.value.get(&method).ok_or_else(|| {
            ValidationError::ValidationFailed(format!(
                "Method {} not allowed for path: {}",
                method.as_str(),
                path
            ))
        })?;

        Ok((operation, matched.params))
    }
}

