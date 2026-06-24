//! Proves the integration-test target builds and the error contract holds.
//! Feature-level integration tests live beside their tasks in this folder.

use app_lib::error::AppError;

#[test]
fn app_error_serializes_to_kind_and_message() {
    let err = AppError::NotFound("problem 'x' not found".into());
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(
        json,
        serde_json::json!({ "kind": "not-found", "message": "problem 'x' not found" })
    );
}

#[test]
fn io_errors_convert_and_serialize() {
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
    let err = AppError::from(io);
    let json = serde_json::to_value(&err).unwrap();
    assert_eq!(json["kind"], "io");
    assert_eq!(json["message"], "io error: denied");
}
