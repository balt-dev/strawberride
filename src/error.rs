use std::{fmt, io, string::FromUtf8Error};


#[derive(Debug)]
#[non_exhaustive]
/// Something that can go wrong when loading a map.
pub enum LoadError {
    /// The map header was invalid.
    InvalidHeader(String),
    /// An IO error occurred.
    IoError(io::Error),
    /// A string was outside the range of the lookup table for strings.
    InvalidString(usize),
    /// A value of an element was of an invalid type.
    InvalidValueType(u8),
    /// An element was missing a required value.
    MissingElement(&'static str),
    /// An element was of a type not valid for its field.
    InvalidFieldType(&'static str, crate::Value),
    /// An element had a field that was invalid.
    InvalidFieldData(&'static str, String),
    /// An element had an unexpected name for its location.
    InvalidElementName(String, &'static str)
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::InvalidHeader(header) =>
                write!(f, "invalid file header: {header:?}"),
            LoadError::IoError(err) =>
                write!(f, "io error: {err}"),
            LoadError::InvalidString(index) =>
                write!(f, "out-of-bounds index into string table: {index}"),
            LoadError::InvalidValueType(ty) =>
                write!(f, "invalid value type: {ty}"),
            LoadError::MissingElement(name) =>
                write!(f, "element was missing required value: {name:?}"),
            LoadError::InvalidFieldType(name, value) =>
                write!(f, "element field {name:?} was of an invalid type: {value:?}"),
            LoadError::InvalidFieldData(name, value) =>
                write!(f, "element field {name:?} had malformed data: {value}"),
            LoadError::InvalidElementName(value, expected) =>
                write!(f, "found unexpected element {value:?} when looking for elements of name {expected:?}")
         }
    }
}

impl std::error::Error for LoadError {}

impl From<io::Error> for LoadError {
    fn from(value: io::Error) -> Self {
        LoadError::IoError(value)
    }
}

impl From<FromUtf8Error> for LoadError {
    fn from(err: FromUtf8Error) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, err.utf8_error()).into()
    }
}

impl LoadError {
    /// Tries to clone the [`LoadError`], returning [`None`] if it cannot be cloned.
    pub fn try_clone(&self) -> Option<Self> {
        Some( match self {
            LoadError::InvalidHeader(h) => LoadError::InvalidHeader(h.clone()),
            LoadError::InvalidString(s) => LoadError::InvalidString(*s),
            LoadError::InvalidValueType(t) => LoadError::InvalidValueType(*t),
            LoadError::MissingElement(el) => LoadError::MissingElement(*el),
            LoadError::InvalidFieldType(name, v) => LoadError::InvalidFieldType(*name, v.clone()),
            LoadError::InvalidFieldData(name, d) => LoadError::InvalidFieldData(*name, d.clone()),
            LoadError::InvalidElementName(name, expected) => LoadError::InvalidElementName(name.clone(), *expected),

            LoadError::IoError(_) => return None
        } )
    }
}