use crate::AppState;
use axum::{
    async_trait,
    body::Body,
    extract::{FromRequestParts, State},
    http::{
        request::Parts,
        StatusCode,
        header, HeaderValue
    },
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::headers::{Cookie, HeaderMapExt};
use bcrypt::DEFAULT_COST;
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

pub struct AuthUser {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {

        // Extraer cookies
        let cookies = parts
            .headers
            .typed_get::<Cookie>()
            .ok_or(AuthError::InvalidToken)?;

        // Obtener el username desde las cookies
        let username = cookies
            .get("session_user")
            .ok_or(AuthError::InvalidToken)?;

        Ok(AuthUser {
            username: username.to_string(),
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid session"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

pub async fn login(
    _state: State<AppState>,
    Json(credentials): Json<LoginCredentials>,
) -> impl IntoResponse {
    // Credenciales predefinidas (en duro)
    let hardcoded_username = "testuser";
    let hardcoded_password_hash = bcrypt::hash("password123", DEFAULT_COST).unwrap();

    // Validar que se hayan enviado las credenciales
    if credentials.username.is_empty() || credentials.password.is_empty() {
        return AuthError::MissingCredentials.into_response();
    }

    // Validar username y contraseña
    if credentials.username == hardcoded_username 
        && bcrypt::verify(&credentials.password, &hardcoded_password_hash).unwrap_or(false) {
        
        // Crear cookie de sesión
        let cookie_value = format!(
            "session_user={}; HttpOnly; Max-Age={}; Path=/",
            credentials.username,
            Duration::hours(24).whole_seconds()
        );

        let mut response = Json(json!({
            "message": "Login successful",
        }))
        .into_response();

        response.headers_mut().insert(
            header::SET_COOKIE,
            HeaderValue::from_str(&cookie_value).unwrap(),
        );

        response
    } else {
        AuthError::WrongCredentials.into_response()
    }
}

pub async fn protected(auth_user: AuthUser) -> Response<Body> {
    Json(json!({
        "message": format!("Hello, {}!", auth_user.username),
        "status": "authenticated"
    }))
    .into_response()
}

pub async fn logout() -> impl IntoResponse {

    let cookie_value = "session_user=; HttpOnly; Max-Age=0; Path=/";

    let mut response = Json(json!({
        "message": "Logged out successfully"
    }))
    .into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(cookie_value).unwrap(),
    );

    response
}