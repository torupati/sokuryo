use std::sync::Arc;
use crate::geoid::JpGeoidModel;
use crate::semidyna::SemiDynaParams;

/// Axum の共有状態。起動時に環境変数からデータファイルをロードして保持する。
#[derive(Clone)]
pub struct AppState {
    /// GNSSPOS_JP_GSI_GEOID 環境変数でファイルパスを指定
    pub geoid: Option<Arc<JpGeoidModel>>,
    /// SURVEYCALC_JP_SEMIDYNAPARAM_FILE_PATHS 環境変数で JSON 設定ファイルパスを指定
    pub semidyna: Option<Arc<SemiDynaParams>>,
}
