pub fn wgs842gcj02(lng: f64, lat: f64) -> (f64, f64) {
    const SEMI_MAJOR_AXIS: f64 = 6378245.0;
    const EE: f64 = 0.00669342162296594323;
    const PI: f64 = std::f64::consts::PI;

    let gcj02_x: f64;
    let gcj02_y: f64;

    let lng2: f64 = lng - 105.0;
    let lat2: f64 = lat - 35.0;
    let mut dlat: f64 = -100.0
        + 2.0 * lng2
        + 3.0 * lat2
        + 0.2 * lat2 * lat2
        + 0.1 * lng2 * lat2
        + 0.2 * lng2.abs().sqrt()
        + (20.0 * (6.0 * lng2 * PI).sin() + 20.0 * (2.0 * lng2 * PI).sin()) * 2.0 / 3.0
        + (20.0 * (lat2 * PI).sin() + 40.0 * (lat2 / 3.0 * PI).sin()) * 2.0 / 3.0
        + (160.0 * (lat2 / 12.0 * PI).sin() + 320.0 * (lat2 * PI / 30.0).sin()) * 2.0 / 3.0;
    let mut dlng: f64 = 300.0
        + lng2
        + 2.0 * lat2
        + 0.1 * lng2 * lng2
        + 0.1 * lng2 * lat2
        + 0.1 * (lng2.abs()).sqrt()
        + (20.0 * (6.0 * lng2 * PI).sin() + 20.0 * (2.0 * lng2 * PI).sin()) * 2.0 / 3.0
        + (20.0 * (lng2 * PI).sin() + 40.0 * (lng2 / 3.0 * PI).sin()) * 2.0 / 3.0
        + (150.0 * (lng2 / 12.0 * PI).sin() + 300.0 * (lng2 / 30.0 * PI).sin()) * 2.0 / 3.0;

    let radlat = lat / 180.0 * PI;
    let mut magic = radlat.sin();
    magic = 1.0 - EE * magic * magic;
    let sqrtmagic = magic.sqrt();
    dlat = (dlat * 180.0) / ((SEMI_MAJOR_AXIS * (1.0 - EE)) / (magic * sqrtmagic) * PI);
    dlng = (dlng * 180.0) / (SEMI_MAJOR_AXIS / sqrtmagic * radlat.cos() * PI);
    gcj02_y = lat + dlat;
    gcj02_x = lng + dlng;
    (gcj02_x, gcj02_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_wgs842gcj02() {
        assert_eq!(
            wgs842gcj02(107.55892137719799, 48.048944178597864),
            (107.56503861907252, 48.05044502209457)
        );
    }
}
