#[cfg(target_pointer_width = "16")]
compile_error!("tilemaps cannot properly function when usize is less than 32 bytes long");

use std::{cmp::Ordering, collections::HashMap, iter, ops::{Index, IndexMut}};
use itertools::Itertools;
use crate::{Element, LoadError, Value};

mod seal {
    pub trait TilemapCell: Copy {
        const EMPTY: Self;
        const SEPARATOR: &'static str;
    }
    
    impl TilemapCell for i32 {
        const EMPTY: Self = -1;
        const SEPARATOR: &'static str = ", ";
    }
    
    impl TilemapCell for char {
        const EMPTY: Self = '0';
        const SEPARATOR: &'static str = "";
    }
}

use seal::TilemapCell;

use crate::MapElement;

#[derive(Clone, PartialEq, Eq, Default, Hash)]
/// A 2-dimensional tilemap.

// Safety contracts:
// width * height <= usize::MAX
// width * height == data.len()
pub struct Tilemap<T: TilemapCell> {
    width: usize,
    height: usize,
    data: Vec<T>
}

impl<T: TilemapCell + std::fmt::Display> std::fmt::Debug for Tilemap<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sep = if f.alternate() {"\n\t"} else {" "};
        write!(f, "Tilemap {{{sep}")?;
        write!(f, "width: {},{sep}", self.width)?;
        write!(f, "height: {},{sep}", self.height)?;
        if !f.alternate() {
            write!(f, "data: ... }}")?;
            return Ok(());
        }
        write!(f, "data:")?;
        for mut row in self.data.iter().copied().chunks(self.width).into_iter() {
            write!(f, "\n\t\t{}", row.join(T::SEPARATOR))?;
        }
        
        write!(f, "\n}}")
    }
} 

impl<T: TilemapCell> Tilemap<T> {
    /// Creates a new tilemap of the given width and height, initialized with empty values.
    /// 
    /// Will return [`None`] if the width and height cannot be multiplied as [`usize`]s without arithmetic overflow.
    pub fn new(width: usize, height: usize) -> Option<Self> {
        (height).checked_mul(width)
            .map(|size| 
                Self {
                    width, height,
                    data: iter::repeat(T::EMPTY)
                        .take(size)
                        .collect()
                }
            )
    }

    /// Gets the width of the tilemap.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Gets the height of the tilemap.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Sets the width of the tilemap, padding or truncating the underlying data as needed.
    /// 
    /// Will not change the width if the new area is greater than [`usize::MAX`]. In this case, the function will return `false`.
    pub fn set_width(&mut self, new_width: usize) -> bool {
        let Some(_) = self.height.checked_mul(new_width) else { return false };

        match self.width.cmp(&new_width) {
            Ordering::Equal => (),
            Ordering::Less =>
                // Pad out the width
                self.data = self.data.iter()
                    .copied()
                    .chunks(self.width)
                    .into_iter()
                    .flat_map(|chunk| chunk.chain(
                        iter::repeat(T::EMPTY).take(new_width - self.width)
                    ))
                    .collect(),
            Ordering::Greater =>
                self.data = self.data.iter()
                    .copied()
                    .chunks(self.width)
                    .into_iter()
                    .flat_map(|chunk| chunk.take(new_width))
                    .collect()
        }
        self.width = new_width;
        true
    }

    /// Sets the height of the tilemap, padding or truncating the underlying data as needed.
    /// 
    /// Will not change the height if the new area is greater than [`usize::MAX`]. In this case, the function will return `false`.
    pub fn set_height(&mut self, new_height: usize) -> bool {
        let Some(new_area) = (self.width).checked_mul(new_height) else { return false };

        match self.height.cmp(&new_height) {
            Ordering::Equal => (),
            Ordering::Less => self.data = self.data.iter()
                .copied()
                .chain(iter::repeat(T::EMPTY).take(
                    self.width * new_height // cannot overflow due to above
                ))
                .collect(),
            Ordering::Greater => self.data.truncate(new_area)
        }

        true
    }

    /// Gets a reference to the cell at the index, returning [`None`] if out of bounds or multiplication would overflow.
    // Overflow checking is held by the safety contracts on Tilemap.
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        (
            (0 .. self.width).contains(&x)
            && (0 .. self.height).contains(&y)
        ).then(|| unsafe {
            self.get_unchecked(x, y)
        })
    }

    /// Gets a mutable reference to the cell at the index, returning [`None`] if out of bounds or multiplication would overflow.
    // Overflow checking is held by the safety contracts on Tilemap.
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        (
            (0 .. self.width).contains(&x)
            && (0 .. self.height).contains(&y)
        ).then(|| unsafe {
            self.get_unchecked_mut(x, y)
        })
    }

    /// Gets a reference to the value in the cell at the index, without checking boundaries.
    /// 
    /// # Safety
    /// The cell index must be within the bounds of the tilemap, and `self.width * y + x` must not overflow a usize.
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> &T {
        self.data.get_unchecked(self.width * y + x)
    }

    /// Gets a mutable reference to the cell at the index, without checking boundaries.
    /// 
    /// # Safety
    /// The cell index must be within the bounds of the tilemap, and `self.width * y + x` must not overflow a usize.
    pub unsafe fn get_unchecked_mut(&mut self, x: usize, y: usize) -> &mut T {
        self.data.get_unchecked_mut(self.width * y + x)
    }

    /// Gets a reference to the underlying raw data of the tilemap.
    pub fn raw_data(&self) -> &[T] {
        &self.data
    }

    /// Gets a mutable reference to the underlying raw data of the tilemap.
    /// 
    /// # Safety
    /// The slice must not be shrunk below its initial length.
    pub unsafe fn raw_data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

impl<T: TilemapCell> Index<(usize, usize)> for Tilemap<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        self.get(index.0, index.1).expect("index out of bounds for tilemap")
    }
}

impl<T: TilemapCell> IndexMut<(usize, usize)> for Tilemap<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        self.get_mut(index.0, index.1).expect("index out of bounds for tilemap")
    }
}

impl Tilemap<char> {
    pub(crate) fn load(s: String, width: usize, height: usize) -> Option<Self> {
        let mut map = Self::new(width, height)?;
        for (y, line) in s.lines().enumerate() {
            for (x, chr) in line.chars().enumerate() {
                if let Some(addr) = map.get_mut(x, y) {
                    *addr = chr;
                }
            }
        }
        
        Some(map)
    }

    pub(crate) fn store(&self) -> String {
        let mut rows = Vec::with_capacity(self.height);
        for row in self.data.chunks_exact(self.width) {
            let mut buf = String::with_capacity(self.width);
            let mut last_run = 0;
            for char in row.iter().copied() {
                if char == char::EMPTY {
                    last_run += 1;
                    continue;
                } else if last_run > 0 {
                    buf.extend(iter::repeat(char::EMPTY).take(last_run));
                    last_run = 0;
                }
                buf.push(char);
            }
            rows.push(buf);
        }
        rows.join("\n").into()
    }
}

impl Tilemap<i32> {
    pub(crate) fn load(s: String, width: usize, height: usize) -> Option<Self> {
        let mut map = Self::new(width, height)?;
        for (y, line) in s.lines().enumerate() {
            for (x, id) in line.split(',').map(|v| v.trim().parse()).enumerate() {
                if let Some(addr) = map.get_mut(x, y) {
                    // This needs to be pretty damn resilient. Customs can be quite broken sometimes.
                    *addr = id.unwrap_or(-1);
                }
            }
        }
        
        Some(map)
    }

    pub(crate) fn store(&self) -> String {
        let mut rows = Vec::with_capacity(self.height);
        for row in self.data.chunks_exact(self.width) {
            let mut line_buf = Vec::with_capacity(self.width);
            let mut last_run = 0;
            for id in row.iter().copied() {
                if id == i32::EMPTY {
                    last_run += 1;
                    continue;
                } else if last_run > 0 {
                    line_buf.extend(
                        iter::repeat(i32::EMPTY).take(last_run)
                    );
                    last_run = 0;
                }
                line_buf.push(id);
            }
            rows.push(
                line_buf.into_iter()
                    .map(|v| v.to_string())
                    .join(",")
            )
        }

        rows.into_iter().join("\n")
    }
}