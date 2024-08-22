use std::collections::HashMap;

use itertools::Itertools as _;

use crate::{
    map_data::WindPattern, Decal, Element, Entity, Filler, Level, LevelData, LoadError, Map, Value,
};

// So.
// This originally used Serde, and it was pretty nice,
// but I ran into some fundamental incompatibilities that
// would be pretty janky to work around.
// So I didn't.
// Below is manual implementations of TryFrom<Element> and Into<Element>.

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Map {}
    impl Sealed for super::Filler {}
    impl Sealed for super::Level {}
    impl Sealed for super::LevelData {}
    impl Sealed for super::Entity {}
    impl Sealed for super::Decal {}
}

/// Dictates an object as part of a map.
///
/// This trait is **sealed.**
pub trait MapElement: TryFrom<Element, Error = LoadError> + Into<Element> + sealed::Sealed {}

impl MapElement for Map {}
impl MapElement for Filler {}
impl MapElement for Level {}
impl MapElement for LevelData {}
impl MapElement for Entity {}
impl MapElement for Decal {}

macro_rules! get_as {
    ($el: ident [ $field_name: literal ]: $ty: ident or $default: expr) => {
        if let Some(field) = $el.attributes.remove($field_name) {
            if let Value::$ty(res) = field {
                res
            } else {
                return Err(LoadError::InvalidFieldType($field_name, field));
            }
        } else {
            $default
        }
    };
    ($el: ident [ $field_name: literal ]: $ty: ident) => {
        if let Some(field) = $el.attributes.remove($field_name) {
            if let Value::$ty(res) = field {
                Some(res)
            } else {
                return Err(LoadError::InvalidFieldType($field_name, field));
            }
        } else {
            None
        }
    };
}

macro_rules! attributes {
    ($($name: literal $(if $guard: expr)? => $expr: expr),*) => {{
        let mut map = HashMap::<String, Value>::new();
        $(
            attributes!(_single map $name $(if $guard)? => $expr);
        )*
        map
    }};
    (_single $map: ident $name: literal => $expr: expr) => {
        $map.insert($name.into(), $expr.into());
    };
    (_single $map: ident $name: literal if $guard: expr => $expr: expr) => {
        if $guard {
            $map.insert($name.into(), $expr.into());
        }
    }
}

macro_rules! check_name {
    ($ident: ident is $name: literal) => {
        if ($ident.name != $name) {
            return Err(LoadError::InvalidElementName($ident.name, $name));
        }
    };
}

impl TryFrom<Element> for Decal {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        check_name!(value is "decal");
        let color_string = get_as!(value["color"]: String or "ffffff".into());
        if !matches!(color_string.len(), 6 | 8)
            || color_string.chars().any(|c| !c.is_ascii_hexdigit())
        {
            return Err(LoadError::InvalidFieldData("color", color_string));
        }

        let mut color_buf = [0xFF_u8; 4];
        for (i, chunk) in color_string.chars().chunks(2).into_iter().enumerate() {
            let byte = chunk.collect::<String>();
            color_buf[i] = u8::from_str_radix(&byte, 16).expect("hex string was already validated")
        }

        Ok(Self {
            position: (
                get_as!(value["x"]: Float or 0.),
                get_as!(value["y"]: Float or 0.),
            ),
            scale: (
                get_as!(value["scaleX"]: Float or 1.),
                get_as!(value["scaleY"]: Float or 1.),
            ),
            texture: get_as!(value["texture"]: String or "".into()),
            depth: get_as!(value["depth"]: Integer or 0),
            rotation: get_as!(value["rotation"]: Float or 0.),
            color: color_buf,
        })
    }
}

impl From<Decal> for Element {
    fn from(value: Decal) -> Self {
        Self {
            name: "decal".into(),
            attributes: attributes! {
                "x" => value.position.0,
                "y" => value.position.1,
                "scaleX" => value.scale.0,
                "scaleY" => value.scale.1,
                "texture" => value.texture,
                "rotation" if value.rotation != 0.0 => value.rotation,
                "depth" if value.depth != 0 => value.depth,
                "color" if value.color != [0xFF; 4] =>
                    value.color
                        .into_iter()
                        .map(|v| format!("{v:02x}"))
                        .join("")
            },
            children: vec![],
        }
    }
}

impl TryFrom<Element> for Filler {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        check_name!(value is "rect");
        Ok(Self {
            position: (
                get_as!(value["x"]: Integer or 0),
                get_as!(value["y"]: Integer or 0),
            ),
            size: (
                get_as!(value["w"]: Integer or 0),
                get_as!(value["h"]: Integer or 0),
            ),
        })
    }
}

impl From<Filler> for Element {
    fn from(value: Filler) -> Self {
        Self {
            name: "rect".into(),
            attributes: attributes! {
                "x" => value.position.0,
                "y" => value.position.1,
                "w" => value.size.0,
                "h" => value.size.1
            },
            children: vec![],
        }
    }
}

impl TryFrom<Element> for LevelData {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        let music_progress_str = get_as!(value["musicProgress"]: String or String::new());
        let music_progress: Option<i32> = (!music_progress_str.is_empty())
            .then(|| music_progress_str.parse())
            .transpose()
            .map_err(|_| LoadError::InvalidFieldData("musicProgress", music_progress_str))?;

        let ambience_progress_str = get_as!(value["ambienceProgress"]: String or String::new());
        let ambience_progress: Option<i32> = (!ambience_progress_str.is_empty())
            .then(|| ambience_progress_str.parse())
            .transpose()
            .map_err(|_| LoadError::InvalidFieldData("ambienceProgress", ambience_progress_str))?;

        let wind_pattern_str = get_as!(value["windPattern"]: String or "None".into());
        let wind_pattern =
            (wind_pattern_str != "None").then(|| WindPattern::from(wind_pattern_str));

        Ok(Self {
            position: (
                get_as!(value["x"]: Integer or 0),
                get_as!(value["y"]: Integer or 0)
            ),
            size: (
                get_as!(value["width"]: Integer or 8), // Celeste has 8-pixel tiles
                get_as!(value["height"]: Integer or 8)
            ),
            music_layers: [
                get_as!(value["musicLayer1"]: Boolean or false),
                get_as!(value["musicLayer2"]: Boolean or false),
                get_as!(value["musicLayer3"]: Boolean or false),
                get_as!(value["musicLayer4"]: Boolean or false),
            ],
            underwater: get_as!(value["underwater"]: Boolean or false),
            space: get_as!(value["space"]: Boolean or false),
            disable_down_transition: get_as!(value["disableDownTransition"]: Boolean or false),
            music_progress,
            camera_offset: (
                get_as!(value["cameraOffsetX"]: Integer or 0),
                get_as!(value["cameraOffsetY"]: Integer or 0)
            ),
            wind_pattern,
            ambience_progress,
            alt_music: get_as!(value["alt_music"]: String or String::new()),
            ambience: get_as!(value["ambience"]: String or String::new()),
            delay_alt_music_fade: get_as!(value["delayAltMusicFade"]: Boolean or false),
            music: get_as!(value["music"]: String or String::new()),
            color: get_as!(value["c"]: Integer or 0),
            dark: get_as!(value["dark"]: Boolean or false),
            enforce_dash_number: get_as!(value["enforceDashNumber"]: Integer),
            whisper: get_as!(value["whisper"]: Boolean or false),
        })
    }
}

impl From<LevelData> for Element {
    
}