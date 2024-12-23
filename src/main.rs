use axum::{
    routing::{get, get_service},
    Router,
    response::{Html, IntoResponse},
    http::StatusCode,
};
use handlebars::Handlebars;
use serde_derive::Serialize;
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[derive(Serialize)]
struct IndexContext {
    message: String,
    title: String,
}

#[derive(Serialize)]
struct ErrorContext {
    message: String,
    title: String,
}

async fn index(hb: Arc<Handlebars<'_>>) -> Html<String> {
    let context = IndexContext {
        message: "Hello from Handlebars in Axum!".to_string(),
        title: "Home".to_string(),
    };
   
    let body = hb.render("index", &context).unwrap_or_else(|err| {
        format!("Error rendering template: {}", err)
    });
    Html(body)
}

async fn not_found(hb: Arc<Handlebars<'_>>) -> impl IntoResponse {
    let context = ErrorContext {
        message: "Página no encontrada".to_string(),
        title: "Error 404".to_string(),
    };
    
    let body = hb.render("error/404", &context).unwrap_or_else(|err| {
        format!("Error rendering 404 template: {}", err)
    });
    (StatusCode::NOT_FOUND, Html(body))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .init();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("index", "templates/index.hbs")
        .expect("Error registering index template");
    handlebars
        .register_template_file("navbar", "./templates/layouts/navbar.hbs")
        .expect("Error registering navbar template");
    handlebars
        .register_template_file("footer", "./templates/layouts/footer.hbs")
        .expect("Error registering footer template");
    handlebars
        .register_template_file("error/404", "templates/error/404.hbs")
        .expect("Error registering 404 template");
    handlebars
        .register_template_file("layouts/base", "templates/layouts/base.hbs")
        .expect("Error registering base layout");
    
    let handlebars = Arc::new(handlebars);

    let app = Router::new()
        .route("/", get({
            let hb = handlebars.clone();
            move || index(hb)
        }))
        .nest_service("/static", ServeDir::new("static"))
        .fallback({
            let hb = handlebars.clone();
            move || not_found(hb)
        })
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Servidor corriendo en http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Archivos estáticos disponibles en http://{}/static", addr);

    println!("Verificando archivos:");
        for file in ["favicon.ico", "reaching_sky.avif", "Yuragi.png"] {
            let path = std::path::Path::new("static").join(file);
            println!("{}: {}", file, path.exists());
        }
    axum::serve(listener, app).await.unwrap();
}