# strawberride

![Passing Tests](https://img.shields.io/badge/tests-passing-success.svg)
[![Documentation](https://docs.rs/strawberride/badge.svg)](https://docs.rs/strawberride)
[![Repository](https://img.shields.io/badge/-GitHub-%23181717?style=flat&logo=github&labelColor=%23555555&color=%23181717)](https://github.com/balt-dev/strawberride)
[![Latest version](https://img.shields.io/crates/v/strawberride.svg)](https://crates.io/crates/strawberride)
[![License](https://img.shields.io/crates/l/strawberride.svg)](https://github.com/balt-dev/strawberride/blob/trunk/LICENSE)
![Maintenance](https://img.shields.io/maintenance/passively-developed/2024?color=ok)

This is a library for loading and saving Celeste maps from files,
including high-level representations of map objects like
levels, entities, decals, tilemaps, and more!

This is focused on accuracy and ergonomics, being able to losslessly load and save any map as needed.

```rust
/// Rotate all levels in the map 180 degrees
let mut f = File::options()
    .read(true)
    .write(true)
    .open("map.bin").unwrap();
let mut map = Map::load(&mut f, true).unwrap();

for level in map.levels.iter_mut() {
    let tilemap = &mut level.solids;
    let width = tilemap.width();
    let height = tilemap.height();

    for y in 0..(height / 2) {
        for x in 0..width {
            let src = *tilemap.get(x, y).unwrap();
            let dst = *tilemap.get(width - x - 1, height - y - 1).unwrap();
            *tilemap.get_mut(x, y).unwrap() = dst;
            *tilemap.get_mut(width - x - 1, height - y - 1).unwrap() = src;
        }
    }
}

f.seek(SeekFrom::Start(0)).unwrap();

map.store(&mut f, true).unwrap();
```