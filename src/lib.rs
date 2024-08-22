#![warn(clippy::pedantic, clippy::perf)]

use std::io;
use byteorder::{LittleEndian, ReadBytesExt};

mod ext;
use ext::ReadExt;

pub mod error;
pub use error::LoadError;

pub mod element;
pub use element::{Element, Value}; 

pub mod map_data;
pub use map_data::{Map, Level, Filler, Entity, Decal, LevelData};

pub mod map_serde;
pub use map_serde::MapElement;

impl Element {
    /// Loads a map from a readable stream.
    /// 
    /// # Errors
    /// Errors if the map fails to load. See [`LoadError`] for more information.
    pub fn load_map(stream: &mut dyn io::Read, check_header: bool) -> Result<Element, LoadError> {
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
        map.attributes.insert("package".to_string(), package.into());
        Ok(map)
    }
}