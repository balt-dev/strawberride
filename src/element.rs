use std::{collections::HashMap, io::Read};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt as _};
use itertools::Itertools as _;

use crate::{ext::ReadExt as _, LoadError};

/// A value that can appear in the attributes of an element.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i32),
    Float(f32),
    String(String),
    Map(HashMap<String, Value>)
}

impl Value {
    pub(crate) fn decode(stream: &mut dyn Read, lookup: &Vec<String>) -> Result<Self, LoadError> {
        Ok( match stream.read_u8()? {
            0 => (stream.read_u8()? > 0).into(), // Boolean value
            1 => (stream.read_u8()? as i32).into(),
            2 => (stream.read_i16::<LittleEndian>()? as i32).into(),
            3 => stream.read_i32::<LittleEndian>()?.into(),
            4 => stream.read_f32::<BigEndian>()?.into(),
            5 => stream.lookup_string(lookup)?.to_string().into(),
            6 => stream.read_string()?.into(),
            7 => stream.read_rle_string()?.into(),
            invalid => Err(LoadError::InvalidValueType(invalid))?
        })
    }
}

macro_rules! value_from_type_impl {
    ($($ty: ty => $name: ident),*) => {$(
        impl From<$ty> for Value {
            fn from(value: $ty) -> Value {
                Self::$name(value)
            }
        }
    )*};
}

value_from_type_impl! {
    bool => Boolean,
    i32 => Integer,
    f32 => Float,
    String => String,
    HashMap<String, Value> => Map
}

/// An element of a map.
#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// The element's name.
    pub name: String,
    /// The element's attributes.
    pub attributes: HashMap<String, Value>,
    /// The element's child elements.
    pub children: Vec<Element>
}

impl Element {
    pub(crate) fn decode(stream: &mut dyn Read, lookup: &Vec<String>) -> Result<Element, LoadError> {
        let name = stream.lookup_string(lookup)?.to_string();

        let attr_count = stream.read_u8()?;
        let mut attributes = HashMap::with_capacity(attr_count as usize);
        (0..attr_count).map(|_| {
            let key = stream.lookup_string(lookup)?.to_string();
            let value = Value::decode(stream, lookup)?;

            Ok::<_, LoadError>((key, value))
        }).process_results(|iter| attributes.extend(iter))?;

        let child_count = stream.read_u16::<LittleEndian>()?;
        let children = (0 .. child_count)
            .map(|_| Element::decode(stream, lookup))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Element {
            name,
            attributes,
            children
        })
    }
}
