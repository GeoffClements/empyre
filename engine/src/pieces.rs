use std::fmt::Display;

pub enum Piece {
    City
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "O")?;
        Ok(())
    }
}