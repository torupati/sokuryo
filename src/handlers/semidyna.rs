use axum::{Json, extract::State, http::StatusCode};
use crate::app_state::AppState;
use crate::models::{SemiDynaCorrectionRequest, SemiDynaCorrectionResponse};
use crate::semidyna::{semidynamic_correction, SemiDynaParams};

#[utoipa::path(
    post,
    path = "/api/semidyna/correct",
    request_body = SemiDynaCorrectionRequest,
    responses(
        (status = 200, description = "セミダイナミック補正の計算結果", body = SemiDynaCorrectionResponse),
        (status = 404, description = "指定した年月のパラメータファイルが未ロード"),
        (status = 422, description = "対象点のメッシュコードがパラメータに存在しない"),
        (status = 503, description = "パラメータ未ロード（SURVEYCALC_JP_SEMIDYNAPARAM_FILE_PATHS 未設定）"),
    ),
    tag = "Sokuryo API"
)]
pub async fn semidyna_correct(
    State(state): State<AppState>,
    Json(req): Json<SemiDynaCorrectionRequest>,
) -> Result<Json<SemiDynaCorrectionResponse>, StatusCode> {
    let params = state.semidyna.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let param_name = SemiDynaParams::param_name_for_date(req.year, req.month);
    let param_set = params.params.get(&param_name).ok_or(StatusCode::NOT_FOUND)?;
    let (d_b, d_l, d_h) =
        semidynamic_correction(req.latitude, req.longitude, param_set)
            .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    // forward: 元期→今期, !forward: 今期→元期
    let sign = if req.forward { 1.0_f64 } else { -1.0_f64 };
    let lat = req.latitude + sign * d_b / 3600.0;
    let lon = req.longitude + sign * d_l / 3600.0;
    let hgt = req.height + sign * d_h;

    Ok(Json(SemiDynaCorrectionResponse {
        latitude: lat,
        longitude: lon,
        height: hgt,
        param_name,
    }))
}
