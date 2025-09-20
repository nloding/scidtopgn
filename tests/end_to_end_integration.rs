// End-to-End Integration Tests
//
// These tests validate the complete SCID to PGN conversion pipeline,
// testing all components working together as specified in Phase 4 Step 4.2
// of the Comprehensive Remediation Plan.

mod integration;

// Re-export the integration tests
pub use integration::*;
