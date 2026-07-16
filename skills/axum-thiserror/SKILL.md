---
name: axum-thiserror
description: Use when defining error types for an axum web service with thiserror — #[derive(ErrorStatus)] maps each enum variant to an HTTP status code and implements IntoResponse, with a JSON body by default (serde/schemars) or plain text without default features.
---

# axum_thiserror

`#[derive(ErrorStatus)]` implements `axum::response::IntoResponse` for a `thiserror`
enum, so handlers can return `Result<T, MyError>` directly. Every variant needs
`#[status(...)]` with a `StatusCode` path or a `u16` literal; enums only.

- **Default** (`schemars` feature): responds with the status code and `axum::Json(self)`.
  The enum must also derive `serde::Serialize` and `schemars::JsonSchema` (missing
  impls are a compile error); the `JsonSchema` matches the body exactly for OpenAPI
  tooling (`aide`, `utoipa`).
- **`default-features = false`**: just implements `IntoResponse` with the status code
  and the `#[error(...)]` message as a plain-text body — no extra derives needed.

## Example

```toml
axum_thiserror = "0.1"
serde = { version = "1", features = ["derive"] }
schemars = "1"
```

```rust
#[derive(Error, Debug, Serialize, JsonSchema, ErrorStatus)]
pub enum UserCreateError {
    #[error("User {0} already exists")]
    #[status(StatusCode::CONFLICT)]
    UserAlreadyExists(String),
    #[error("Invalid email {email}")]
    #[status(422)]
    InvalidEmail { email: String },
}
```

`UserAlreadyExists("user01")` responds `409` with body `{"UserAlreadyExists":["user01"]}`
(shape it with regular serde attributes).

## Example: wrapping foreign errors

Wrapped errors (`sqlx::Error`, `std::io::Error`, ...) implement neither `Serialize`
nor `JsonSchema`, and `#[serde(skip)]` does not work on newtype variants. Serialize
them as their message string:

```rust
fn as_string<T: std::fmt::Display, S: serde::Serializer>(v: &T, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(v)
}

#[derive(Error, Debug, Serialize, JsonSchema, ErrorStatus)]
pub enum ApiError {
    #[error("database error: {0}")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    Database(
        #[from]
        #[serde(serialize_with = "as_string")]
        #[schemars(with = "String")]
        sqlx::Error,
    ),
}
```

Responds `{"Database":"error returned from database: ..."}`. Whatever serde serializes
is sent to the client — keep sensitive fields out of the body.
