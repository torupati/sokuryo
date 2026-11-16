use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct GeoidHeightRequest {
    /// 緯度（度）
    pub latitude: f64,
    /// 経度（度）
    pub longitude: f64,
}

#[derive(Serialize, ToSchema)]
pub struct GeoidHeightResponse {
    /// ジオイド高（m）
    pub geoid_height: f64,
}

#[derive(Deserialize, ToSchema)]
pub struct ElevationRequest {
    /// 緯度（度）
    pub latitude: f64,
    /// 経度（度）
    pub longitude: f64,
    /// 楕円体高（m）
    pub altitude: f64,
}

#[derive(Serialize, ToSchema)]
pub struct ElevationResponse {
    /// 標高 = 楕円体高 − ジオイド高（m）
    pub elevation: f64,
}
