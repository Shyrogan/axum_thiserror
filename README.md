# axum_thiserror

`axum_thiserror` is a library that offers a procedural macro to allow `thiserror` error types to be used as `axum` responses.

## Usage

Add the library to your current project using Cargo:
```bash
cargo add axum_thiserror
```

Then you can create a basic `thiserror` error:
```rust
#[derive(Error, Debug)]
pub enum UserCreateError {
  #[error("User {0} already exists")]
  UserAlreadyExists(String),
}
```

Now you can use `axum_thiserror` to implement `IntoResponse` on your error. By
default (the `schemars` feature), the response is the variant's status code with
`axum::Json(self)` as the body, so the enum must also derive `serde::Serialize`
and `schemars::JsonSchema`:

```toml
[dependencies]
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
}
```

responds with status `409` and body:

```json
{ "UserAlreadyExists": ["user01"] }
```

The body is whatever serde produces for your enum — shape it with regular serde
attributes. Deriving `schemars::JsonSchema` gives a schema matching the response
body exactly, handy for OpenAPI tooling such as `aide` or `utoipa`. A missing
`Serialize` or `JsonSchema` implementation is a deliberate compile error rather
than a silent fallback to plain text.

## Plain-text responses

Disable default features to simply implement `IntoResponse` with the variant's
status code and the plain-text `#[error(...)]` message as the body — no extra
derives needed:

```toml
[dependencies]
axum_thiserror = { version = "0.1", default-features = false }
```

## Wrapping foreign errors (`sqlx::Error`, `std::io::Error`, ...)

Wrapped error types usually implement neither `Serialize` nor `JsonSchema`. Serialize
them as their message string with `#[serde(serialize_with)]` and `#[schemars(with)]`:

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

which responds with a body like:

```json
{ "Database": "error returned from database: relation \"users\" does not exist" }
```

## License

This project is licensed under the [MIT License](LICENSE).
