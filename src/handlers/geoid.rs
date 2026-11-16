use axum::{Json, extract::State, http::StatusCode};
use crate::app_state::AppState;
use crate::models::{GeoidHeightRequest, GeoidHeightResponse, ElevationRequest, ElevationResponse};

#[utoipa::path(
    post,
    path = "/api/geoid/height",
    request_body = GeoidHeightRequest,
    responses(
        (status = 200, description = "ジオイド高の計算結果", body = GeoidHeightResponse),
        (status = 422, description = "範囲外または欠損データ"),
        (status = 503, description = "ジオイドモデル未ロード（GNSSPOS_JP_GSI_GEOID 未設定）"),
    ),
    tag = "Sokuryo API"
)]
pub async fn geoid_height(
    State(state): State<AppState>,
    Json(req): Json<GeoidHeightRequest>,
) -> Result<Json<GeoidHeightResponse>, StatusCode> {
    let model = state.geoid.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let h = model
        .geoid_height(req.latitude, req.longitude)
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    Ok(Json(GeoidHeightResponse { geoid_height: h }))
}

#[utoipa::path(
    post,
    path = "/api/geoid/elevation",
    request_body = ElevationRequest,
    responses(
        (status = 200, description = "標高の計算結果", body = ElevationResponse),
        (status = 422, description = "範囲外または欠損データ"),
        (status = 503, description = "ジオイドモデル未ロード（GNSSPOS_JP_GSI_GEOID 未設定）"),
    ),
    tag = "Sokuryo API"
)]
pub async fn geoid_elevation(
    State(state): State<AppState>,
    Json(req): Json<ElevationRequest>,
) -> Result<Json<ElevationResponse>, StatusCode> {
    let model = state.geoid.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let elev = model
        .elevation(req.latitude, req.longitude, req.altitude)
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;
    Ok(Json(ElevationResponse { elevation: elev }))
}
