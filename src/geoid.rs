use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// GSI ジオイドモデル（gsigeo2011_ver2_1.asc）をメモリに保持し、バイリニア補間でジオイド高・標高を計算する。
pub struct JpGeoidModel {
    lat_min: f64,
    lon_min: f64,
    lat_div: f64,
    lon_div: f64,
    lat_ngrid: usize,
    lon_ngrid: usize,
    data: Vec<f64>, // row-major [lat_ngrid × lon_ngrid], 欠損値は NaN
}

impl JpGeoidModel {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        const LAT_MIN: f64 = 20.0;
        const LON_MIN: f64 = 120.0;
        const LAT_MAX: f64 = 50.0;
        const LON_MAX: f64 = 150.0;
        const LAT_NGRID: usize = 1801;
        const LON_NGRID: usize = 1201;

        let lat_div = (LAT_MAX - LAT_MIN) / (LAT_NGRID - 1) as f64;
        let lon_div = (LON_MAX - LON_MIN) / (LON_NGRID - 1) as f64;
        let mut data = vec![f64::NAN; LAT_NGRID * LON_NGRID];

        let mut lines = BufReader::new(File::open(path)?).lines();
        lines.next(); // ヘッダー行をスキップ

        let mut i_idx = 0usize;
        let mut j_idx = 0usize;

        for line in lines {
            let line = line?;
            // Python の len(line) != 253 に相当（BufReader::lines は改行を除くため 252）
            if line.len() != 252 {
                continue;
            }
            for j in 0..28usize {
                let s = &line[j * 9..(j + 1) * 9];
                if let Ok(v) = s.trim().parse::<f64>() {
                    // 999.0 は欠損値マーカー
                    let value = if (v - 999.0).abs() < 1e-6 { f64::NAN } else { v };
                    if i_idx < LAT_NGRID && j_idx < LON_NGRID {
                        data[i_idx * LON_NGRID + j_idx] = value;
                    }
                    j_idx += 1;
                }
            }
            if j_idx >= LON_NGRID {
                j_idx = 0;
                i_idx += 1;
            }
        }

        tracing::info!("geoid model loaded: {}x{} from {}", LAT_NGRID, LON_NGRID, path.display());
        Ok(Self {
            lat_min: LAT_MIN,
            lon_min: LON_MIN,
            lat_div,
            lon_div,
            lat_ngrid: LAT_NGRID,
            lon_ngrid: LON_NGRID,
            data,
        })
    }

    /// バイリニア補間でジオイド高（m）を計算する。範囲外または欠損データの場合は None。
    pub fn geoid_height(&self, lat: f64, lon: f64) -> Option<f64> {
        let lat_max = self.lat_min + self.lat_div * (self.lat_ngrid - 1) as f64;
        let lon_max = self.lon_min + self.lon_div * (self.lon_ngrid - 1) as f64;
        if lat < self.lat_min || lat > lat_max || lon < self.lon_min || lon > lon_max {
            return None;
        }
        let i = ((lat - self.lat_min) / self.lat_div).floor() as usize;
        let j = ((lon - self.lon_min) / self.lon_div).floor() as usize;
        let i = i.min(self.lat_ngrid - 2);
        let j = j.min(self.lon_ngrid - 2);
        let t = (lat - (self.lat_min + self.lat_div * i as f64)) / self.lat_div;
        let u = (lon - (self.lon_min + self.lon_div * j as f64)) / self.lon_div;
        let g = |ii: usize, jj: usize| self.data[ii * self.lon_ngrid + jj];
        let (z00, z01, z10, z11) = (g(i, j), g(i, j + 1), g(i + 1, j), g(i + 1, j + 1));
        if z00.is_nan() || z01.is_nan() || z10.is_nan() || z11.is_nan() {
            return None;
        }
        Some(
            (1.0 - t) * (1.0 - u) * z00
                + (1.0 - t) * u * z01
                + t * (1.0 - u) * z10
                + t * u * z11,
        )
    }

    /// 楕円体高 alt からジオイド高を差し引いた標高（m）を計算する。
    pub fn elevation(&self, lat: f64, lon: f64, alt: f64) -> Option<f64> {
        self.geoid_height(lat, lon).map(|z| alt - z)
    }
}
