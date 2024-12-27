pub mod auth;

use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
    http::StatusCode,
    extract::State,
};
use handlebars::Handlebars;
use serde_derive::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub use auth::{login, protected, AuthUser};

#[derive(Clone)]
pub struct AppState {
    pub handlebars: Arc<Handlebars<'static>>,
}

#[derive(Serialize)]
pub struct IndexContext {
    message: String,
    title: String,
}

#[derive(Serialize)]
pub struct ErrorContext {
    message: String,
    title: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Error al renderizar la plantilla: {0}")]
    TemplateError(#[from] handlebars::RenderError),
    #[error("Error al registrar la plantilla: {0}")]
    TemplateRegistrationError(#[from] handlebars::TemplateError),
    #[error("Error del servidor: {0}")]
    ServerError(String),
}

// Implementar IntoResponse para AppError
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let body = format!("Error: {}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, Html(body)).into_response()
    }
}

pub fn setup_handlebars() -> Result<Handlebars<'static>, AppError> {
    let mut handlebars = Handlebars::new();
    
    let templates = vec![
        ("index", "templates/index.hbs"),
        ("navbar", "templates/layouts/navbar.hbs"),
        ("footer", "templates/layouts/footer.hbs"),
        ("error/404", "templates/error/404.hbs"),
        ("layouts/base", "templates/layouts/base.hbs"),
        ("login", "templates/login.hbs"),
    ];

    for (name, path) in templates {
        handlebars.register_template_file(name, path)
            .map_err(AppError::TemplateRegistrationError)?;
    }

    Ok(handlebars)
}

// Handler modificado para retornar impl IntoResponse
pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let context = IndexContext {
        message: "Hello from Handlebars in Axum!".to_string(),
        title: "Home".to_string(),
    };

    match state.handlebars.render("index", &context) {
        Ok(body) => Html(body).into_response(),
        Err(err) => AppError::TemplateError(err).into_response(),
    }
}

pub async fn not_found(State(state): State<AppState>) -> impl IntoResponse {
    let context = ErrorContext {
        message: "Página no encontrada".to_string(),
        title: "Error 404".to_string(),
    };

    match state.handlebars.render("error/404", &context) {
        Ok(body) => (StatusCode::NOT_FOUND, Html(body)).into_response(),
        Err(err) => AppError::TemplateError(err).into_response(),
    }
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/login", post(login))
        .route("/login", get(login_page))
        .route("/protected", get(protected))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn check_static_files() {
    println!("Verificando archivos:");
    for file in ["favicon.ico", "reaching_sky.avif", "Yuragi.png"] {
        let path = std::path::Path::new("static").join(file);
        println!("{}: {}", file, path.exists());
    }
}

use serde_json::json;

pub async fn login_page(State(state): State<AppState>) -> impl IntoResponse {
    let context = json!({
        "title": "Login"
    });

    match state.handlebars.render("login", &context) {
        Ok(body) => Html(body).into_response(),
        Err(err) => AppError::TemplateError(err).into_response(),
    }
}