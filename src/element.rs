use std::{borrow::Cow, collections::HashMap, fmt::Write, io};

use byteorder::{LittleEndian, ReadBytesExt as _, WriteBytesExt};
use indexmap::IndexSet;
use itertools::Itertools as _;
use indent_write::fmt::IndentWriter;

use crate::{ext::{ReadExt as _, WriteExt as _}, LoadError};

/// A value that can appear in the attributes of an element.
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    /// A boolean value.
    Boolean(bool),
    /// An integer.
    Integer(i32),
    /// A floating-point value.
    Float(f32),
    /// A string.
    String(String),
    /// A string, specifically written in run-length encoding.
    RleString(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean(arg0) => write!(f, "{arg0}"),
            Self::Integer(arg0) => write!(f, "{arg0}"),
            Self::Float(arg0) => write!(f, "{arg0}"),
            Self::String(arg0)
                | Self::RleString(arg0)
                => write!(f, "{arg0:?}"),
        }
    }
}

impl Value {
    pub(crate) fn decode(stream: &mut dyn io::Read, lookup: &Vec<String>) -> Result<Self, LoadError> {
        Ok( match stream.read_u8()? {
            0 => (stream.read_u8()? > 0).into(), // Boolean value
            1 => (stream.read_u8()? as i32).into(),
            2 => (stream.read_i16::<LittleEndian>()? as i32).into(),
            3 => stream.read_i32::<LittleEndian>()?.into(),
            4 => stream.read_f32::<LittleEndian>()?.into(),
            5 => stream.lookup_string(lookup)?.to_string().into(),
            6 => stream.read_string()?.into(),
            7 => Self::RleString(stream.read_rle_string()?),
            invalid => Err(LoadError::InvalidValueType(invalid))?
        })
    }

    pub(crate) fn encode(self, stream: &mut dyn io::Write, lookup: &mut IndexSet<String>) -> io::Result<()> {
        match self {
            Value::Boolean(bool) => stream.write_all(&[0, bool as u8]),
            Value::Integer(int) => 
                if let Ok(byte) = u8::try_from(int) {
                    stream.write_all(&[1, byte])
                } else if let Ok(short) = i16::try_from(int) {
                    stream.write_u8(2)?;
                    stream.write_i16::<LittleEndian>(short)
                } else {
                    stream.write_u8(3)?;
                    stream.write_i32::<LittleEndian>(int)
                },
            Value::Float(float) => {
                stream.write_u8(4)?;
                stream.write_f32::<LittleEndian>(float)
            },
            Value::String(str) => {
                // Arbitrary cutoff for strings that are 
                // too long to likely be repeated
                // e.g. tilemaps
                if str.len() >= 64 {
                    stream.write_u8(6)?;
                    return stream.write_string(&str);
                }
                let (index, _) = lookup.insert_full(str);
                if index > u16::MAX as usize {
                    // Since we're out of space, write normally
                    stream.write_u8(6)?;
                    return stream.write_string(
                        lookup.get_index(index).unwrap()
                    );
                }
                stream.write_u8(5)?;
                stream.write_u16::<LittleEndian>(index as u16)
            },
            Value::RleString(str) => {
                stream.write_u8(7)?;
                stream.write_rle_string(&str)
            }
        }
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
    String => String
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
    pub(crate) fn decode(stream: &mut dyn io::Read, lookup: &Vec<String>) -> Result<Element, LoadError> {
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

    pub(crate) fn encode(self, stream: &mut dyn io::Write, lookup: &mut IndexSet<String>) -> io::Result<()> {
        let name_index = u16::try_from(lookup.insert_full(self.name).0)
            .map_err(|_| io::Error::other("cannot store more than 65535 unique strings"))?;
        stream.write_u16::<LittleEndian>(name_index)?;
        let attr_count = u8::try_from(self.attributes.len())
            .map_err(|_| io::Error::other("cannot have more than 255 attributes on an element"))?;
        stream.write_u8(attr_count)?;

        for (name, value) in self.attributes {
            let name_index = u16::try_from(lookup.insert_full(name).0)
                .map_err(|_| io::Error::other("cannot store more than 65535 unique strings"))?;
            stream.write_u16::<LittleEndian>(name_index)?;
            value.encode(stream, lookup)?;
        }

        let child_count = u16::try_from(self.children.len())
            .map_err(|_| io::Error::other("element cannot have more than 65535 children"))?;

        stream.write_u16::<LittleEndian>(child_count)?;

        for child in self.children {
            child.encode(stream, lookup)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, mut f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}", self.name)?;

        let mut inner_text = None;
        for (name, value) in self.attributes.iter().sorted_unstable_by(
            |(a, _), (b, _)| a.cmp(b)
        ) {
            if name == "innerText" {
                let value = match value {
                    Value::String(value) 
                        | Value::RleString(value) 
                        => Cow::Borrowed(value),
                    other => Cow::Owned(format!("{other}"))
                };
                inner_text = Some(value);
                continue;
            }
            write!(f, " {name}={value}")?;
        }
        if self.children.len() == 0 && inner_text.is_none() {
            return write!(f, " />");
        }

        writeln!(f, ">")?;

        {
            let mut indented_f = IndentWriter::new("\t", &mut f);
            for child in self.children.iter().sorted_unstable_by_key(|child| &child.name) {
                writeln!(indented_f, "{child}")?;
            }
            if let Some(inner) = inner_text {
                indented_f.write_str(inner.as_str())?;
                writeln!(indented_f)?;
            }
        }

        write!(f, "</{}>", self.name)
    }
}