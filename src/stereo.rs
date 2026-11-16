//! 二重ステレオ投影（現場座標変換）。
//! 仕様: misc/stereoparameter/stereo_projection_spec.md
#![allow(dead_code)]

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ellipsoid {
    pub a: f64,
    pub f: f64,
}

impl Ellipsoid {
    pub const GRS80: Ellipsoid = Ellipsoid { a: 6378137.0, f: 1.0 / 298.257222101 };
}

impl Default for Ellipsoid {
    fn default() -> Self {
        Self::GRS80
    }
}

/// 現場（ローカライゼーション）パラメータ。角度は rad。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SiteParams {
    pub origin_lat: f64,
    pub origin_lon: f64,
    pub false_east: f64,
    pub false_north: f64,
    pub scale: f64,
    pub rotation: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EN {
    pub east: f64,
    pub north: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidParams,
    InvalidInput,
    NotConverged,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidParams => write!(f, "invalid projection parameters"),
            Error::InvalidInput => write!(f, "invalid input coordinate"),
            Error::NotConverged => write!(f, "latitude iteration did not converge"),
        }
    }
}

impl std::error::Error for Error {}

const ITER_LIMIT: usize = 20;
const ITER_TOL: f64 = 1.0e-12;

pub fn deg_to_rad(x: f64) -> f64 {
    x.to_radians()
}

pub fn rad_to_deg(x: f64) -> f64 {
    x.to_degrees()
}

/// 原点パラメータを前計算して保持する。生成後は不変。
#[derive(Debug, Clone)]
pub struct StereoProjection {
    e: f64,
    e2: f64,
    params: SiteParams,
    r: f64,
    n: f64,
    c: f64,
    chi0: f64,
    sin_chi0: f64,
    cos_chi0: f64,
    sin_rot: f64,
    cos_rot: f64,
}

impl StereoProjection {
    pub fn new(ellipsoid: Ellipsoid, params: SiteParams) -> Result<Self, Error> {
        let vals = [
            ellipsoid.a, ellipsoid.f, params.origin_lat, params.origin_lon,
            params.false_east, params.false_north, params.scale, params.rotation,
        ];
        if vals.iter().any(|v| !v.is_finite())
            || params.origin_lat.abs() >= FRAC_PI_2
            || params.scale <= 0.0
            || ellipsoid.a <= 0.0
        {
            return Err(Error::InvalidParams);
        }

        let a = ellipsoid.a;
        let e2 = ellipsoid.f * (2.0 - ellipsoid.f);
        let e = e2.sqrt();
        let phi0 = params.origin_lat;
        let s = phi0.sin();

        let rho0 = a * (1.0 - e2) / (1.0 - e2 * s * s).powf(1.5);
        let nu0 = a / (1.0 - e2 * s * s).sqrt();
        let r = (rho0 * nu0).sqrt();
        let n = (1.0 + e2 * phi0.cos().powi(4) / (1.0 - e2)).sqrt();

        let s1 = (1.0 + s) / (1.0 - s);
        let s2 = (1.0 - e * s) / (1.0 + e * s);
        let w1 = (s1 * s2.powf(e)).powf(n);
        let sin_chi0 = (w1 - 1.0) / (w1 + 1.0);
        let c = (n + s) * (1.0 - sin_chi0) / ((n - s) * (1.0 + sin_chi0));
        let w2 = c * w1;
        let chi0 = ((w2 - 1.0) / (w2 + 1.0)).asin();

        Ok(Self {
            e,
            e2,
            params,
            r,
            n,
            c,
            chi0,
            sin_chi0: chi0.sin(),
            cos_chi0: chi0.cos(),
            sin_rot: params.rotation.sin(),
            cos_rot: params.rotation.cos(),
        })
    }

    /// 緯度経度 [rad] → 現場座標 [m]。
    pub fn forward(&self, p: LatLon) -> Result<EN, Error> {
        if !p.lat.is_finite() || !p.lon.is_finite() || p.lat.abs() >= FRAC_PI_2 {
            return Err(Error::InvalidInput);
        }
        let e = self.e;
        let s = p.lat.sin();
        let sa = (1.0 + s) / (1.0 - s);
        let sb = (1.0 - e * s) / (1.0 + e * s);
        let w = self.c * (sa * sb.powf(e)).powf(self.n);
        let chi = ((w - 1.0) / (w + 1.0)).asin();
        let dl = self.n * (p.lon - self.params.origin_lon);

        let (sin_chi, cos_chi) = (chi.sin(), chi.cos());
        let b = 1.0 + sin_chi * self.sin_chi0 + cos_chi * self.cos_chi0 * dl.cos();
        let k = 2.0 * self.r * self.params.scale / b;
        let x = k * cos_chi * dl.sin();
        let y = k * (sin_chi * self.cos_chi0 - cos_chi * self.sin_chi0 * dl.cos());

        Ok(EN {
            east: self.params.false_east + x * self.cos_rot + y * self.sin_rot,
            north: self.params.false_north - x * self.sin_rot + y * self.cos_rot,
        })
    }

    /// 現場座標 [m] → 緯度経度 [rad]。
    pub fn inverse(&self, q: EN) -> Result<LatLon, Error> {
        if !q.east.is_finite() || !q.north.is_finite() {
            return Err(Error::InvalidInput);
        }
        let de = q.east - self.params.false_east;
        let dn = q.north - self.params.false_north;
        let x = de * self.cos_rot - dn * self.sin_rot;
        let y = de * self.sin_rot + dn * self.cos_rot;

        let rk = self.r * self.params.scale;
        let g = 2.0 * rk * (FRAC_PI_4 - self.chi0 / 2.0).tan();
        let h = 4.0 * rk * self.chi0.tan() + g;
        let i = x.atan2(h + y);
        let j = x.atan2(g - y) - i;
        let chi = self.chi0 + 2.0 * ((y - x * (j / 2.0).tan()) / (2.0 * rk)).atan();
        let lam0 = self.params.origin_lon;
        let big_lambda = j + 2.0 * i + lam0;
        let lon = (big_lambda - lam0) / self.n + lam0;

        let sin_chi = chi.sin();
        let psi_target = 0.5 * ((1.0 + sin_chi) / (self.c * (1.0 - sin_chi))).ln() / self.n;

        let e = self.e;
        let mut phi = 2.0 * psi_target.exp().atan() - FRAC_PI_2;
        for _ in 0..ITER_LIMIT {
            let s = phi.sin();
            let psi = ((phi / 2.0 + FRAC_PI_4).tan()
                * ((1.0 - e * s) / (1.0 + e * s)).powf(e / 2.0))
            .ln();
            let delta =
                (psi - psi_target) * phi.cos() * (1.0 - self.e2 * s * s) / (1.0 - self.e2);
            phi -= delta;
            if !phi.is_finite() {
                return Err(Error::NotConverged);
            }
            if delta.abs() < ITER_TOL {
                return Ok(LatLon { lat: phi, lon });
            }
        }
        Err(Error::NotConverged)
    }
}

pub fn forward_once(lat: f64, lon: f64, params: SiteParams) -> Result<EN, Error> {
    StereoProjection::new(Ellipsoid::GRS80, params)?.forward(LatLon { lat, lon })
}

pub fn inverse_once(east: f64, north: f64, params: SiteParams) -> Result<LatLon, Error> {
    StereoProjection::new(Ellipsoid::GRS80, params)?.inverse(EN { east, north })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(lat0: f64, lon0: f64, fe: f64, fn_: f64, k0: f64, rot_deg: f64) -> SiteParams {
        SiteParams {
            origin_lat: deg_to_rad(lat0),
            origin_lon: deg_to_rad(lon0),
            false_east: fe,
            false_north: fn_,
            scale: k0,
            rotation: deg_to_rad(rot_deg),
        }
    }

    // (lat, lon, params, E, N)  仕様書 §6
    fn vectors() -> Vec<(f64, f64, SiteParams, f64, f64)> {
        vec![
            (36.0, 139.0, params(35.5, 139.5, 0.0, 0.0, 1.0, 0.0), -45082.692369, 55592.468012),
            (36.0, 139.0, params(35.5, 139.5, 1000.0, 2000.0, 0.9999, 0.5), -43591.386528, 57978.168560),
            (34.7, 135.0, params(34.5, 135.0, 0.0, 0.0, 1.0, 0.0), 0.0, 22186.676376),
        ]
    }

    #[test]
    fn forward_matches_vectors() {
        for (lat, lon, p, e, n) in vectors() {
            let r = forward_once(deg_to_rad(lat), deg_to_rad(lon), p).unwrap();
            assert!((r.east - e).abs() < 1e-6, "E {} vs {}", r.east, e);
            assert!((r.north - n).abs() < 1e-6, "N {} vs {}", r.north, n);
        }
    }

    #[test]
    fn inverse_matches_vectors() {
        for (lat, lon, p, e, n) in vectors() {
            let r = inverse_once(e, n, p).unwrap();
            assert!((rad_to_deg(r.lat) - lat).abs() < 1e-8, "lat {}", rad_to_deg(r.lat));
            assert!((rad_to_deg(r.lon) - lon).abs() < 1e-8, "lon {}", rad_to_deg(r.lon));
        }
    }

    #[test]
    fn round_trip_is_tight() {
        let p = params(36.0, 139.8, 500.0, -300.0, 0.9999, 12.3);
        let proj = StereoProjection::new(Ellipsoid::GRS80, p).unwrap();
        for dlat in [-0.2, -0.05, 0.0, 0.03, 0.2] {
            for dlon in [-0.2, -0.05, 0.0, 0.04, 0.2] {
                let ll = LatLon { lat: p.origin_lat + deg_to_rad(dlat), lon: p.origin_lon + deg_to_rad(dlon) };
                let back = proj.inverse(proj.forward(ll).unwrap()).unwrap();
                assert!(rad_to_deg((back.lat - ll.lat).abs()) < 1e-10);
                assert!(rad_to_deg((back.lon - ll.lon).abs()) < 1e-10);
            }
        }
    }

    #[test]
    fn origin_maps_to_false_origin() {
        let p = params(35.0, 139.0, 123.0, 456.0, 0.9996, 3.0);
        let proj = StereoProjection::new(Ellipsoid::GRS80, p).unwrap();
        let en = proj.forward(LatLon { lat: p.origin_lat, lon: p.origin_lon }).unwrap();
        assert!((en.east - 123.0).abs() < 1e-9 && (en.north - 456.0).abs() < 1e-9);
        let ll = proj.inverse(EN { east: 123.0, north: 456.0 }).unwrap();
        assert!((ll.lat - p.origin_lat).abs() < 1e-12 && (ll.lon - p.origin_lon).abs() < 1e-12);
    }

    #[test]
    fn rotation_is_applied_about_false_origin() {
        let base = params(35.5, 139.5, 0.0, 0.0, 1.0, 0.0);
        let rot = params(35.5, 139.5, 0.0, 0.0, 1.0, 10.0);
        let (lat, lon) = (deg_to_rad(35.6), deg_to_rad(139.6));
        let a = forward_once(lat, lon, base).unwrap();
        let b = forward_once(lat, lon, rot).unwrap();
        let t = deg_to_rad(10.0);
        assert!((b.east - (a.east * t.cos() + a.north * t.sin())).abs() < 1e-8);
        assert!((b.north - (-a.east * t.sin() + a.north * t.cos())).abs() < 1e-8);
    }

    #[test]
    fn rejects_invalid_input() {
        let ok = params(35.0, 139.0, 0.0, 0.0, 1.0, 0.0);
        let mut bad = ok;
        bad.scale = 0.0;
        assert_eq!(StereoProjection::new(Ellipsoid::GRS80, bad).unwrap_err(), Error::InvalidParams);
        bad = ok;
        bad.origin_lat = PI / 2.0;
        assert_eq!(StereoProjection::new(Ellipsoid::GRS80, bad).unwrap_err(), Error::InvalidParams);
        bad = ok;
        bad.rotation = f64::NAN;
        assert_eq!(StereoProjection::new(Ellipsoid::GRS80, bad).unwrap_err(), Error::InvalidParams);
        let proj = StereoProjection::new(Ellipsoid::GRS80, ok).unwrap();
        assert_eq!(proj.forward(LatLon { lat: f64::NAN, lon: 0.0 }).unwrap_err(), Error::InvalidInput);
        assert_eq!(proj.inverse(EN { east: f64::INFINITY, north: 0.0 }).unwrap_err(), Error::InvalidInput);
    }
}
