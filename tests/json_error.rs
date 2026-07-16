#![cfg(feature = "schemars")]

use axum::{http::StatusCode, response::IntoResponse};
use axum_thiserror::ErrorStatus;
use schemars::JsonSchema;
use serde::Serialize;
use thiserror::Error;

fn as_string<T: std::fmt::Display, S: serde::Serializer>(v: &T, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(v)
}

#[derive(Error, Debug, Serialize, JsonSchema, ErrorStatus)]
pub enum UserCreateError {
    #[error("io error: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    Io(
        #[from]
        #[serde(serialize_with = "as_string")]
        #[schemars(with = "String")]
        std::io::Error,
    ),
    #[error("Invalid email {email}")]
    #[status(StatusCode::UNAUTHORIZED)]
    InvalidEmail { email: String },
    #[error("User {0} already exists with email {1}")]
    #[status(StatusCode::CONFLICT)]
    UserAlreadyExists(String, String),
}

#[tokio::test]
async fn json_named_fields() {
    let error = UserCreateError::InvalidEmail {
        email: "user01".to_string(),
    };
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], br#"{"InvalidEmail":{"email":"user01"}}"#);
}

#[tokio::test]
async fn json_unnamed_fields() {
    let error = UserCreateError::UserAlreadyExists("user01".to_string(), "a@b.fr".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], br#"{"UserAlreadyExists":["user01","a@b.fr"]}"#);
}

#[tokio::test]
async fn json_wrapped_error() {
    let error: UserCreateError =
        std::io::Error::new(std::io::ErrorKind::Other, "disk on fire").into();
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], br#"{"Io":"disk on fire"}"#);
}
