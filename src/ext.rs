use std::io;
use byteorder::{NativeEndian, LittleEndian, ReadBytesExt};
use itertools::Itertools;

use crate::LoadError;


pub trait ReadExt {
    fn read_rle_string(&mut self) -> io::Result<String>;
    fn lookup_string<'arr>(&mut self, arr: &'arr Vec<String>) -> Result<&'arr str, LoadError>;
    fn read_string(&mut self) -> io::Result<String>;
    fn read_variable_length_int(&mut self) -> io::Result<usize>;

}

impl ReadExt for dyn io::Read + '_ {
    /// Gets a variably-long integer from the file.
    fn read_variable_length_int(&mut self) -> io::Result<usize> {
        let mut result = 0;
        let mut length = 0;
        loop {
            let byte = self.read_u8()?;
            result += ((byte & 0b0111_1111) as usize) << (length * 7);
            length += 1;
            if byte & 0b1000_0000 == 0 {
                return Ok(result);
            }
            if length * 7 >= usize::BITS {
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("Variable length integer is too large for target architecture")))
            }
        }
    }

    /// Reads a variable-length string from the file.
    fn read_string(&mut self) -> io::Result<String> {
        let length = self.read_variable_length_int()?;
        let mut buf = vec![0u8; length];
        self.read_exact(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|err| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("string is not valid utf-8: {}", String::from_utf8_lossy(err.as_bytes()))
            ))
    }

    /// Grabs a string from the given array from an index in the file.
    fn lookup_string<'arr>(&mut self, arr: &'arr Vec<String>) -> Result<&'arr str, LoadError> {
        let index = self.read_u16::<LittleEndian>()? as usize;
        arr.get(index)
            .map(String::as_str)
            .ok_or(LoadError::InvalidString(index))
    }

    /// Reads a run-length encoded string.
    fn read_rle_string(&mut self) -> io::Result<String> {
        let size = self.read_u16::<LittleEndian>()? as usize;
        (0..(size / 2))
            .map(|_| {
                let [times, char] = self.read_u16::<NativeEndian>()?.to_ne_bytes();
                Ok::<_, io::Error>(std::iter::repeat(char).take(times as usize))
            }).process_results(|iter| {
                String::from_utf8(iter.flatten().collect::<Vec<u8>>())
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.utf8_error()))
            })?
    }
}