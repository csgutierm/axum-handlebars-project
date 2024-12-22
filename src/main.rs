use axum::{
    routing::get,
    Router,
    response::Html
};
use handlebars::Handlebars;
use serde_derive::Serialize;
use std::sync::Arc;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

#[derive(Serialize)]
struct IndexContext {
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

#[tokio::main]
async fn main() {
    // Inicializar Handlebars y registrar las plantillas
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("index", "templates/index.hbs")
        .expect("Error registering template");
    handlebars
        .register_template_file("layouts/base", "templates/layouts/base.hbs")
        .expect("Error registering base layout");
    let handlebars = Arc::new(handlebars);

    // Crear el router principal
    let app = Router::new()
        .route("/", get({
            let hb = handlebars.clone();
            move || index(hb)
        }))
        // Agregar servicio de archivos estáticos
        .nest_service("/static", ServeDir::new("static"));

    // Configurar la dirección del servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Servidor corriendo en http://{}", addr);

    // Iniciar el servidor
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Archivos estáticos disponibles en http://{}/static", addr);
    axum::serve(listener, app).await.unwrap();
}