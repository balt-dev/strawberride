use std::{
    error::Error,
    io::{Cursor, Seek, SeekFrom}
};

use strawberride::Map;

static TEST_MAP: &[u8] = include_bytes!(r"9D.bin");

#[test]
fn round_trip_real_data() -> Result<(), Box<dyn Error>> {
    let mut cur = Cursor::new(TEST_MAP);
    let map = Map::load(&mut cur, true)?;
    let mut buf = Cursor::new(Vec::new());
    map.clone().store(&mut buf, true)?;
    buf.seek(SeekFrom::Start(0))?;
    let same_map = Map::load(&mut buf, true)?;
    if map != same_map {
        // assert_eq prints out both structs, which prints like 8 MB of text to the console
        panic!("round trip equality failed")
    }
    Ok(())
}