extern crate serde;
#[macro_use] extern crate serde_derive;

/// General purpose coordinate for use within prototty crates.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl<T: Into<[i32; 2]>> From<T> for Coord {
    fn from(t: T) -> Self {
        let array = t.into();
        Coord::new(array[0], array[1])
    }
}

/// Used to describe the size of a prototty view, and of a prototty context, in cells.
/// A size cannot be created which would contain un-addressable cells.
/// That is, the maximum size has a width and height of one greater than the maximum `i32`.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Size {
    x: u32,
    y: u32,
}

impl Size {
    /// Creates a new `Size`.
    /// Panics if `x` or `y` is greater than `::std::i32::MAX as u32 + 1`.
    pub fn new(x: u32, y: u32) -> Self {
        const SIZE_MAX: u32 = ::std::i32::MAX as u32 + 1;
        if x > SIZE_MAX || y > SIZE_MAX {
            panic!("Size is too big: ({}, {})", x, y);
        }
        Self { x, y }
    }

    /// Returns the width.
    pub fn x(&self) -> u32 { self.x }

    /// Returns the height.
    pub fn y(&self) -> u32 { self.y }

    /// Returns an iterator over all the coordinates within
    /// a rectangle of this size.
    pub fn coords(&self) -> CoordIter {
        CoordIter::new(*self)
    }

    /// Suppose an array is used to implement a 2D grid of this size,
    /// where traversing the array from start to end is equivalent
    /// to traversing the 2D grid top to bottom, traversing left
    /// to right within each row. If a given coordinate is valid
    /// for such a grid, this function returns the index into the
    /// array corresponding to that coordinate.
    pub fn index(&self, coord: Coord) -> Option<usize> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }

        let x = coord.x as u32;
        let y = coord.y as u32;

        if x >= self.x || y >= self.y {
            return None;
        }

        Some((y * self.x + x) as usize)
    }

    /// Return the number of cells in a 2D grid of this size.
    pub fn count(&self) -> usize {
        (self.x * self.y) as usize
    }
}

impl<T: Into<[u32; 2]>> From<T> for Size {
    fn from(t: T) -> Self {
        let array = t.into();
        Size::new(array[0], array[1])
    }
}

impl ::std::ops::Add for Coord {
    type Output = Coord;
    fn add(self, rhs: Coord) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ::std::ops::Add for Size {
    type Output = Size;
    fn add(self, rhs: Size) -> Self::Output {
        Size::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl ::std::ops::Add<Size> for Coord {
    type Output = Coord;
    fn add(self, rhs: Size) -> Self::Output {
        Coord {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
        }
    }
}

impl ::std::ops::Add<Coord> for Size {
    type Output = Coord;
    fn add(self, rhs: Coord) -> Self::Output {
        rhs + self
    }
}

impl ::std::ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, rhs: Coord) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ::std::ops::Sub for Size {
    type Output = Size;
    fn sub(self, rhs: Size) -> Self::Output {
        Size::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl ::std::ops::Sub<Size> for Coord {
    type Output = Coord;
    fn sub(self, rhs: Size) -> Self::Output {
        Coord {
            x: self.x - rhs.x as i32,
            y: self.y - rhs.y as i32,
        }
    }
}

impl ::std::ops::Sub<Coord> for Size {
    type Output = Coord;
    fn sub(self, rhs: Coord) -> Self::Output {
        rhs - self
    }
}

/// Iterates over all the coordinates in a grid from
/// top to bottom, and left to right within each row.
pub struct CoordIter {
    size: Size,
    coord: Coord,
}

impl CoordIter {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            coord: Coord::new(0, 0),
        }
    }
}

impl Iterator for CoordIter {
    type Item = Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.coord.y as u32 == self.size.y {
            return None;
        }

        let coord = self.coord;

        self.coord.x += 1;
        if self.coord.x as u32 == self.size.x {
            self.coord.x = 0;
            self.coord.y += 1;
        }

        Some(coord)
    }
}
