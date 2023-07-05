use super::coordinate;
use geo::{BoundingRect, Intersects, Polygon};
use wkt::TryFromWkt;

pub fn get_tile_level(tile_id: i64) -> i8 {
    // 根据tile_id获取其瓦片缩放级别
    let mut tile = tile_id;
    let mut level = 0;
    while tile != 0 {
        tile >>= 1;
        level += 1;
    }
    level - 17
}

pub fn get_tile_width(tile_id: i64) -> i64 {
    // 获取tile图幅宽度(分)
    let level = get_tile_level(tile_id);
    1 << (31 - level)
}

pub fn encode_bits(v: i64) -> i64 {
    // 压缩32位
    let mut val = v;
    val &= 0x5555555555555555;
    val = (val ^ (val >> 1)) & 0x3333333333333333;
    val = (val ^ (val >> 2)) & 0x0F0F0F0F0F0F0F0F;
    val = (val ^ (val >> 4)) & 0x00FF00FF00FF00FF;
    val = (val ^ (val >> 8)) & 0x0000FFFF0000FFFF;
    val = (val ^ (val >> 16)) & 0x00000000FFFFFFFF;
    val
}

pub fn decode_bits(v: i64) -> i64 {
    // 解压32
    let mut val = v;
    if val < 0 {
        val &= 0xFFFFFFFF;
    }
    val = (val | (val << 16)) & 0x0000FFFF0000FFFF;
    val = (val | (val << 8)) & 0x00FF00FF00FF00FF;
    val = (val | (val << 4)) & 0x0F0F0F0F0F0F0F0F;
    val = (val | (val << 2)) & 0x3333333333333333;
    val = (val | (val << 1)) & 0x5555555555555555;
    val
}

pub fn nds2deg(nds_val: f64) -> f64 {
    // """分转度"""
    let i: i64 = 1 << 30;
    let f: f64 = i as f64;
    let r: f64 = 90.0 * nds_val / f;
    r
}

pub fn deg2nds(deg_val: f64) -> i64 {
    // 度转分
    assert!(deg_val >= -180.0);
    assert!(deg_val <= 180.0);
    let i: i64 = 1 << 30;
    let f: f64 = i as f64;
    let r: f64 = f * deg_val / 90.0;
    r as i64
}

pub fn deg2morton(lon: f64, lat: f64) -> i64 {
    // 弧度(度)转莫顿编码
    let nds_x = deg2nds(lon);
    let ndx_y = deg2nds(lat);
    decode_bits(nds_x) | (decode_bits(ndx_y & 0x7FFFFFFF) << 1)
}

pub fn nds2morton(nds_x: i64, ndx_y: i64) -> i64 {
    // """弧度(分)转莫顿编码"""
    decode_bits(nds_x) | (decode_bits(ndx_y & 0x7FFFFFFF) << 1)
}

pub fn tileid2morton(tileid: i64) -> i64 {
    // """图幅号转莫顿码"""
    let level = get_tile_level(tileid);
    let morton_code_tile = (0xFFFFFFFF >> (31 - 2 * level)) & tileid;
    let morton_code = morton_code_tile << (62 - 2 * level);
    morton_code
}

pub fn tileid2nds(tileid: i64) -> (i64, i64) {
    // 根据图幅号获取图幅坐标值(分)
    let mortoncode = tileid2morton(tileid);
    let x = encode_bits(mortoncode);
    let y = encode_bits(mortoncode >> 1);
    (x, y)
}

pub fn tileid2deg(tileid: i64) -> (f64, f64) {
    // 根据图幅号获取图幅坐标值(度)

    let (nds_x, nds_y) = tileid2nds(tileid);
    let deg_x = nds2deg(nds_x as f64);
    let deg_y = nds2deg(nds_y as f64);
    (deg_x, deg_y)
}

pub fn morton2nds(morton_code: i64) -> (i64, i64) {
    // 通过莫顿编码获取弧度坐标(分)
    let x = encode_bits(morton_code);
    let mut y = encode_bits(morton_code >> 1);
    if (y & 0x40000000) != 0 {
        y = y | 0x80000000;
    }
    (x, y)
}

pub fn morton2tileid(morton_code: i64, level: i8) -> i64 {
    // 通过莫顿编码获取图幅号
    let packed_tile_id = morton_code >> (62 - 2 * level);
    let packed_level = 1 << (16 + level);
    packed_tile_id | packed_level
}

pub fn nds2tileid(nds_x: i64, nds_y: i64, level: i8) -> i64 {
    // 通过位置(分)获取图幅号
    let morton_code = nds2morton(nds_x, nds_y);
    let tileid = morton2tileid(morton_code, level);
    tileid
}

pub fn deg2tileid(lon: f64, lat: f64, level: i8) -> i64 {
    let nds_x = deg2nds(lon);
    let nds_y = deg2nds(lat);
    nds2tileid(nds_x, nds_y, level)
}

pub fn get_nds_border_by_tileid(tileid: i64) -> (i64, i64, i64, i64) {
    // 通过图幅号获取图幅边界(分)
    let morton_code = tileid2morton(tileid);
    let (x, y) = morton2nds(morton_code);
    let tile_width = get_tile_width(tileid);
    (x, y, x + tile_width, y + tile_width)
}

pub fn get_deg_border_by_tileid(tileid: i64) -> (f64, f64, f64, f64) {
    // 通过图幅号获取图幅边界(度)
    let (left, bottom, right, top) = get_nds_border_by_tileid(tileid);
    let left = nds2deg(left as f64);
    let bottom = nds2deg(bottom as f64);
    let right = nds2deg(right as f64);
    let top = nds2deg(top as f64);
    (left, bottom, right, top)
}

pub fn get_ceneter_nds_by_tileid(tileid: i64) -> (i64, i64) {
    // 根据图幅号获取图幅中心点弧度(分)
    let (left, bottom, right, top) = get_nds_border_by_tileid(tileid);
    let nds_x = left + (right - left) / 2;
    let nds_y = bottom + (top - bottom) / 2;
    (nds_x, nds_y)
}

pub fn get_center_deg_by_tileid(tileid: i64, system: &str) -> (f64, f64) {
    // 根据图幅号获取图幅中心点弧度(度)
    let (nds_x, nds_y) = get_ceneter_nds_by_tileid(tileid);
    let lon = nds2deg(nds_x as f64);
    let lat = nds2deg(nds_y as f64);
    if system == "02" {
        return coordinate::wgs842gcj02(lon, lat);
    }
    (lon, lat)
}

pub fn tile2geometry(tileid: i64, dimension: i8) -> String {
    // 根据图幅号获取图幅边界几何
    let (left, bottom, right, top) = get_deg_border_by_tileid(tileid);
    let (left_bottom_x, left_bottom_y) = coordinate::wgs842gcj02(left, bottom);
    let (left_top_x, left_top_y) = coordinate::wgs842gcj02(left, top);
    let (right_bottom_x, right_bottom_y) = coordinate::wgs842gcj02(right, bottom);
    let (rigth_top_x, rigth_top_y) = coordinate::wgs842gcj02(right, top);
    let mut tile_wkt = format!(
        "POLYGON Z (({} {} -1000000, {} {} -1000000, {} {} -1000000, {} {} -1000000, {} {} -1000000))",
        left_bottom_x,
        left_bottom_y,
        left_top_x,
        left_top_y,
        rigth_top_x,
        rigth_top_y,
        right_bottom_x,
        right_bottom_y,
        left_bottom_x,
        left_bottom_y
    );
    if dimension == 2 {
        tile_wkt = format!(
            "POLYGON (({} {}, {} {}, {} {}, {} {}, {} {}))",
            left_bottom_x,
            left_bottom_y,
            left_top_x,
            left_top_y,
            rigth_top_x,
            rigth_top_y,
            right_bottom_x,
            right_bottom_y,
            left_bottom_x,
            left_bottom_y
        );
    }
    tile_wkt
}

pub fn get_neighbor_tileid(tileid: i64, direction: &str) -> i64 {
    // 根据图幅号获取指定方位的相邻图幅号
    let level = get_tile_level(tileid);
    let tile_width = get_tile_width(tileid);
    let (mut center_x, mut center_y) = get_ceneter_nds_by_tileid(tileid);
    if direction == "RIGHT" {
        center_x += tile_width;
    } else if direction == "UP" {
        center_y += tile_width;
    } else if direction == "DOWN" {
        center_y -= tile_width;
    } else if direction == "LEFT" {
        center_x -= tile_width;
    } else if direction == "LEFT_DOWM" {
        center_x -= tile_width;
        center_y -= tile_width;
    } else if direction == "RIGHT_DOWN" {
        center_x += tile_width;
        center_y -= tile_width;
    } else if direction == "LEFT_UP" {
        center_x -= tile_width;
        center_y += tile_width;
    } else if direction == "RIGHT_UP" {
        center_y += tile_width;
        center_x += tile_width;
    }
    let tile_id = nds2tileid(center_x, center_y, level);
    tile_id
}

pub fn get_all_neighbor_tiles(tileid: i64) -> Vec<i64> {
    // 根据图幅号获取所有相邻图幅号
    let mut neighbor_tiles = Vec::new();
    neighbor_tiles.push(tileid);
    neighbor_tiles.push(get_neighbor_tileid(tileid, "UP"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "DOWN"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "LEFT"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "RIGHT"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "LEFT_UP"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "RIGHT_UP"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "LEFT_DOWM"));
    neighbor_tiles.push(get_neighbor_tileid(tileid, "RIGHT_DOWN"));
    neighbor_tiles
}

pub fn polygon2tiles(polygon: &str, level: i8) -> Vec<i64> {
    let g: Polygon = Polygon::try_from_wkt_str(polygon).unwrap();
    let b = g.bounding_rect().unwrap();
    let min_coords = b.min();
    let max_coords = b.max();
    let min_tile = deg2tileid(min_coords.x, min_coords.y, level);
    let max_tile = deg2tileid(max_coords.x, max_coords.y, level);
    let mut checked_tiles: Vec<i64> = vec![];
    for tileid in min_tile..=max_tile {
        let tile_wkt = tile2geometry(tileid, 2);
        let tile_polygon = Polygon::try_from_wkt_str(&tile_wkt).unwrap();
        if g.intersects(&tile_polygon) {
            checked_tiles.push(tileid);
        }
    }
    checked_tiles
}

pub fn tileid_transform(tileid: i64, to_level: i8) -> i64 {
    let (lng, lat) = tileid2deg(tileid);
    deg2tileid(lng, lat, to_level)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_tile_level() {
        assert_eq!(get_tile_level(556610400), 13);
        assert_eq!(get_tile_level(67424006), 10);
    }
    #[test]
    fn test_get_tile_width() {
        assert_eq!(get_tile_width(556610400), 262144);
    }
    #[test]
    fn test_encode_decode_bits() {
        assert_eq!(encode_bits(1330786202097287168), 1336934400);
        assert_eq!(encode_bits(1330779330149613568), 1334312960);
        assert_eq!(decode_bits(1334312960), 1176917521249337344);
        assert_eq!(decode_bits(1336934400), 1176922743929569280);
    }
    #[test]
    fn test_deg2nds() {
        assert_eq!(deg2nds(77.50886984831503), 924716836);
        assert_eq!(nds2deg(924716836.0), 77.50886980444193);
    }

    #[test]
    fn test_deg2morton() {
        assert_eq!(
            deg2morton(77.50886984831503, 43.13809797582891),
            557753618534958618
        );
    }

    #[test]
    fn test_nds2morton() {
        assert_eq!(
            nds2morton(deg2nds(77.50886984831503), deg2nds(43.13809797582891)),
            557753618534958618
        );
    }
    #[test]
    fn test_tile2morton() {
        assert_eq!(tileid2morton(556236300), 1330779330149613568);
    }
    #[test]
    fn test_tileid2deg() {
        assert_eq!(tileid2deg(556236300), (111.8408203125, 30.6298828125));
    }
    #[test]
    fn test_deg2tileid() {
        assert_eq!(deg2tileid(111.8408203125, 30.6298828125, 13), 556236300);
        assert_eq!(deg2tileid(111.8408203125, 30.6298828125, 10), 67411448)
    }
    #[test]
    fn test_get_deg_border_by_tileid() {
        assert_eq!(
            get_deg_border_by_tileid(556236300),
            (
                111.8408203125,
                30.6298828125,
                111.86279296875,
                30.65185546875
            )
        );
        assert_eq!(
            get_deg_border_by_tileid(67411448),
            (111.796875, 30.5859375, 111.97265625, 30.76171875)
        );
    }
    #[test]
    fn test_get_center_deg_by_tileid() {
        assert_eq!(
            get_center_deg_by_tileid(556236300, "02"),
            (111.85744626366223, 30.63825364888663)
        );
        assert_eq!(
            get_center_deg_by_tileid(67411448, "02"),
            (111.89035707209221, 30.671186392934953)
        );
    }
    #[test]
    fn test_tile2geometry() {
        let tile_polygon = "POLYGON Z ((111.84648227716731 30.62728190721755 -1000000, 111.84648417243615 30.649263977286452 -1000000, 111.86841269240549 30.649227710764272 -1000000, 111.86841080660852 30.62724564013721 -1000000, 111.84648227716731 30.62728190721755 -1000000))";
        assert_eq!(tile2geometry(556236300, 3), tile_polygon);
    }
    #[test]
    fn test_get_all_neighbor_tiles() {
        assert_eq!(
            get_all_neighbor_tiles(556236300),
            [
                556236300, 556236302, 556236294, 556236297, 556236301, 556236299, 556236303,
                556236291, 556236295
            ]
        );
    }
    #[test]
    fn test_polygon2tiles() {
        let tile_polygon = "POLYGON ((111.84648227716731 30.62728190721755, 111.84648417243615 30.649263977286452, 111.86841269240549 30.649227710764272, 111.86841080660852 30.62724564013721, 111.84648227716731 30.62728190721755))";
        assert_eq!(
            polygon2tiles(tile_polygon, 13),
            [556236294, 556236295, 556236297, 556236299, 556236300, 556236301]
        )
    }
    #[test]
    fn test_tileid_transform() {
        assert_eq!(tileid_transform(556236300, 10), 67411448);
    }
}
