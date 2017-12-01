use std::slice;
use cgmath::Vector2;

pub struct CoordIter {
    size: Vector2<i16>,
    coord: Vector2<i16>,
}

impl CoordIter {
    pub fn new(size: Vector2<u16>) -> Self {
        Self {
            size: size.cast(),
            coord: Vector2::new(0, 0),
        }
    }
}

impl Iterator for CoordIter {
    type Item = Vector2<i16>;
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
    type Item = (Vector2<i16>, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        self.coords.next().and_then(|c| {
            self.iter.next().map(|t| (c, t))
        })
    }
}

pub struct CoordEnumerateMut<'a, T: 'a> {
    coords: CoordIter,
    iter: slice::IterMut<'a, T>,
}

impl<'a, T> CoordEnumerateMut<'a, T> {
    pub(crate) fn new(coords: CoordIter, iter: slice::IterMut<'a, T>) -> Self {
        Self { coords, iter }
    }
}

impl<'a, T> Iterator for CoordEnumerateMut<'a, T> {
    type Item = (Vector2<i16>, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        self.coords.next().and_then(|c| {
            self.iter.next().map(|t| (c, t))
        })
    }
}
