use jsonschema::error::ValidationErrorKind;

#[derive(Debug, Clone)]
pub enum DriftType {
    ParameterTypeMismatch,
    RequestBodyTypeMismatch,
    ResponseBodyTypeMismatch,
    ParameterMissingRequired,
    RequestBodyMissingRequired,
    ResponseBodyMissingRequired,
    ParameterEnumViolation,
    RequestBodyEnumViolation,
    ResponseBodyEnumViolation,
    ParameterOneOfNoMatch,
    RequestBodyOneOfNoMatch,
    ResponseBodyOneOfNoMatch,
    ParameterAnyOfNoMatch,
    RequestBodyAnyOfNoMatch,
    ResponseBodyAnyOfNoMatch,
}

impl DriftType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ParameterTypeMismatch => "PARAMETER_TYPE_MISMATCH",
            Self::RequestBodyTypeMismatch => "REQUEST_BODY_TYPE_MISMATCH",
            Self::ResponseBodyTypeMismatch => "RESPONSE_BODY_TYPE_MISMATCH",
            Self::ParameterMissingRequired => "PARAMETER_MISSING_REQUIRED",
            Self::RequestBodyMissingRequired => "REQUEST_BODY_MISSING_REQUIRED",
            Self::ResponseBodyMissingRequired => "RESPONSE_BODY_MISSING_REQUIRED",
            Self::ParameterEnumViolation => "PARAMETER_ENUM_VIOLATION",
            Self::RequestBodyEnumViolation => "REQUEST_BODY_ENUM_VIOLATION",
            Self::ResponseBodyEnumViolation => "RESPONSE_BODY_ENUM_VIOLATION",
            Self::ParameterOneOfNoMatch => "PARAMETER_ONEOF_NO_MATCH",
            Self::RequestBodyOneOfNoMatch => "REQUEST_BODY_ONEOF_NO_MATCH",
            Self::ResponseBodyOneOfNoMatch => "RESPONSE_BODY_ONEOF_NO_MATCH",
            Self::ParameterAnyOfNoMatch => "PARAMETER_ANYOF_NO_MATCH",
            Self::RequestBodyAnyOfNoMatch => "REQUEST_BODY_ANYOF_NO_MATCH",
            Self::ResponseBodyAnyOfNoMatch => "RESPONSE_BODY_ANYOF_NO_MATCH",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ValidationContext {
    Parameter,
    RequestBody,
    ResponseBody,
}

/// Maps ValidationErrorKind to DriftType based on context
pub fn map_to_drift_type(kind: &ValidationErrorKind, context: ValidationContext) -> Option<DriftType> {
    use ValidationContext::*;
    
    match kind {
        ValidationErrorKind::Type { .. } => Some(match context {
            Parameter => DriftType::ParameterTypeMismatch,
            RequestBody => DriftType::RequestBodyTypeMismatch,
            ResponseBody => DriftType::ResponseBodyTypeMismatch,
        }),
        ValidationErrorKind::Required { .. } => Some(match context {
            Parameter => DriftType::ParameterMissingRequired,
            RequestBody => DriftType::RequestBodyMissingRequired,
            ResponseBody => DriftType::ResponseBodyMissingRequired,
        }),
        ValidationErrorKind::Enum { .. } => Some(match context {
            Parameter => DriftType::ParameterEnumViolation,
            RequestBody => DriftType::RequestBodyEnumViolation,
            ResponseBody => DriftType::ResponseBodyEnumViolation,
        }),
        ValidationErrorKind::OneOfNotValid { .. } => Some(match context {
            Parameter => DriftType::ParameterOneOfNoMatch,
            RequestBody => DriftType::RequestBodyOneOfNoMatch,
            ResponseBody => DriftType::ResponseBodyOneOfNoMatch,
        }),
        ValidationErrorKind::AnyOf { .. } => Some(match context {
            Parameter => DriftType::ParameterAnyOfNoMatch,
            RequestBody => DriftType::RequestBodyAnyOfNoMatch,
            ResponseBody => DriftType::ResponseBodyAnyOfNoMatch,
        }),
        _ => None,
    }
}

