use clap::Parser;
use engine::{Grid, DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH};

#[derive(Parser)]
#[command(name = "empyre", author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'w',
        default_value = "70",
        help = "Must be in range 0..90",
        value_parser = clap::value_parser!(u16).range(0..=90))]
    water: Option<u16>,

    #[arg(
        short = 's',
        default_value = "5",
        help = "Must be greater or equal to zero"
    )]
    smooth: Option<u16>,
}

fn main() {
    let cli = Cli::parse();
    let mut grid = Grid::new_random(DEFAULT_MAP_WIDTH, DEFAULT_MAP_HEIGHT);

    for _ in 0..cli.smooth.unwrap() {
        grid = grid.smooth();
    }

    let mut map = grid.make_terrain(cli.water.unwrap());
    map.place_cities();
    println!("{map}");
}
