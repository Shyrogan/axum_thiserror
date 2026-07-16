use axum::{http::StatusCode, response::IntoResponse};
use axum_thiserror::ErrorStatus;
use thiserror::Error;

#[derive(Error, Debug, ErrorStatus)]
#[cfg_attr(feature = "schemars", derive(serde::Serialize, schemars::JsonSchema))]
pub enum UserCreateError {
    #[error("Invalid email {email}")]
    #[status(StatusCode::UNAUTHORIZED)]
    InvalidEmail { email: String },
    #[error("User {0} already exists with email")]
    #[status(StatusCode::CONFLICT)]
    UserAlreadyExists(String, String),
}

#[test]
fn basic_error() {
    let error = UserCreateError::InvalidEmail {
        email: "user01".to_string(),
    };
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let error = UserCreateError::UserAlreadyExists("user01".to_string(), "email".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[cfg(not(feature = "schemars"))]
#[tokio::test]
async fn text_body() {
    let error = UserCreateError::InvalidEmail {
        email: "user01".to_string(),
    };
    let response = error.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"Invalid email user01");
}
