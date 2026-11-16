use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SemiDynaCorrectionRequest {
    /// 緯度（度）
    pub latitude: f64,
    /// 経度（度）
    pub longitude: f64,
    /// 楕円体高（m）
    pub height: f64,
    /// 観測年（西暦）
    pub year: i32,
    /// 観測月（1〜12）
    pub month: u32,
    /// true: 元期→今期（デフォルト）、false: 今期→元期
    #[serde(default = "default_true")]
    pub forward: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, ToSchema)]
pub struct SemiDynaCorrectionResponse {
    /// 補正後の緯度（度）
    pub latitude: f64,
    /// 補正後の経度（度）
    pub longitude: f64,
    /// 補正後の楕円体高（m）
    pub height: f64,
    /// 使用したパラメータファイル名
    pub param_name: String,
}
