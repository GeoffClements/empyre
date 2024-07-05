fn main() {
    let mut grid = engine::Grid::new_random(10, 6);
    println!("{grid}");

    grid = grid.smooth();
    println!("{grid}");
    grid = grid.smooth();
    println!("{grid}");
    grid = grid.smooth();
    println!("{grid}");
    grid = grid.smooth();
    println!("{grid}");
}
