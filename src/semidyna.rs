use std::collections::HashMap;
use std::path::Path;

/// セミダイナミック補正の1グリッド点データ
pub struct SemiDynaRecord {
    pub d_b: f64,       // 緯度方向補正量（秒）
    pub d_l: f64,       // 経度方向補正量（秒）
    pub d_h: f64,       // 高さ方向補正量（m）
    pub latitude: f64,  // グリッド点の緯度（度）
    pub longitude: f64, // グリッド点の経度（度）
}

/// メッシュコード→パラメータのマップ（1パラメータファイル分）
pub type ParamSet = HashMap<String, SemiDynaRecord>;

/// 複数年分のパラメータを保持する
pub struct SemiDynaParams {
    pub params: HashMap<String, ParamSet>,
}

impl SemiDynaParams {
    /// 年月から使用するパラメータ名を決定する（1〜3月は前年度を使用）
    pub fn param_name_for_date(year: i32, month: u32) -> String {
        let y = if month <= 3 { year - 1 } else { year };
        format!("SemiDyna{}.par", y)
    }
}

/// メッシュコード（8桁文字列）から緯度経度（度）を計算する
pub fn meshcode_to_latlon(code: &str) -> (f64, f64) {
    let first_two: f64 = code[0..2].trim().parse().unwrap_or(0.0);
    let last_two: f64 = code[2..4].trim().parse().unwrap_or(0.0);
    let mut lat = first_two * 2.0 / 3.0;
    let mut lon = last_two + 100.0;

    if code.len() >= 6 {
        let fifth: f64 = code[4..5].parse().unwrap_or(0.0);
        let sixth: f64 = code[5..6].parse().unwrap_or(0.0);
        lat += fifth * 2.0 / 3.0 / 8.0;
        lon += sixth / 8.0;
    }
    if code.len() >= 8 {
        let seventh: f64 = code[6..7].parse().unwrap_or(0.0);
        let eighth: f64 = code[7..8].parse().unwrap_or(0.0);
        lat += seventh * 2.0 / 3.0 / 8.0 / 10.0;
        lon += eighth / 8.0 / 10.0;
    }
    (lat, lon)
}

/// 緯度経度（度）から3次メッシュコード（8桁文字列）を計算する
pub fn latlon_to_meshcode(lat: f64, lon: f64) -> String {
    let p = (lat * 120.0 / 80.0).floor() as i64 * 1_000_000
        + (lon - 100.0).floor() as i64 * 10_000
        + (lat * 120.0 % 80.0 / 10.0).floor() as i64 * 1_000
        + ((lon - 100.0) * 80.0 % 80.0 / 10.0).floor() as i64 * 100
        + (lat * 120.0 % 10.0).floor() as i64 * 10
        + ((lon - 100.0) * 80.0).floor() as i64 % 10;
    format!("{:8}", p)
}

/// セミダイナミック補正のバイリニア補間に使う近傍4点のメッシュコードを返す
/// 戻り値: (p0, p1, p2, p3, p4) - p0: 入力点のコード、p1〜p4: 補間に使う4隅
pub fn meshcode_vicinity(lat: f64, lon: f64) -> (String, String, String, String, String) {
    let p = latlon_to_meshcode(lat, lon);
    let pc: Vec<char> = p.chars().collect();
    let mut p1 = pc.clone();
    let mut p2 = pc.clone();
    let mut p3 = pc.clone();
    let mut p4 = pc.clone();

    // 緯度方向（インデックス6）
    if matches!(pc[6], '0' | '1' | '2' | '3' | '4') {
        p1[6] = '0';
        p2[6] = '0';
        p3[6] = '5';
        p4[6] = '5';
    } else {
        p1[6] = '5';
        p2[6] = '5';
        p3[6] = '0';
        p4[6] = '0';
        // 上位グリッドのインデックスを1つ繰り上げる
        let v1: i32 = p[0..2].trim().parse().unwrap_or(0);
        let v2: i32 = pc[4].to_digit(10).unwrap_or(0) as i32 + 1;
        let (v1, v2) = if v2 == 8 { (v1 + 1, 0i32) } else { (v1, v2) };
        let sv: Vec<char> = format!("{:2}", v1).chars().collect();
        p3[0] = sv[0];
        p3[1] = sv[1];
        p3[4] = char::from_digit(v2 as u32, 10).unwrap_or('0');
        p4[0] = sv[0];
        p4[1] = sv[1];
        p4[4] = char::from_digit(v2 as u32, 10).unwrap_or('0');
    }

    // 経度方向（インデックス7）
    if matches!(pc[7], '0' | '1' | '2' | '3' | '4') {
        p1[7] = '0';
        p3[7] = '0';
        p2[7] = '5';
        p4[7] = '5';
    } else {
        p1[7] = '5';
        p3[7] = '5';
        p2[7] = '0';
        p4[7] = '0';
        // 上位グリッドのインデックスを1つ繰り上げる
        let v1: i32 = p[2..4].trim().parse().unwrap_or(0);
        let v2: i32 = pc[5].to_digit(10).unwrap_or(0) as i32 + 1;
        let (v1, v2) = if v2 == 8 { (v1 + 1, 0i32) } else { (v1, v2) };
        let sv: Vec<char> = format!("{:2}", v1).chars().collect();
        p2[2] = sv[0];
        p2[3] = sv[1];
        p2[5] = char::from_digit(v2 as u32, 10).unwrap_or('0');
        p4[2] = sv[0];
        p4[3] = sv[1];
        p4[5] = char::from_digit(v2 as u32, 10).unwrap_or('0');
    }

    (
        p,
        p1.iter().collect(),
        p2.iter().collect(),
        p3.iter().collect(),
        p4.iter().collect(),
    )
}

fn bilinear(x: f64, y: f64, z1: f64, z2: f64, z3: f64, z4: f64) -> f64 {
    (1.0 - x) * (1.0 - y) * z1 + (1.0 - x) * y * z2 + x * (1.0 - y) * z3 + x * y * z4
}

/// セミダイナミック補正量 (dB, dL, dH) を計算する。
/// 近傍メッシュコードがパラメータに存在しない場合は None。
pub fn semidynamic_correction(lat: f64, lon: f64, params: &ParamSet) -> Option<(f64, f64, f64)> {
    let (_, p1, p2, p3, p4) = meshcode_vicinity(lat, lon);
    let v1 = params.get(p1.trim())?;
    let v2 = params.get(p2.trim())?;
    let v3 = params.get(p3.trim())?;
    let v4 = params.get(p4.trim())?;

    let dlat3 = 2.0_f64 / 3.0 / 8.0 / 10.0;
    let dlon3 = 1.0_f64 / 8.0 / 10.0;
    let x = (lat - v1.latitude) / (dlat3 * 5.0);
    let y = (lon - v1.longitude) / (dlon3 * 5.0);

    Some((
        bilinear(x, y, v1.d_b, v2.d_b, v3.d_b, v4.d_b),
        bilinear(x, y, v1.d_l, v2.d_l, v3.d_l, v4.d_l),
        bilinear(x, y, v1.d_h, v2.d_h, v3.d_h, v4.d_h),
    ))
}

/// セミダイナミック補正パラメータファイル（.par、Shift-JIS）を読み込む。
/// ヘッダー16行はスキップし、データ行はASCIIとして処理する。
pub fn load_par_file(path: &Path) -> std::io::Result<ParamSet> {
    let bytes = std::fs::read(path)?;
    let mut map = HashMap::new();
    let mut line_count = 0usize;

    for raw_line in bytes.split(|&b| b == b'\n') {
        line_count += 1;
        if line_count <= 16 {
            continue; // Shift-JISヘッダーをスキップ
        }
        // Windows改行の \r を除去
        let raw_line = if raw_line.last() == Some(&b'\r') {
            &raw_line[..raw_line.len() - 1]
        } else {
            raw_line
        };
        if raw_line.len() < 38 {
            continue;
        }
        let Ok(line) = std::str::from_utf8(raw_line) else {
            continue;
        };
        let mesh_code = line[0..8].to_string();
        let d_b: f64 = line[8..18].trim().parse().unwrap_or(0.0);
        let d_l: f64 = line[18..28].trim().parse().unwrap_or(0.0);
        let d_h: f64 = line[28..38].trim().parse().unwrap_or(0.0);
        let (latitude, longitude) = meshcode_to_latlon(&mesh_code);
        map.insert(mesh_code, SemiDynaRecord { d_b, d_l, d_h, latitude, longitude });
    }

    tracing::info!("loaded {} semidyna records from {}", map.len(), path.display());
    Ok(map)
}
