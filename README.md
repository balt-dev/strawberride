# strawberride


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