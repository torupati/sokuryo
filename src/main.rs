mod models;
mod handlers;
mod geoid;
mod semidyna;
mod app_state;
mod stereo;

use std::collections::HashMap;
use std::sync::Arc;
use axum::{Router, routing::post};
use handlers::coord::latlon_to_plane;
use handlers::geoid::{geoid_elevation, geoid_height};
use handlers::semidyna::semidyna_correct;
use app_state::AppState;
use geoid::JpGeoidModel;
use semidyna::{load_par_file, SemiDynaParams};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::coord::latlon_to_plane,
        handlers::geoid::geoid_height,
        handlers::geoid::geoid_elevation,
        handlers::semidyna::semidyna_correct,
    ),
    components(schemas(
        models::LatlonToJapanPlaneRequest,
        models::JapanPlaneCoordResponse,
        models::GeoidHeightRequest,
        models::GeoidHeightResponse,
        models::ElevationRequest,
        models::ElevationResponse,
        models::SemiDynaCorrectionRequest,
        models::SemiDynaCorrectionResponse,
    )),
    tags((name = "Sokuryo API", description = "Survey Calculation Web API"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // GNSSPOS_JP_GSI_GEOID 環境変数でジオイドモデルファイルのパスを指定
    let geoid_model = std::env::var("GNSSPOS_JP_GSI_GEOID")
        .ok()
        .and_then(|path| {
            match JpGeoidModel::load(std::path::Path::new(&path)) {
                Ok(m) => Some(m),
                Err(e) => {
                    tracing::warn!("Failed to load geoid model from '{}': {}", path, e);
                    None
                }
            }
        })
        .map(Arc::new);

    // SURVEYCALC_JP_SEMIDYNAPARAM_FILE_PATHS 環境変数で {"SemiDyna20xx.par": "/path/to/file"} の JSON ファイルパスを指定
    let semidyna_params = std::env::var("SURVEYCALC_JP_SEMIDYNAPARAM_FILE_PATHS")
        .ok()
        .and_then(|json_path| {
            let f = match std::fs::File::open(&json_path) {
                Ok(f) => f,
                Err(e) => {
                    tracing::warn!("Failed to open semidyna config '{}': {}", json_path, e);
                    return None;
                }
            };
            let file_map: HashMap<String, String> = match serde_json::from_reader(f) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to parse semidyna config: {}", e);
                    return None;
                }
            };
            let mut all: HashMap<String, _> = HashMap::new();
            for (name, path) in &file_map {
                match load_par_file(std::path::Path::new(path)) {
                    Ok(p) => { all.insert(name.clone(), p); }
                    Err(e) => tracing::warn!("Failed to load semidyna '{}': {}", path, e),
                }
            }
            Some(Arc::new(SemiDynaParams { params: all }))
        });

    let state = AppState { geoid: geoid_model, semidyna: semidyna_params };

    let app = Router::new()
        .route("/api/coord/latlon-to-plane", post(latlon_to_plane))
        .route("/api/geoid/height", post(geoid_height))
        .route("/api/geoid/elevation", post(geoid_elevation))
        .route("/api/semidyna/correct", post(semidyna_correct))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    tracing::info!("Swagger UI: http://localhost:3000/swagger-ui");
    tracing::info!("Redoc:      http://localhost:3000/redoc");
    axum::serve(listener, app).await.unwrap();
}
