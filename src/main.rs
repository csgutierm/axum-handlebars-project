use std::{net::SocketAddr, sync::Arc};
use axum_handlebars_project::{AppState, AppError, setup_handlebars, create_router, check_static_files};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .init();

    let handlebars = setup_handlebars()?;
    let state = AppState {
        handlebars: Arc::new(handlebars),
    };

    let app = create_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Servidor corriendo en http://{}", addr);
    println!("Archivos estáticos disponibles en http://{}/static", addr);

    check_static_files();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::ServerError(e.to_string()))?;

    Ok(())
}