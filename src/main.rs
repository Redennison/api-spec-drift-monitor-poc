use api_spec_drift_monitor_poc::{build_api_validator, load_openapi_spec};
use std::path::Path;

fn main() {
    println!("=== API Spec Drift Monitor ===\n");

    // Load OpenAPI specification
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

    // Build API validator from the spec
    let _api_validator = match build_api_validator(&spec) {
        Ok(validator) => {
            println!("✓ API Validator built successfully\n");
            validator
        }
        Err(e) => {
            eprintln!("✗ Failed to build validator: {}", e);
            return;
        }
    };

    println!("Ready to validate API traffic.");
}
