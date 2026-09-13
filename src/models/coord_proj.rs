use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LatlonToJapanPlaneRequest {
    /// latitude (degrees)
    pub latitude: f64,
    /// longitude (degrees)
    pub longitude: f64,
    /// Japan Plane Rectangular Coordinate System zone number (1-19)
    pub zone: u8,
}

#[derive(Serialize, ToSchema)]
pub struct JapanPlaneCoordResponse {
    /// Plane Rectangular Coordinate X (north-south direction, m)
    pub x: f64,
    /// Plane Rectangular Coordinate Y (east-west direction, m)
    pub y: f64,
    /// EPSG code of the used zone number
    pub epsg: u32,
}