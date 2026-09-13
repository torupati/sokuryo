mod models;
mod handlers;

use axum::{Router, routing::post};
use handlers::coord::latlon_to_plane;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(handlers::coord::latlon_to_plane),
    components(schemas(
        models::LatlonToJapanPlaneRequest,
        models::JapanPlaneCoordResponse,
    )),
    tags((name = "Sokuryo API", description = "Survey Calculation Web API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/coord/latlon-to-plane", post(latlon_to_plane))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui");
    tracing::info!("Redoc:      http://localhost:3000/redoc");
    axum::serve(listener, app).await.unwrap();
}
