use crate::position::Position;
use crate::square::Rank;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// The direction pawns move.
    pub fn pawn_dir(self) -> i8 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }

    /// Is this white?
    pub fn is_white(self) -> bool {
        self == Color::White
    }

    /// If the provided character is lowercase, return black.
    pub fn of_char(char: char) -> Self {
        if char.is_lowercase() {
            Color::Black
        } else {
            Color::White
        }
    }

    /// The back rank for the color, a.k.a. the
    /// rank on which the king and rooks start.
    pub fn back_rank(self) -> Rank {
        match self {
            Self::White => Rank::_1,
            Self::Black => Rank::_8,
        }
    }

    /// 'w' for white, 'b' for black.
    pub fn to_char(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
