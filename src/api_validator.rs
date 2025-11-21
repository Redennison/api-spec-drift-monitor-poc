use crate::error::ValidationError;
use crate::validators::{ParametersValidator, RequestBodyValidator, ResponseValidator};
use matchit::Router;
use std::collections::HashMap;
use std::str::FromStr;

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
            Self::TRACE => "TRACE",
        }
    }
}

impl FromStr for HttpMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "PATCH" => Ok(Self::PATCH),
            "HEAD" => Ok(Self::HEAD),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            _ => Err(()),
        }
    }
}

/// Validator for a single API operation (path + method combination)
pub struct OperationValidator {
    pub request_body: Option<RequestBodyValidator>,
    pub responses: ResponseValidator,
    pub parameters: ParametersValidator,
}

impl OperationValidator {
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
pub struct ApiValidator {
    router: Router<OperationMap>,
}

impl ApiValidator {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
        }
    }

    /// Adds all operations for a path at once
    pub fn add_path_operations(
        &mut self,
        path: &str,
        operations: HashMap<HttpMethod, OperationValidator>,
    ) -> Result<(), ValidationError> {
        self.router.insert(path, operations).map_err(|e| {
            ValidationError::SchemaCompilationError(format!(
                "Failed to add route '{}': {}",
                path, e
            ))
        })
    }

    /// Finds the operation validator for a given path and method
    pub fn find_operation<'a>(
        &'a self,
        path: &'a str,
        method: HttpMethod,
    ) -> Result<(&'a OperationValidator, matchit::Params<'a, 'a>), ValidationError> {
        let matched = self.router.at(path).map_err(|_| {
            ValidationError::ValidationFailed(format!("No route found for path: {}", path))
        })?;

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
