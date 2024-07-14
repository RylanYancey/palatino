use crate::bitmask::Bitmask;
use std::mem::transmute;

pub use definitions::*;

#[rustfmt::skip]
mod definitions {
    /// A single column in the board grid.
    /// A = 0, G = 7.
    #[repr(u8)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum File {
        A=0, B, C, D, E, F, G, H
    }

    /// A single row in the board grid.
    /// _1 = 0, _8 = 7.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub enum Rank {
        _1, _2, _3, _4, _5, _6, _7, _8
    }

    /// A single square in the board grid.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd)]
    pub enum Square {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5, 
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8,
    }
}

impl File {
    /// Create a new file, representing a column of cells vertically.
    /// Panics of the number is greater than 7, or less than 0.
    pub fn new(idx: u8) -> Self {
        Self::try_idx(idx).unwrap()
    }

    /// Attempt to convert a number to a column of cells vertically.
    pub fn try_idx(idx: u8) -> Option<Self> {
        // Rust doesn't give us a way to convert u8 to enum for some reason, so transmute.
        if idx > 8 {
            None
        } else {
            Some(unsafe { transmute(idx) })
        }
    }

    /// Iterate all variants of the File enum from File::A to File::H.
    pub fn iter() -> impl DoubleEndedIterator<Item = Self> {
        (0..8).map(|i| Self::try_idx(i).unwrap())
    }

    /// Conver the file to a lowercase character.
    pub fn to_char_lower(&self) -> char {
        match self {
            File::A => 'a',
            File::B => 'b',
            File::C => 'c',
            File::D => 'd',
            File::E => 'e',
            File::F => 'f',
            File::G => 'g',
            File::H => 'h',
        }
    }

    /// Convert the file to an uppercase character.
    pub fn to_char_upper(&self) -> char {
        match self {
            File::A => 'A',
            File::B => 'B',
            File::C => 'C',
            File::D => 'D',
            File::E => 'E',
            File::F => 'F',
            File::G => 'G',
            File::H => 'H',
        }
    }

    /// Convert a character to a file, case-agnostic.
    pub fn from_char(char: char) -> Option<Self> {
        let id = match char.to_ascii_lowercase() {
            'a' => File::A,
            'b' => File::B,
            'c' => File::C,
            'd' => File::D,
            'e' => File::E,
            'f' => File::F,
            'g' => File::G,
            'h' => File::H,
            _ => return None,
        };

        Some(id)
    }
}

impl Rank {
    /// Create a new rank, representing a row of cells horizontally.
    /// Panics if the number is greater than 7, or less than 0.
    pub fn new(idx: u8) -> Self {
        Self::try_idx(idx).unwrap()
    }

    /// Attempt to convert a number to a row of cells horizontally.
    pub fn try_idx(idx: u8) -> Option<Self> {
        // Rust doesn't give us a way to convert u8 to enum for some reason, so transmute.
        if idx > 8 {
            None
        } else {
            Some(unsafe { transmute(idx) })
        }
    }

    /// Iterate all variants of the Rank enum from Rank::_1, to Rank::_8.
    pub fn iter() -> impl DoubleEndedIterator<Item = Self> {
        (0..8).map(|i| Self::try_idx(i).unwrap())
    }

    pub fn from_char(char: char) -> Option<Self> {
        let c = match char {
            '1' => Rank::_1,
            '2' => Rank::_2,
            '3' => Rank::_3,
            '4' => Rank::_4,
            '5' => Rank::_5,
            '6' => Rank::_6,
            '7' => Rank::_7,
            '8' => Rank::_8,
            _ => return None,
        };

        Some(c)
    }

    /// Convert this rank to a character.
    pub fn to_char(&self) -> char {
        match self {
            Rank::_1 => '1',
            Rank::_2 => '2',
            Rank::_3 => '3',
            Rank::_4 => '4',
            Rank::_5 => '5',
            Rank::_6 => '6',
            Rank::_7 => '7',
            Rank::_8 => '8',
        }
    }
}

impl Square {
    /// Try to create a new Square, representing a single cell on the board grid.
    pub fn try_new(file: u8, rank: u8) -> Option<Self> {
        Some(Self::new(File::try_idx(file)?, Rank::try_idx(rank)?))
    }

    /// Create a new Square, representing a single cell on the board grid.
    /// The result will be File + Rank, so File::A and Rank::_1 will become Square::A1.
    pub fn new(file: File, rank: Rank) -> Self {
        // Functionally equivalent to rank * 8 + file.
        // Only works because the width of the board is 8, a power of 2.
        // The first 3 bits indicate the file, and the last 3 indicate the rank.
        Self::try_idx(((rank as u8) << 3) | file as u8).unwrap()
    }

    /// Attempt to convert a number to a grid cell.
    pub const fn try_idx(idx: u8) -> Option<Self> {
        // Rust doesn't give us a way to convert u8 to enum for some reason, so transmute.
        if idx > 63 {
            None
        } else {
            Some(unsafe { transmute(idx) })
        }
    }

    /// The rank should remain the same, but change the file.
    pub fn with_file(self, file: File) -> Self {
        Self::new(file, self.rank())
    }

    /// The file should remain the same, but change the file.
    pub fn with_rank(self, rank: Rank) -> Self {
        Self::new(self.file(), rank)
    }

    /// Convert a square to a lowercase string.
    pub fn to_string_lower(&self) -> String {
        format!("{}{}", self.file().to_char_lower(), self.rank().to_char())
    }

    /// get the diagonal edge in some direction.
    pub fn diag_edge(self, dir: (i8, i8)) -> Self {
        let sf = if dir.0 == -1 {
            self.file() as i8
        } else {
            7 - self.file() as i8
        };
        let sr = if dir.1 == -1 {
            self.rank() as i8
        } else {
            7 - self.rank() as i8
        };

        let df = i8::min(sf, sr);
        let dr = i8::min(sf, sr);

        self.try_offset(df * dir.0, dr * dir.1).unwrap()
    }

    /// Iterate all possible squares from Square::A1 to Square::H8.
    pub fn iter() -> impl DoubleEndedIterator<Item = Self> {
        (0..64).map(|i| Self::try_idx(i).unwrap())
    }

    /// Get the Lettered Column this square belongs to.
    pub fn file(self) -> File {
        // The first 3 bits indicate the file.
        File::new(self as u8 & 0b000111)
    }

    /// Get the Numbered Row this square belongs to.
    pub fn rank(self) -> Rank {
        // The last 3 bits indicate the rank.
        Rank::new(self as u8 >> 3)
    }

    /// Attempt to offset the square by some amount, returning None if it is not possible.
    pub fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        Some(Square::new(
            File::try_idx((self.file() as i8 + file_offset).try_into().ok()?)?,
            Rank::try_idx((self.rank() as i8 + rank_offset).try_into().ok()?)?,
        ))
    }

    /// Attempt to convert a string to a square, for example 'e4' or 'd5'.
    pub fn try_from_string(str: &str) -> Option<Self> {
        if str.len() == 2 {
            let c1 = str.as_bytes()[0];
            let c2 = str.as_bytes()[1];

            if let Some(rank) = Rank::from_char(c2 as char) {
                if let Some(file) = File::from_char(c1 as char) {
                    return Some(Square::new(file, rank));
                }
            }
        }

        None
    }

    /// Create a bitmask with a 1 bit at this square position.
    pub fn mask(self) -> Bitmask {
        // the relationship is 1:1 - we can just shift the bit into the board.
        Bitmask::from(self)
    }

    /// Returns true if self and other are on the same rank or same file.
    pub fn shares_orthogonal(self, other: Self) -> bool {
        self.file() == other.file() || self.rank() == other.rank()
    }

    /// Returns true if self and other share either the +x or -x diagonal.
    pub fn shares_diagonal(self, other: Self) -> bool {
        let (x1, y1, x2, y2) = (
            self.file() as i8,
            self.rank() as i8,
            other.file() as i8,
            other.rank() as i8,
        );
        (x1 - y1) == (x2 - y2) || (x1 - y2) == (x2 - y1)
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_lower())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_new() {
        assert_eq!(Rank::new(0), Rank::_1);
        assert_eq!(Rank::new(1), Rank::_2);
        assert_eq!(Rank::new(2), Rank::_3);
        assert_eq!(Rank::new(3), Rank::_4);
        assert_eq!(Rank::new(4), Rank::_5);
        assert_eq!(Rank::new(5), Rank::_6);
        assert_eq!(Rank::new(6), Rank::_7);
        assert_eq!(Rank::new(7), Rank::_8);
    }

    #[test]
    fn file_new() {
        assert_eq!(File::new(0), File::A);
        assert_eq!(File::new(1), File::B);
        assert_eq!(File::new(2), File::C);
        assert_eq!(File::new(3), File::D);
        assert_eq!(File::new(4), File::E);
        assert_eq!(File::new(5), File::F);
        assert_eq!(File::new(6), File::G);
        assert_eq!(File::new(7), File::H);
    }

    #[test]
    fn square_new() {
        assert_eq!(Square::new(File::A, Rank::_1), Square::A1);
        assert_eq!(Square::new(File::H, Rank::_8), Square::H8);
        assert_eq!(Square::new(File::A, Rank::_8), Square::A8);
        assert_eq!(Square::new(File::H, Rank::_1), Square::H1);
    }

    #[test]
    #[should_panic]
    fn rank_new_out_of_bounds() {
        Rank::new(8);
    }

    #[test]
    #[should_panic]
    fn file_new_out_of_bounds() {
        File::new(8);
    }

    #[test]
    fn rank_try_idx() {
        assert_eq!(Rank::try_idx(0).unwrap(), Rank::_1);
        assert_eq!(Rank::try_idx(1).unwrap(), Rank::_2);
        assert_eq!(Rank::try_idx(2).unwrap(), Rank::_3);
        assert_eq!(Rank::try_idx(3).unwrap(), Rank::_4);
        assert_eq!(Rank::try_idx(4).unwrap(), Rank::_5);
        assert_eq!(Rank::try_idx(5).unwrap(), Rank::_6);
        assert_eq!(Rank::try_idx(6).unwrap(), Rank::_7);
        assert_eq!(Rank::try_idx(7).unwrap(), Rank::_8);
    }

    #[test]
    fn file_try_idx() {
        assert_eq!(File::try_idx(0).unwrap(), File::A);
        assert_eq!(File::try_idx(1).unwrap(), File::B);
        assert_eq!(File::try_idx(2).unwrap(), File::C);
        assert_eq!(File::try_idx(3).unwrap(), File::D);
        assert_eq!(File::try_idx(4).unwrap(), File::E);
        assert_eq!(File::try_idx(5).unwrap(), File::F);
        assert_eq!(File::try_idx(6).unwrap(), File::G);
        assert_eq!(File::try_idx(7).unwrap(), File::H);
    }

    #[test]
    fn square_try_idx() {
        assert_eq!(Square::try_idx(0).unwrap(), Square::A1);
        assert_eq!(Square::try_idx(7).unwrap(), Square::H1);
        assert_eq!(Square::try_idx(56).unwrap(), Square::A8);
        assert_eq!(Square::try_idx(63).unwrap(), Square::H8);
    }

    #[test]
    fn rank_try_idx_out_of_bounds() {
        assert_eq!(Rank::try_idx(8), None);
    }

    #[test]
    fn file_try_idx_out_of_bounds() {
        assert_eq!(File::try_idx(8), None);
    }

    #[test]
    fn square_try_idx_out_of_bounds() {
        assert_eq!(Square::try_idx(64), None);
    }

    #[test]
    fn square_get_rank() {
        assert_eq!(Square::A1.rank(), Rank::_1);
        assert_eq!(Square::A2.rank(), Rank::_2);
        assert_eq!(Square::H1.rank(), Rank::_1);
        assert_eq!(Square::A8.rank(), Rank::_8);
        assert_eq!(Square::H8.rank(), Rank::_8);
    }

    #[test]
    fn square_get_file() {
        assert_eq!(Square::A1.file(), File::A);
        assert_eq!(Square::A2.file(), File::A);
        assert_eq!(Square::H1.file(), File::H);
        assert_eq!(Square::A8.file(), File::A);
        assert_eq!(Square::H8.file(), File::H);
    }

    #[test]
    fn square_try_offset_pos() {
        assert_eq!(Square::A1.try_offset(1, 1).unwrap(), Square::B2);
        assert_eq!(Square::C3.try_offset(1, 1).unwrap(), Square::D4);
        assert_eq!(Square::G7.try_offset(1, 1).unwrap(), Square::H8);
        assert_eq!(Square::H8.try_offset(1, 1), None);
    }

    #[test]
    fn square_try_offset_neg() {
        assert_eq!(Square::A1.try_offset(-1, -1), None);
        assert_eq!(Square::H8.try_offset(-1, -1).unwrap(), Square::G7);
    }

    #[test]
    fn square_share_orthogonal() {
        assert!(Square::A1.shares_orthogonal(Square::A8));
        assert!(Square::A1.shares_orthogonal(Square::H1));
        assert!(Square::D4.shares_orthogonal(Square::A4));
        assert!(Square::D4.shares_orthogonal(Square::D1));
        assert!(Square::H8.shares_orthogonal(Square::H1));
        assert!(Square::H8.shares_orthogonal(Square::A8));
        assert!(!Square::H8.shares_orthogonal(Square::G7));
        assert!(!Square::C6.shares_orthogonal(Square::D3));
    }

    #[test]
    fn square_share_diagonal() {
        assert!(Square::A1.shares_diagonal(Square::H8));
        assert!(Square::A8.shares_diagonal(Square::H1));
        assert!(Square::D4.shares_diagonal(Square::F6));
        assert!(Square::G1.shares_diagonal(Square::C5));
        assert!(Square::B3.shares_diagonal(Square::E6));
        assert!(!Square::H4.shares_diagonal(Square::C3));
        assert!(!Square::A6.shares_diagonal(Square::B6));
    }

    #[test]
    fn square_diag_edge() {
        // A1
        assert_eq!(Square::A1.diag_edge((1, 1)), Square::H8);
        assert_eq!(Square::A1.diag_edge((1, -1)), Square::A1);
        assert_eq!(Square::A1.diag_edge((-1, 1)), Square::A1);
        assert_eq!(Square::A1.diag_edge((-1, -1)), Square::A1);
        // H8
        assert_eq!(Square::H8.diag_edge((1, 1)), Square::H8);
        assert_eq!(Square::H8.diag_edge((1, -1)), Square::H8);
        assert_eq!(Square::H8.diag_edge((-1, 1)), Square::H8);
        assert_eq!(Square::H8.diag_edge((-1, -1)), Square::A1);
        // F3
        assert_eq!(Square::F3.diag_edge((1, 1)), Square::H5);
        assert_eq!(Square::F3.diag_edge((1, -1)), Square::H1);
        assert_eq!(Square::F3.diag_edge((-1, 1)), Square::A8);
        assert_eq!(Square::F3.diag_edge((-1, -1)), Square::D1);
        // E7
        assert_eq!(Square::E7.diag_edge((1, 1)), Square::F8);
        assert_eq!(Square::E7.diag_edge((1, -1)), Square::H4);
        assert_eq!(Square::E7.diag_edge((-1, 1)), Square::D8);
        assert_eq!(Square::E7.diag_edge((-1, -1)), Square::A3);
        // B8
        assert_eq!(Square::B8.diag_edge((1, 1)), Square::B8);
        assert_eq!(Square::B8.diag_edge((1, -1)), Square::H2);
        assert_eq!(Square::B8.diag_edge((-1, 1)), Square::B8);
        assert_eq!(Square::B8.diag_edge((-1, -1)), Square::A7);
    }
}
