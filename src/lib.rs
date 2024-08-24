#![warn(missing_docs, clippy::pedantic, clippy::perf)]
/*!
# strawberride

---

This is a library for loading and saving Celeste maps from files,
including high-level representations of map objects like
[`Level`], [`Entity`], [`Decal`], and [`Tilemap`].

```ignore
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

*/


use std::io::{self, Cursor};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt as _};

mod ext;
use ext::{ReadExt, WriteExt};

mod error;
pub use error::LoadError;

mod element;
pub use element::{Element, Value}; 

mod map_data;
use indexmap::IndexSet;
pub use map_data::{Map, Level, Filler, Entity, Decal, LevelData};

mod map_serde;
pub use map_serde::MapElement;

mod tilemap;
pub use tilemap::Tilemap; 

impl Map {
    /// Loads a [`Map`] from a readable stream, with Celeste's map format.
    /// 
    /// # Errors
    /// Errors if the map fails to load. See [`LoadError`] for more information.
    pub fn load(stream: &mut dyn io::Read, check_header: bool) -> Result<Map, LoadError> {
        if check_header {
            let header = stream.read_string()?;
            if header != "CELESTE MAP" {
                return Err(LoadError::InvalidHeader(header));
            }
        }
        
        let package = stream.read_string()?;
        let lookup_length = stream.read_u16::<LittleEndian>()?;
        let lookup = (0 .. lookup_length)
            .map(|_| stream.read_string())
            .collect::<Result<Vec<_>, _>>()?;
        
        let mut map = Element::decode(stream, &lookup)?;
        map.attributes.insert("_package".to_string(), package.into());

        map.try_into()
    }

    /// Stores this [`Map`] into a writable stream, with Celeste's map format.
    /// 
    /// # Errors
    /// Errors if an IO error occurs during writing.
    pub fn store(self, stream: &mut dyn io::Write, write_header: bool) -> io::Result<()> {
        let mut buf = Cursor::new(Vec::new());
        let mut strings = IndexSet::new();

        let mut el = Element::from(self);
        let package = el.attributes.remove("_package")
            .map_or(String::new(), |val| match val {
                Value::Boolean(bool) => bool.to_string(),
                Value::Float(float) => float.to_string(),
                Value::Integer(int) => int.to_string(),
                Value::String(str) | Value::RleString(str)
                    => str
            });
        el.encode(&mut buf, &mut strings)?;
        
        let Ok(lookup_length) = u16::try_from(strings.len())
        else {
            return Err(io::Error::other("cannot store more than 65535 unique strings in a map"))
        };

        if write_header {
            stream.write_string("CELESTE MAP")?;
        }

        stream.write_string(&package)?;
        stream.write_u16::<LittleEndian>(lookup_length)?;

        for string in strings.iter() {
            stream.write_string(string)?;
        }

        stream.write_all(buf.get_ref())
    }
}