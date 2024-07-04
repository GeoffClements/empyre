use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use rand::{distributions::Uniform, Rng};

const DEFAULT_MAP_WIDTH: u16 = 100;
const DEFAULT_MAP_HEIGHT: u16 = 60;

const MAX_HEIGHT: u16 = 999;

#[derive(Clone, Copy)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(Debug, PartialEq)]
pub struct Grid<T> {
    width: u16,
    height: u16,
    map: Vec<T>,
}

impl<T> Grid<T> {
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            map: Vec::<T>::with_capacity((width * height) as usize),
        }
    }

    // fn is_in(&self, pos: Position) -> bool {
    //     let idx = pos.x * pos.y;
    //     idx >= 0 && idx as usize <= self.map.len()
    // }
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
}

impl<T> Default for Grid<T>
where
    T: Clone + Default,
{
    fn default() -> Self {
        Self {
            width: DEFAULT_MAP_WIDTH,
            height: DEFAULT_MAP_HEIGHT,
            map: vec![T::default(); (DEFAULT_MAP_WIDTH * DEFAULT_MAP_HEIGHT) as usize],
        }
    }
}

impl<T> Index<Position> for Grid<T> {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        &self.map[(index.x * self.width as i16 + index.y) as usize]
    }
}

impl<T> IndexMut<Position> for Grid<T> {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self.map[(index.x * self.width as i16 + index.y) as usize]
    }
}

impl<T> Display for Grid<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for w in 0..self.width {
            for h in 0..self.height {
                write!(
                    f,
                    "{},",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid() {
        assert_eq!(
            Grid::new(10, 20),
            Grid::<u16> {
                width: 10,
                height: 20,
                map: Vec::new()
            }
        );
    }

    #[test]
    fn default_grid() {
        assert_eq!(
            Grid::default(),
            Grid::<i32> {
                width: DEFAULT_MAP_WIDTH,
                height: DEFAULT_MAP_HEIGHT,
                map: vec![<i32>::default(); (DEFAULT_MAP_WIDTH * DEFAULT_MAP_HEIGHT) as usize]
            }
        )
    }

    #[test]
    fn grid_index() {
        let mut grid = Grid::<i32>::default();
        let pos = Position { x: 5, y: 5 };
        grid[pos] = 1;
        assert!(grid[pos] == 1);
    }
}
