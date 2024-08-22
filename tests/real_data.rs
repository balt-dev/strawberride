use std::{error::Error, io::{Cursor, Seek}};

use strawberride::Element;

static TEST_MAP: &[u8] = include_bytes!("blank.bin");

#[test]
fn parse_real_data() -> Result<(), Box<dyn Error>> {
    let mut cur = Cursor::new(TEST_MAP);
    let map = Element::load_map(&mut cur, true)?;
    dbg!(map);
    Ok(())
}