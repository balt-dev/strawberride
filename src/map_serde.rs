use std::collections::HashMap;

use itertools::Itertools as _;

use crate::{
    Decal, Element, Entity, Filler, Level, LevelData, LoadError, Map, Tilemap, Value
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
impl MapElement for Entity {}
impl MapElement for Decal {}

macro_rules! remove_as {
    ($el: ident [ $field_name: literal ]: String or $default: expr) => {
        if let Some(field) = $el.attributes.remove($field_name) {
            if let Value::String(res) = field {
                res
            } else if let Value::RleString(res) = field {
                res
            } else {
                return Err(LoadError::InvalidFieldType($field_name, field));
            }
        } else {
            $default
        }
    };
    ($el: ident [ $field_name: literal ]: Float or $default: expr) => {
        if let Some(field) = $el.attributes.remove($field_name) {
            if let Value::Float(res) = field {
                res
            } else if let Value::Integer(res) = field {
                res as f32
            } else {
                return Err(LoadError::InvalidFieldType($field_name, field));
            }
        } else {
            $default
        }
    };
    ($el: ident [ $field_name: literal ]: Integer or $default: expr) => {
        if let Some(field) = $el.attributes.remove($field_name) {
            if let Value::Integer(res) = field {
                res
            } else if let Value::Float(res) = field {
                res as i32
            } else {
                return Err(LoadError::InvalidFieldType($field_name, field));
            }
        } else {
            $default
        }
    };
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

/// Syntactic sugar for defining the attributes of an [`Element`].
#[macro_export]
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

fn parse_color(color_string: String) -> Result<[u8; 4], LoadError> {
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

    Ok(color_buf)
}

impl TryFrom<Element> for Decal {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        check_name!(value is "decal");
        let color_string = remove_as!(value["color"]: String or "ffffff".into());

        Ok(Self {
            position: (
                remove_as!(value["x"]: Float or 0.),
                remove_as!(value["y"]: Float or 0.),
            ),
            scale: (
                remove_as!(value["scaleX"]: Float or 1.),
                remove_as!(value["scaleY"]: Float or 1.),
            ),
            texture: remove_as!(value["texture"]: String or "".into()),
            depth: remove_as!(value["depth"]: Integer or 0),
            rotation: remove_as!(value["rotation"]: Float or 0.),
            color: parse_color(color_string)?,
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
                remove_as!(value["x"]: Integer or 0),
                remove_as!(value["y"]: Integer or 0),
            ),
            size: (
                remove_as!(value["w"]: Integer or 0),
                remove_as!(value["h"]: Integer or 0),
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

impl LevelData {
    fn load_from(value: &mut Element) -> Result<Self, LoadError> {
        let music_progress_str = remove_as!(value["musicProgress"]: String or String::new());
        let music_progress: Option<i32> = (!music_progress_str.is_empty())
            .then(|| music_progress_str.parse())
            .transpose()
            .map_err(|_| LoadError::InvalidFieldData("musicProgress", music_progress_str))?;

        let ambience_progress_str = remove_as!(value["ambienceProgress"]: String or String::new());
        let ambience_progress: Option<i32> = (!ambience_progress_str.is_empty())
            .then(|| ambience_progress_str.parse())
            .transpose()
            .map_err(|_| LoadError::InvalidFieldData("ambienceProgress", ambience_progress_str))?;

        Ok(Self {
            position: (
                remove_as!(value["x"]: Integer or 0),
                remove_as!(value["y"]: Integer or 0)
            ),
            size: (
                remove_as!(value["width"]: Integer or 8), // Celeste has 8-pixel tiles
                remove_as!(value["height"]: Integer or 8)
            ),
            music_layers: [
                remove_as!(value["musicLayer1"]: Boolean or false),
                remove_as!(value["musicLayer2"]: Boolean or false),
                remove_as!(value["musicLayer3"]: Boolean or false),
                remove_as!(value["musicLayer4"]: Boolean or false),
            ],
            underwater: remove_as!(value["underwater"]: Boolean or false),
            space: remove_as!(value["space"]: Boolean or false),
            disable_down_transition: remove_as!(value["disableDownTransition"]: Boolean or false),
            music_progress,
            camera_offset: (
                remove_as!(value["cameraOffsetX"]: Integer or 0),
                remove_as!(value["cameraOffsetY"]: Integer or 0)
            ),
            wind_pattern: remove_as!(value["windPattern"]: String or "None".into()),
            ambience_progress,
            alt_music: remove_as!(value["alt_music"]: String or String::new()),
            ambience: remove_as!(value["ambience"]: String or String::new()),
            delay_alt_music_fade: remove_as!(value["delayAltMusicFade"]: Boolean or false),
            music: remove_as!(value["music"]: String or String::new()),
            color: remove_as!(value["c"]: Integer or 0),
            dark: remove_as!(value["dark"]: Boolean or false),
            enforce_dash_number: remove_as!(value["enforceDashNumber"]: Integer),
            whisper: remove_as!(value["whisper"]: Boolean or false),
        })
    }

    fn store_to(self, el: &mut Element) {
        el.attributes.extend(attributes! {
            "x" => self.position.0,
            "y" => self.position.1,
            "width" => self.size.0,
            "height" => self.size.1,
            // OBOE's, anyone?
            "musicLayer1" => self.music_layers[0],
            "musicLayer2" => self.music_layers[1],
            "musicLayer3" => self.music_layers[2],
            "musicLayer4" => self.music_layers[3],
            "underwater" => self.underwater,
            "space" => self.space,
            "disableDownTransition" => self.disable_down_transition,
            "musicProgress" => self.music_progress.map_or(String::new(), |v| v.to_string()),
            "cameraOffsetX" => self.camera_offset.0,
            "cameraOffsetY" => self.camera_offset.1,
            "windPattern" => self.wind_pattern,
            "ambienceProgress" => self.ambience_progress.map_or(String::new(), |v| v.to_string()),
            "alt_music" => self.alt_music,
            "ambience" => self.ambience,
            "delayAltMusicFade" => self.delay_alt_music_fade,
            "music" => self.music,
            "c" => self.color,
            "dark" => self.dark,
            "enforceDashNumber" if self.enforce_dash_number.is_some() => self.enforce_dash_number.unwrap(),
            "whisper" => self.whisper
        })
    }
}

impl TryFrom<Element> for Entity {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        let id = remove_as!(value["id"]: Integer or 0);
        let position = (
            remove_as!(value["x"]: Float or 0.),
            remove_as!(value["y"]: Float or 0.)
        );
        let width = remove_as!(value["width"]: Integer);
        let height = remove_as!(value["height"]: Integer);
        let origin = (
            remove_as!(value["originX"]: Float or 0.),
            remove_as!(value["originY"]: Float or 0.)
        );

        let nodes = value.children
            .into_iter()
            .map(|mut el| {
                check_name!(el is "node");
                let x = remove_as!(el["x"]: Float or 0.);
                let y = remove_as!(el["y"]: Float or 0.);
                Ok((x, y))
            }).collect::<Result<_, _>>()?;

        Ok(Self {
            name: value.name,
            id, position, width, height, origin,
            nodes,
            values: value.attributes // The other ones were already removed from remove_as!()
        })
    }
}

impl From<Entity> for Element {
    fn from(value: Entity) -> Element {
        let mut attrs = value.values;
        attrs.extend(attributes! {
            "x" => value.position.0,
            "y" => value.position.1,
            "width" if value.width.is_some() => value.width.unwrap(),
            "height" if value.height.is_some() => value.height.unwrap(),
            "originX" => value.origin.0,
            "originY" => value.origin.1,
            "id" => value.id
        });

        Element {
            name: value.name,
            attributes: attrs,
            children: value.nodes.into_iter().map(|(x, y)| Element {
                name: "node".into(),
                attributes: attributes! {
                    "x" => x,
                    "y" => y
                },
                children: vec![]
            }).collect()
        }
    }
}

impl TryFrom<Element> for Level {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        check_name!(value is "level");
        let data = LevelData::load_from(&mut value)?;
        if data.size.0 < 0 {
            return Err(LoadError::InvalidFieldData("width", "width cannot be negative".into()))
        }
        if data.size.0 < 0 {
            return Err(LoadError::InvalidFieldData("height", "height cannot be negative".into()))
        }
        let tile_width = (data.size.0 / 8) as usize;
        let tile_height = (data.size.1 / 8) as usize;
        let name = remove_as!(value["name"]: String or "<unnamed>".into());
        
        let mut entities = vec![];
        let mut triggers = vec![];
        let mut bg_decals = vec![];
        let mut fg_decals = vec![];

        let mut bg = Tilemap::new(tile_width, tile_height).ok_or(
            LoadError::InvalidFieldData("bg", "tilemap size is too large for this machine to store in memory".into())
        )?;
        // Past this point we unwrap because we know the tilemap fits
        let mut bg_tiles = Tilemap::new(tile_width, tile_height).unwrap();
        let mut fg_tiles = Tilemap::new(tile_width, tile_height).unwrap();
        let mut obj_tiles = Tilemap::new(tile_width, tile_height).unwrap();
        let mut solids = Tilemap::new(tile_width, tile_height).unwrap();

        let mut extra_children = Vec::new();
        for mut child in value.children {
            match child.name.as_str() {
                "entities" => 
                    entities = child.children.into_iter()
                        .map(Entity::try_from)
                        .collect::<Result<_, _>>()?,
                "triggers" => 
                    triggers = child.children.into_iter()
                        .map(Entity::try_from)
                        .collect::<Result<_, _>>()?,
                "bgdecals" => 
                    bg_decals = child.children.into_iter()
                        .map(Decal::try_from)
                        .collect::<Result<_, _>>()?,
                "fgdecals" => 
                    fg_decals = child.children.into_iter()
                        .map(Decal::try_from)
                        .collect::<Result<_, _>>()?,
                "bg" => 
                    bg = Tilemap::<char>::load(
                        remove_as!(child["innerText"]: String or String::new()), 
                        tile_width, tile_height
                    ).unwrap(),
                "bgtiles" => bg_tiles = Tilemap::<i32>::load(
                    remove_as!(child["innerText"]: String or String::new()), 
                    tile_width, tile_height
                ).unwrap(),
                "fgtiles" => fg_tiles = Tilemap::<i32>::load(
                    remove_as!(child["innerText"]: String or String::new()), 
                    tile_width, tile_height
                ).unwrap(),
                "solids" => solids = Tilemap::<char>::load(
                    remove_as!(child["innerText"]: String or String::new()), 
                    tile_width, tile_height
                ).unwrap(),
                "objtiles" => obj_tiles = Tilemap::<i32>::load(
                    remove_as!(child["innerText"]: String or String::new()), 
                    tile_width, tile_height
                ).unwrap(),
                _ => extra_children.push(child)
            }
        }

        Ok(Level {
            name, data, entities, triggers, bg_decals, fg_decals,
            bg, bg_tiles, fg_tiles, obj_tiles, solids,
            extra_children, 
            extra_data: value.attributes
        })
    }
}

impl From<Level> for Element {
    fn from(value: Level) -> Self {
        let mut el = Element {
            name: "level".into(),
            attributes: value.extra_data,
            children: value.extra_children
        };
        value.data.store_to(&mut el);
        el.attributes.extend(attributes! {
            "name" => value.name
        });

        el.children.extend([
            Element {
                name: "entities".into(),
                attributes: HashMap::new(),
                children: value.entities.into_iter()
                    .map(Into::into)
                    .collect()
            },
            Element {
                name: "triggers".into(),
                attributes: HashMap::new(),
                children: value.triggers.into_iter()
                    .map(Into::into)
                    .collect()
            },
            Element {
                name: "bgdecals".into(),
                attributes: HashMap::new(),
                children: value.bg_decals.into_iter()
                    .map(Into::into)
                    .collect()
            },
            Element {
                name: "fgdecals".into(),
                attributes: HashMap::new(),
                children: value.fg_decals.into_iter()
                    .map(Into::into)
                    .collect()
            },
            Element {
                name: "bg".into(),
                attributes: attributes!{ "innerText" => Value::RleString(value.bg.store()) },
                children: vec![]
            },
            Element {
                name: "bgtiles".into(),
                attributes: attributes!{ "innerText" => value.bg_tiles.store() },
                children: vec![]
            },
            Element {
                name: "fgtiles".into(),
                attributes: attributes!{ "innerText" => value.fg_tiles.store() },
                children: vec![]
            },
            Element {
                name: "objtiles".into(),
                attributes: attributes!{ "innerText" => value.obj_tiles.store() },
                children: vec![]
            },
            Element {
                name: "solids".into(),
                attributes: attributes!{ "innerText" => Value::RleString(value.solids.store()) },
                children: vec![]
            },
        ]);

        el
    }
}

impl TryFrom<Element> for Map {
    type Error = LoadError;

    fn try_from(mut value: Element) -> Result<Self, Self::Error> {
        check_name!(value is "Map");
        
        let package = remove_as!(value["_package"]: String or String::new());
        let mut bg_color = None;
        let mut filler = Vec::new();
        let mut foregrounds = Vec::new();
        let mut backgrounds = Vec::new();
        let mut levels = Vec::new();

        let mut extra_children = Vec::new();

        for mut child in value.children {
            match child.name.as_str() {
                "Filler" => 
                    filler = child.children.into_iter()
                        .map(Filler::try_from)
                        .collect::<Result<_, _>>()?,
                "Style" => {
                    bg_color = remove_as!(child["color"]: String)
                        .map(parse_color)
                        .transpose()?;
                    for grandchild in child.children {
                        match grandchild.name.as_str() {
                            "Foregrounds" =>
                                foregrounds = grandchild.children,
                            "Backgrounds" =>
                                backgrounds = grandchild.children,
                            _ => {}
                        }
                    }
                },
                "levels" =>
                    levels = child.children.into_iter()
                    .map(Level::try_from)
                    .collect::<Result<_, _>>()?,
                _ => extra_children.push(child)
            }
        }

        Ok(Map {
            package, filler, levels, foregrounds, backgrounds, 
            bg_color, extra_data: value.attributes, extra_children
        })
    }
}

impl From<Map> for Element {
    fn from(value: Map) -> Self {
        let mut children = value.extra_children;
        let mut attributes = value.extra_data;
        attributes.insert("_package".into(), value.package.into());

        children.push(Element {
            name: "Filler".into(),
            attributes: HashMap::new(),
            children: value.filler.into_iter()
                .map(Into::into)
                .collect()
        });

        children.push(Element {
            name: "Style".into(),
            attributes: {
                let mut attrs = HashMap::new();
                if let Some(col) = value.bg_color {
                    attrs.insert("color".into(), col
                        .into_iter()
                        .map(|v| format!("{v:02x}"))
                        .join("")
                        .into()
                    );
                }
                attrs
            },
            children: vec![
                Element {
                    name: "Foregrounds".into(),
                    attributes: HashMap::new(),
                    children: value.foregrounds.into_iter()
                        .map(Into::into)
                        .collect()
                },
                Element {
                    name: "Backgrounds".into(),
                    attributes: HashMap::new(),
                    children: value.backgrounds.into_iter()
                        .map(Into::into)
                        .collect()
                }
            ]
        });

        children.push(Element {
            name: "levels".into(),
            attributes: HashMap::new(),
            children: value.levels
                .into_iter()
                .map(Into::into)
                .collect()
        });

        Element {
            name: "Map".into(), attributes, children
        }
    }
}