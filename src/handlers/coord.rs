use axum::{Json, http::StatusCode};
use proj4rs::{Proj, transform::transform};
use crate::models::{LatlonToJapanPlaneRequest, JapanPlaneCoordResponse};

// JGD2011 geographic (equivalent to EPSG:6668)
const FROM_PROJ: &str = "+proj=longlat +ellps=GRS80 +no_defs";

// Japan Plane Rectangular Coordinate System zone to proj4 string (EPSG:6669–6687)
fn zone_to_proj(zone: u8) -> Option<(&'static str, u32)> {
    // lat_0 and lon_0 per zone as defined in JGD2011
    let params: &[(&str, u32)] = &[
        ("+proj=tmerc +lat_0=33 +lon_0=129.5              +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6669),
        ("+proj=tmerc +lat_0=33 +lon_0=131                +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6670),
        ("+proj=tmerc +lat_0=36 +lon_0=132.1666666666667  +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6671),
        ("+proj=tmerc +lat_0=33 +lon_0=133.5              +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6672),
        ("+proj=tmerc +lat_0=36 +lon_0=134.3333333333333  +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6673),
        ("+proj=tmerc +lat_0=36 +lon_0=136                +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6674),
        ("+proj=tmerc +lat_0=36 +lon_0=137.1666666666667  +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6675),
        ("+proj=tmerc +lat_0=36 +lon_0=138.5              +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6676),
        ("+proj=tmerc +lat_0=36 +lon_0=139.8333333333333  +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6677),
        ("+proj=tmerc +lat_0=40 +lon_0=140.8333333333333  +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6678),
        ("+proj=tmerc +lat_0=44 +lon_0=140.25             +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6679),
        ("+proj=tmerc +lat_0=44 +lon_0=142.25             +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6680),
        ("+proj=tmerc +lat_0=44 +lon_0=144.25             +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6681),
        ("+proj=tmerc +lat_0=26 +lon_0=142               +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6682),
        ("+proj=tmerc +lat_0=26 +lon_0=127.5              +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6683),
        ("+proj=tmerc +lat_0=26 +lon_0=124               +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6684),
        ("+proj=tmerc +lat_0=26 +lon_0=131               +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6685),
        ("+proj=tmerc +lat_0=20 +lon_0=136               +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6686),
        ("+proj=tmerc +lat_0=26 +lon_0=154               +k=0.9999 +x_0=0 +y_0=0 +ellps=GRS80 +units=m +no_defs", 6687),
    ];
    if zone >= 1 && zone <= 19 {
        Some(params[(zone - 1) as usize])
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
    let (to_proj, epsg) = zone_to_proj(req.zone).ok_or(StatusCode::BAD_REQUEST)?;

    let from = Proj::from_proj_string(FROM_PROJ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let to   = Proj::from_proj_string(to_proj).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut point = (req.longitude.to_radians(), req.latitude.to_radians(), 0.0);
    transform(&from, &to, &mut point).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(JapanPlaneCoordResponse { x: point.1, y: point.0, epsg }))
}