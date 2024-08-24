
use std::collections::HashMap;

use crate::{Element, Tilemap, Value};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Map {
    pub package: String, // package
    pub filler: Vec<Filler>, // Filler
    pub levels: Vec<Level>, // levels
    pub foregrounds: Vec<Element>, // Style::Foregrounds
    pub backgrounds: Vec<Element>, // Style::Backgrounds
    pub bg_color: Option<[u8; 4]>, // Style.color
    pub extra_data: HashMap<String, Value>,
    pub extra_children: Vec<Element>
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Filler { // Filler
    pub position: (i32, i32), // x, y
    pub size: (i32, i32), // w, h
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Level {
    pub name: String, // name
    pub data: LevelData, 
    pub entities: Vec<Entity>, // entities, optional
    pub triggers: Vec<Entity>, // triggers, optional
    pub bg_decals: Vec<Decal>, // bgdecals, optional
    pub fg_decals: Vec<Decal>, // fgdecals, optional
    pub bg: Tilemap<char>, // bg (RLE, Vec<char> is usually dumb but it's used here for the ability to index into and set things)
    pub bg_tiles: Tilemap<i32>, // bgtiles
    pub fg_tiles: Tilemap<i32>, // fgtiles
    pub obj_tiles: Tilemap<i32>, // objtiles
    pub solids: Tilemap<char>, // solids (RLE)
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
    pub wind_pattern: String, // windPattern
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
pub struct Entity {
    pub name: String, // element name
    pub id: i32, // id
    pub position: (f32, f32), // x, y
    pub width: Option<i32>, // width
    pub height: Option<i32>, // width
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
