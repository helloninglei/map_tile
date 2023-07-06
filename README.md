map_tile
===
libary for map tile.

Dual-licensed under MIT or the [UNLICENSE](http://unlicense.org).

[![map_tile on Crates.io](https://img.shields.io/crates/v/map_tile.svg?color=brightgreen)](https://crates.io/crates/map_tile)
[![Documentation](https://img.shields.io/docsrs/map_tile/latest.svg)](https://docs.rs/map_tile)
### Documentation

https://docs.rs/map_tile

If you're new to Rust, the

### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
map_tile = "0.1.1"
```

### Example

```rust
use map_tile::tile::{tile2geometry, polygon2tiles, get_all_neighbor_tiles};

let tileid = 67435200;

let geom = tile2geometry(tileid, 3);

let polygon = "POLYGON ((111.84648227716731 30.62728190721755, 111.84648417243615 30.649263977286452, 111.86841269240549 30.649227710764272, 111.86841080660852 30.62724564013721, 111.84648227716731 30.62728190721755))";
let tiles = polygon2tiles(polygon, 13)

let all_near_tiles = get_all_neighbor_tiles(tileid);
let up_tile = get_neighbor_tileid(tileid, "UP");
let down_tile = get_neighbor_tileid(tileid, "DOWN");
let left_tile = get_neighbor_tileid(tileid, "LEFT");
let right_tile = get_neighbor_tileid(tileid, "RIGHT");
let leftup_tile = get_neighbor_tileid(tileid, "LEFT_UP");
let rightup_tile = get_neighbor_tileid(tileid, "RIGHT_UP");
let leftdown_tile = get_neighbor_tileid(tileid, "LEFT_DOWM");
let rightdown_tile = get_neighbor_tileid(tileid, "RIGHT_DOWN");
```