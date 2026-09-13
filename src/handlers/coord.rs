use axum::{Json, http::StatusCode};
use proj4rs::{Proj, transform::transform};
use crate::models::{LatlonToJapanPlaneRequest, JapanPlaneCoordResponse};

// Japan Plane Rectangular Coordinate System zones to EPSG codes
fn zone_to_epsg(zone: u8) -> Option<u32> {
    if zone >= 1 && zone <= 19 {
        Some(6668 + zone as u32)  // EPSG:6669〜6687
    } else {
        None
    }
}

#[utoipa::path(
    post,
    path = "/api/coord/latlon-to-plane",
    request_body = LatlonToJapanPlaneRequest,
    responses(
        (status = 200, description = "Successful conversion", body = JapanPlaneCoordResponse),
        (status = 400, description = "Invalid parameters / zone out of range"),
    ),
    tag = "Sokuryo API"
)]
pub async fn latlon_to_plane(
    Json(req): Json<LatlonToJapanPlaneRequest>,
) -> Result<Json<JapanPlaneCoordResponse>, StatusCode> {
    let epsg = zone_to_epsg(req.zone).ok_or(StatusCode::BAD_REQUEST)?;

    let from = Proj::from_user_string("EPSG:6668").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let to   = Proj::from_user_string(&format!("EPSG:{epsg}")).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut point = (req.longitude.to_radians(), req.latitude.to_radians(), 0.0);
    transform(&from, &to, &mut point).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(JapanPlaneCoordResponse { x: point.1, y: point.0, epsg }))
}