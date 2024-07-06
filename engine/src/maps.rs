use std::{
    fmt::Display,
    ops::{Add, Index, IndexMut},
};

use rand::{
    distributions::Uniform,
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use crate::pieces::Piece;

pub const DEFAULT_MAP_WIDTH: u16 = 100;
pub const DEFAULT_MAP_HEIGHT: u16 = 60;

const MAX_HEIGHT: u16 = 999;

#[derive(PartialEq)]
pub enum Terrain {
    Water,
    Land,
    Unknown,
}

impl Display for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Terrain::Land => '+',
            Terrain::Water => '.',
            Terrain::Unknown => ' ',
        };
        write!(f, "{c}")?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Position {
    x: i16,
    y: i16,
}

impl Position {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Self) -> usize {
        isqrt(((other.x - self.x) ^ 2 + (other.y - self.y) ^ 2) as usize)
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

pub struct Location {
    pos: Position,
    terrain: Terrain,
    piece: Option<Piece>,
}

impl Location {
    fn new(pos: Position, terrain: Terrain) -> Self {
        Self {
            pos,
            terrain,
            piece: None,
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.piece {
            Some(piece) => piece.fmt(f)?,
            None => self.terrain.fmt(f)?,
        };
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Grid<T> {
    width: u16,
    height: u16,
    map: Vec<T>,
}

impl<T> Grid<T>
where
    T: Clone + Default,
{
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            map: vec![T::default(); (width * height) as usize],
        }
    }
}

impl<T> Grid<T> {
    fn covers(&self, pos: Position) -> bool {
        (pos.x >= 0 && pos.x < self.width as i16) && (pos.y >= 0 && pos.y < self.height as i16)
    }
}

impl<'g, T> Grid<T> {
    pub fn neighbours(&'g self, pos: Position) -> NeighbourIter<'g, T> {
        NeighbourIter::<'g, T> {
            grid: &self,
            pos,
            dirs: vec![
                Position { x: -1, y: -1 },
                Position { x: 0, y: -1 },
                Position { x: 1, y: -1 },
                Position { x: -1, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: -1, y: 1 },
                Position { x: 0, y: 1 },
                Position { x: 1, y: 1 },
            ],
            idx: 0,
        }
    }
}

impl Grid<u16> {
    pub fn new_random(width: u16, height: u16) -> Self {
        let rng = rand::thread_rng();

        let mut grid = Grid::<u16>::new(width, height);
        grid.map = rng
            .sample_iter(Uniform::from(0..MAX_HEIGHT))
            .take(grid.map.capacity())
            .collect();
        grid
    }

    pub fn smooth(self) -> Self {
        let mut new_map = self.map.clone();
        for idx in 0..new_map.len() {
            let pos = idx_to_pos(idx, self.width);
            new_map[idx] = (self.neighbours(pos).sum::<u16>() + self[pos])
                / (self.neighbours(pos).count() + 1) as u16;
        }

        Self {
            width: self.width,
            height: self.height,
            map: new_map,
        }
    }

    fn water_height(&self, ratio: u16) -> u16 {
        for h in 0..MAX_HEIGHT {
            let below = self.map.iter().filter(|level| **level <= h).count();
            let above = self.map.iter().filter(|level| **level > h).count();
            if below * 100 / (above + below) > ratio as usize {
                return h;
            }
        }
        MAX_HEIGHT
    }

    pub fn make_terrain(self, water: u16) -> Grid<Location> {
        let wh = self.water_height(water);
        Grid {
            width: self.width,
            height: self.height,
            map: self
                .map
                .iter()
                .enumerate()
                .map(|(idx, level)| {
                    if *level <= wh {
                        Location::new(idx_to_pos(idx, self.width), Terrain::Water)
                    } else {
                        Location::new(idx_to_pos(idx, self.width), Terrain::Land)
                    }
                })
                .collect(),
        }
    }
}

impl Grid<Location> {
    pub fn place_cities(&mut self) {
        let (city_idx, min_city_dist) = {
            let city_num = ((100 * (self.width + self.height)) / 228) as usize;

            let city_idx: Vec<_> = (0..self.map.len())
                .filter(|idx| self.map[*idx].terrain == Terrain::Land)
                .choose_multiple(
                    &mut rand::thread_rng(),
                    ((100 * (self.width + self.height)) / 228) as usize,
                )
                .into_iter()
                .map(|idx| idx_to_pos(idx, self.width))
                .collect();

            let land = self
                .map
                .iter()
                .filter(|l| l.terrain == Terrain::Land)
                .count()
                / city_num;
            (city_idx, isqrt(land))
        };

        for pos in city_idx {
            self.put_piece(Piece::City, pos);
        }
    }

    fn put_piece(&mut self, piece: Piece, pos: Position) {
        self[pos].piece = Some(piece);
    }

    fn remove_piece(&mut self, pos: Position) {
        self[pos].piece = None;
    }
}

impl<T> Index<Position> for Grid<T> {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        &self.map[pos_to_idx(index, self.width)]
    }
}

impl<T> IndexMut<Position> for Grid<T> {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.map[pos_to_idx(index, self.width)]
    }
}

impl<T> Display for Grid<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for h in 0..self.height {
            for w in 0..self.width {
                write!(
                    f,
                    "{}",
                    self[Position {
                        x: w as i16,
                        y: h as i16
                    }]
                )?
            }
            write!(f, "\n")?
        }
        Ok(())
    }
}

pub struct NeighbourIter<'g, T> {
    grid: &'g Grid<T>,
    pos: Position,
    dirs: Vec<Position>,
    idx: usize,
}

impl<'g, T> Iterator for NeighbourIter<'g, T> {
    type Item = &'g T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.dirs.len() {
            return None;
        }

        loop {
            let n_pos = self.pos + self.dirs[self.idx];
            self.idx += 1;

            if self.grid.covers(n_pos) {
                return Some(&self.grid[n_pos]);
            }

            if self.idx >= self.dirs.len() {
                return None;
            }
        }
    }
}

fn pos_to_idx(pos: Position, width: u16) -> usize {
    (pos.y * width as i16 + pos.x) as usize
}

fn idx_to_pos(idx: usize, width: u16) -> Position {
    let y = idx as u16 / width;
    Position {
        x: (idx - (y * width) as usize) as i16,
        y: y as i16,
    }
}

// See https://en.wikipedia.org/wiki/Integer_square_root
fn isqrt(val: usize) -> usize {
    let mut left = 0;
    let mut mid = 0;
    let mut right = val + 1;

    while left != right - 1 {
        mid = (left + right) / 2;

        if mid * mid <= val {
            left = mid;
        } else {
            right = mid;
        }
    }
    left
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_grid() {
        assert_eq!(
            Grid::new(10, 20),
            Grid::<u16> {
                width: 10,
                height: 20,
                map: vec![0; 200]
            }
        );
    }

    #[test]
    fn test_grid_index() {
        let mut grid = Grid::<i32>::new(DEFAULT_MAP_WIDTH, DEFAULT_MAP_HEIGHT);
        let pos = Position { x: 5, y: 5 };
        grid[pos] = 1;
        assert!(grid[pos] == 1);
    }

    #[test]
    fn test_add_position() {
        let p1 = Position { x: 1, y: 2 };
        let p2 = Position { x: -1, y: 3 };
        assert_eq!(p1 + p2, Position { x: 0, y: 5 })
    }

    #[test]
    fn test_neighbours() {
        let mut grid = Grid::<u16>::new(10, 10);
        grid[Position { x: 4, y: 4 }] = 1;
        grid[Position { x: 5, y: 4 }] = 2;
        grid[Position { x: 6, y: 4 }] = 3;
        grid[Position { x: 4, y: 5 }] = 4;
        grid[Position { x: 5, y: 5 }] = 5;
        grid[Position { x: 6, y: 5 }] = 6;
        grid[Position { x: 4, y: 6 }] = 7;
        grid[Position { x: 5, y: 6 }] = 8;
        grid[Position { x: 6, y: 6 }] = 9;

        let mut nbrs = grid.neighbours(Position { x: 5, y: 5 });
        assert_eq!(nbrs.next(), Some(&1));
        assert_eq!(nbrs.next(), Some(&2));
        assert_eq!(nbrs.next(), Some(&3));
        assert_eq!(nbrs.next(), Some(&4));
        assert_eq!(nbrs.next(), Some(&6));
        assert_eq!(nbrs.next(), Some(&7));
        assert_eq!(nbrs.next(), Some(&8));
        assert_eq!(nbrs.next(), Some(&9));
        assert_eq!(nbrs.next(), None);
    }

    #[test]
    fn test_neighbours_nw_corner() {
        let mut grid = Grid::<u16>::new(10, 10);
        grid[Position { x: 0, y: 0 }] = 1;
        grid[Position { x: 1, y: 0 }] = 2;
        grid[Position { x: 0, y: 1 }] = 3;
        grid[Position { x: 1, y: 1 }] = 4;

        let mut nbrs = grid.neighbours(Position { x: 0, y: 0 });
        assert_eq!(nbrs.next(), Some(&2));
        assert_eq!(nbrs.next(), Some(&3));
        assert_eq!(nbrs.next(), Some(&4));
        assert_eq!(nbrs.next(), None);
    }

    #[test]
    fn test_neighbours_ne_corner() {
        let mut grid = Grid::<u16>::new(10, 10);
        grid[Position { x: 8, y: 0 }] = 1;
        grid[Position { x: 9, y: 0 }] = 2;
        grid[Position { x: 8, y: 1 }] = 3;
        grid[Position { x: 9, y: 1 }] = 4;

        let mut nbrs = grid.neighbours(Position { x: 9, y: 0 });
        assert_eq!(nbrs.next(), Some(&1));
        assert_eq!(nbrs.next(), Some(&3));
        assert_eq!(nbrs.next(), Some(&4));
        assert_eq!(nbrs.next(), None);
    }

    #[test]
    fn test_neighbours_sw_corner() {
        let mut grid = Grid::<u16>::new(10, 10);
        grid[Position { x: 0, y: 8 }] = 1;
        grid[Position { x: 1, y: 8 }] = 2;
        grid[Position { x: 0, y: 9 }] = 3;
        grid[Position { x: 1, y: 9 }] = 4;

        let mut nbrs = grid.neighbours(Position { x: 0, y: 9 });
        assert_eq!(nbrs.next(), Some(&1));
        assert_eq!(nbrs.next(), Some(&2));
        assert_eq!(nbrs.next(), Some(&4));
        assert_eq!(nbrs.next(), None);
    }

    #[test]
    fn test_neighbours_se_corner() {
        let mut grid = Grid::<u16>::new(10, 10);
        grid[Position { x: 8, y: 8 }] = 1;
        grid[Position { x: 9, y: 8 }] = 2;
        grid[Position { x: 8, y: 9 }] = 3;
        grid[Position { x: 9, y: 9 }] = 4;

        let mut nbrs = grid.neighbours(Position { x: 9, y: 9 });
        assert_eq!(nbrs.next(), Some(&1));
        assert_eq!(nbrs.next(), Some(&2));
        assert_eq!(nbrs.next(), Some(&3));
        assert_eq!(nbrs.next(), None);
    }

    #[test]
    fn test_idx_to_pos() {
        assert_eq!(idx_to_pos(25, 10), Position { x: 5, y: 2 });
    }
}
