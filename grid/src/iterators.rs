use std::slice;
use prototty_traits::*;

pub struct CoordIter {
    size: Coord,
    coord: Coord,
}

impl CoordIter {
    pub fn new(size: Size) -> Self {
        Self {
            size: size.cast().unwrap(),
            coord: Coord::new(0, 0),
        }
    }
}

impl Iterator for CoordIter {
    type Item = Coord;
    fn next(&mut self) -> Option<Self::Item> {
        if self.coord.y == self.size.y {
            return None;
        }

        let coord = self.coord;

        self.coord.x += 1;
        if self.coord.x == self.size.x {
            self.coord.x = 0;
            self.coord.y += 1;
        }

        Some(coord)
    }
}

pub struct CoordEnumerate<'a, T: 'a> {
    coords: CoordIter,
    iter: slice::Iter<'a, T>,
}

impl<'a, T> CoordEnumerate<'a, T> {
    pub(crate) fn new(coords: CoordIter, iter: slice::Iter<'a, T>) -> Self {
        Self { coords, iter }
    }
}

impl<'a, T> Iterator for CoordEnumerate<'a, T> {
    type Item = (Coord, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.coords.next().and_then(|c| {
            self.iter.next().map(|t| (c, t))
        })
    }
}
