use std::io::{self, Cursor, Seek};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;

use crate::LoadError;

pub trait ReadExt {
    fn read_rle_string(&mut self) -> io::Result<String>;
    fn lookup_string<'arr>(&mut self, arr: &'arr Vec<String>) -> Result<&'arr str, LoadError>;
    fn read_string(&mut self) -> io::Result<String>;
    fn read_variable_length_int(&mut self) -> io::Result<usize>;
}

pub trait WriteExt {
    fn write_rle_string(&mut self, str: &str) -> io::Result<()>;
    fn write_string(&mut self, str: &str) -> io::Result<()>;
    fn write_variable_length_int(&mut self, int: usize) -> io::Result<()>;
}

impl<T: io::Read + ?Sized> ReadExt for T {
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
                return Err(io::Error::new(io::ErrorKind::InvalidData, format!("variable length integer is too large for target architecture")))
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
                let times = self.read_u8()?;
                let chr = self.read_u8()?;
                Ok::<_, io::Error>(std::iter::repeat(chr).take(times as usize))
            }).process_results(|iter| {
                String::from_utf8(iter.flatten().collect::<Vec<u8>>())
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.utf8_error()))
            })?
    }
}

impl<T: io::Write + ?Sized> WriteExt for T {
    fn write_rle_string(&mut self, str: &str) -> io::Result<()> {
        let mut buf = Cursor::new(Vec::<u8>::new());
        let mut last_byte = None;
        let mut last_run = 1u8;
        
        for byte in str.bytes() {
            if let Some(last) = last_byte {
                if last == byte && last_run != u8::MAX {
                    last_run += 1;
                } else {
                    buf.write_u8(last_run)?;
                    buf.write_u8(last)?;
                    last_run = 1;
                }
            }
            last_byte = Some(byte);
        }
        if let Some(byte) = last_byte {
            buf.write_u8(last_run)?;
            buf.write_u8(byte)?;
        }
        let Ok(length) = u16::try_from(buf.stream_position()?)
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "string is too large to be stored with run-length encoding"
            ))
        };

        self.write_u16::<LittleEndian>(length)?;
        self.write_all(buf.get_ref())
    }
    
    fn write_string(&mut self, str: &str) -> io::Result<()> {
        self.write_variable_length_int(str.len())?;
        self.write_all(str.as_bytes())
    }
    
    fn write_variable_length_int(&mut self, mut int: usize) -> io::Result<()> {
        loop {
            let cont = int > 0b0111_1111;
            self.write_u8(
                ((cont as u8) << 7) | (int as u8 & 0b0111_1111)
            )?;
            int >>= 7;
            if !cont { break Ok(()); }
        }
    }
}