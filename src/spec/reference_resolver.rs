use crate::error::ValidationError;
use openapiv3::{Components, OpenAPI, ReferenceOr};

/// Resolves OpenAPI structure-level $ref to actual component definitions
///
/// This trait handles references to OpenAPI components like:
/// - `$ref: "#/components/parameters/PageLimit"`
/// - `$ref: "#/components/requestBodies/CreateUser"`
/// - `$ref: "#/components/responses/ErrorResponse"`
///
/// **Important distinction:**
/// - This resolves **OpenAPI structure references** (RequestBody, Response, Parameter objects)
/// - The `jsonschema` Registry handles **schema-level references** (inside JSON schemas)
///
/// Example:
/// ```yaml
/// requestBody:
///   $ref: "#/components/requestBodies/CreateUser"  # ← We resolve this
///
/// components:
///   requestBodies:
///     CreateUser:
///       content:
///         application/json:
///           schema:
///             $ref: "#/components/schemas/User"    # ← jsonschema Registry resolves this
/// ```
pub trait ResolveReference<T> {
    fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a T, ValidationError>;
}

/// Internal helper that implements the resolution logic
fn resolve_logic<'a, T, F>(
    ref_or: &'a ReferenceOr<T>,
    spec: &'a OpenAPI,
    prefix: &str,
    selector: F,
) -> Result<&'a T, ValidationError>
where
    F: Fn(&'a Components) -> Option<&'a indexmap::IndexMap<String, ReferenceOr<T>>>,
{
    match ref_or {
        ReferenceOr::Item(item) => Ok(item),
        ReferenceOr::Reference { reference } => {
            if !reference.starts_with(prefix) {
                return Err(ValidationError::SchemaCompilationError(format!(
                    "Invalid reference: {}. Expected prefix: {}",
                    reference, prefix
                )));
            }
            let name = &reference[prefix.len()..];

            spec.components
                .as_ref()
                .and_then(selector)
                .and_then(|map| map.get(name))
                .and_then(|r| r.as_item())
                .ok_or_else(|| {
                    ValidationError::SchemaCompilationError(format!(
                        "Reference not found: {}",
                        reference
                    ))
                })
        }
    }
}

impl ResolveReference<openapiv3::Parameter> for ReferenceOr<openapiv3::Parameter> {
    fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a openapiv3::Parameter, ValidationError> {
        resolve_logic(self, spec, "#/components/parameters/", |c| {
            Some(&c.parameters)
        })
    }
}

impl ResolveReference<openapiv3::RequestBody> for ReferenceOr<openapiv3::RequestBody> {
    fn resolve<'a>(
        &'a self,
        spec: &'a OpenAPI,
    ) -> Result<&'a openapiv3::RequestBody, ValidationError> {
        resolve_logic(self, spec, "#/components/requestBodies/", |c| {
            Some(&c.request_bodies)
        })
    }
}

impl ResolveReference<openapiv3::Response> for ReferenceOr<openapiv3::Response> {
    fn resolve<'a>(
        &'a self,
        spec: &'a OpenAPI,
    ) -> Result<&'a openapiv3::Response, ValidationError> {
        resolve_logic(self, spec, "#/components/responses/", |c| {
            Some(&c.responses)
        })
    }
}

