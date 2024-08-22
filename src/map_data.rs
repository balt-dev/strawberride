
use std::collections::HashMap;

use itertools::Itertools;

use crate::{Element, Value, LoadError};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Map {
    pub package: String, // package
    pub filler: Vec<Filler>, // Filler
    pub levels: Vec<Level>, // levels
    pub extra_data: HashMap<String, Value>,
    pub extra_children: Vec<Element>
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Filler { // Filler
    pub position: (i32, i32), // x, y
    pub size: (i32, i32), // w, h
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindPattern {
    Left,
    Right,
    LeftStrong,
    RightStrong,
    LeftOnOff,
    RightOnOff,
    LeftOnOffFast,
    RightOnOffFast,
    Alternating,
    LeftGemsOnly,
    RightCrazy,
    Down,
    Up,
    Space,
    Custom(String) // Thanks, Monika.
}

impl From<String> for WindPattern {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Left" => Self::Left,
            "Right" => Self::Right,
            "LeftStrong" => Self::LeftStrong,
            "RightStrong" => Self::RightStrong,
            "LeftOnOff" => Self::LeftOnOff,
            "RightOnOff" => Self::RightOnOff,
            "LeftOnOffFast" => Self::LeftOnOffFast,
            "RightOnOffFast" => Self::RightOnOffFast,
            "Alternating" => Self::Alternating,
            "LeftGemsOnly" => Self::LeftGemsOnly,
            "RightCrazy" => Self::RightCrazy,
            "Down" => Self::Down,
            "Up" => Self::Up,
            "Space" => Self::Space,
            _ => Self::Custom(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Level {
    pub name: String, // name
    pub data: LevelData, 
    pub entities: Vec<Entity>, // entities
    pub triggers: Vec<Entity>, // triggers
    pub bg_decals: Vec<Decal>, // bgdecals
    pub fg_decals: Vec<Decal>, // fgdecals
    pub bg: Vec<char>, // bg (RLE, Vec<char> is usually dumb but it's used here for the ability to index into and set things)
    pub bgtiles: Vec<char>, // bgtiles (RLE) 
    pub fgtiles: Vec<char>, // fgtiles (RLE)
    pub objtiles: Vec<i32>, // objtiles (stored as comma separated numbers BUT STILL RLE FOR SOME REASON)
    pub solids: Vec<char>, // solids (RLE)
    pub extra_data: HashMap<String, Value>,
    pub extra_children: Vec<Element>
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LevelData {
    pub position: (i32, i32), // x, y
    pub size: (i32, i32), // width, height
    pub music_layers: [bool; 4], // musicLayer1-4
    pub underwater: bool, // underwater
    pub space: bool, // space
    pub disable_down_transition: bool, // disableDownTransition
    pub music_progress: Option<i32>, // musicProgress (stored as string for some reason?)
    pub camera_offset: (i32, i32), // cameraOffsetX, cameraOffsetY
    pub wind_pattern: Option<WindPattern>, // windPattern (stored as a string, None => "None")
    pub ambience_progress: Option<i32>, // ambienceProgress (also stored as a string??)
    pub alt_music: String, // alt_music (confusingly not camelCase)
    pub ambience: String, // ambience
    pub delay_alt_music_fade: bool, // delayAltMusicFade
    pub music: String, // music
    pub color: i32, // c
    pub dark: bool, // dark
    pub enforce_dash_number: Option<i32>, // enforceDashNumber (optional)
    pub whisper: bool, // whisper
}

#[derive(Debug, Clone, PartialEq, Default)]
// manual impl
pub struct Entity {
    pub id: i32, // id
    pub position: (f32, f32), // x, y
    pub size: (i32, i32), // width, height
    pub origin: (f32, f32), // originX, originY
    pub nodes: Vec<(f32, f32)>, // children (with name "node")
    pub values: HashMap<String, Value>
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Decal {
    pub position: (f32, f32), // x,
    pub scale: (f32, f32), // scaleX, scaleY
    pub texture: String, // texture
    pub color: [u8; 4], // color
    pub depth: i32, // depth
    pub rotation: f32 // rotation
}
