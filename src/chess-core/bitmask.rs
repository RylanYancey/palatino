use std::ops::*;

use crate::square::{File, Rank, Square};

/// A 64-bit number representing a selection of squares on a board.
#[derive(Copy, Clone, PartialEq, Default, Hash)]
pub struct Bitmask(pub u64);

impl Bitmask {
    /// A bitmask with no squares set to 1.
    pub const EMPTY: Self = Self(0);

    // a constant for each rank set to 1s'

    pub const RANK1: Self = Self(0xFF);
    pub const RANK2: Self = Self(0xFF_00);
    pub const RANK3: Self = Self(0xFF_00_00);
    pub const RANK4: Self = Self(0xFF_00_00_00);
    pub const RANK5: Self = Self(0xFF_00_00_00_00);
    pub const RANK6: Self = Self(0xFF_00_00_00_00_00);
    pub const RANK7: Self = Self(0xFF_00_00_00_00_00_00);
    pub const RANK8: Self = Self(0xFF_00_00_00_00_00_00_00);

    // a constant for each file set to 1s'

    pub const FILEA: Self = Self(0x01_01_01_01_01_01_01_01);
    pub const FILEB: Self = Self(0x02_02_02_02_02_02_02_02);
    pub const FILEC: Self = Self(0x04_04_04_04_04_04_04_04);
    pub const FILED: Self = Self(0x08_08_08_08_08_08_08_08);
    pub const FILEE: Self = Self(0x10_10_10_10_10_10_10_10);
    pub const FILEF: Self = Self(0x20_20_20_20_20_20_20_20);
    pub const FILEG: Self = Self(0x40_40_40_40_40_40_40_40);
    pub const FILEH: Self = Self(0x80_80_80_80_80_80_80_80);

    /// Get the number of occupied bits in the mask.
    pub fn count(self) -> u8 {
        self.0.count_ones() as u8
    }

    /// Returns true if self==0
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Sets the bit at square to be 1, regardless of current state.
    pub fn with(self, square: Square) -> Self {
        self | square.mask()
    }

    /// Same as 'with', but assigns to self instead of returning a new value.
    pub fn set(&mut self, square: Square) {
        *self = *self | square.mask()
    }

    /// Swap the value at 'from' with 'to' and vice versa.
    pub fn swap(&mut self, from: Square, to: Square) {
        let has_fr = self.has(from);
        let has_to = self.has(to);

        if has_fr {
            self.set(to);
        } else {
            self.remove(to);
        }

        if has_to {
            self.set(from);
        } else {
            self.remove(from)
        }
    }

    /// Flips the bit at square, such that 1=0 and 0=1.
    pub fn flip(self, square: Square) -> Self {
        self ^ square.mask()
    }

    /// Set the bit at square to be 0, regardless of current state.
    pub fn without(self, square: Square) -> Self {
        self & !square.mask()
    }

    /// Same as 'without', but assigns to self instead of returning a new value.
    pub fn remove(&mut self, square: Square) {
        *self = *self & !square.mask()
    }

    /// Returns true if the bit at square is 1.
    pub fn has(self, square: Square) -> bool {
        self | square.mask() == self
    }

    /// The union of two bitmasks.
    /// The resulting bitmask has all the 1s of both self and other.
    pub fn union(self, other: Self) -> Self {
        self | other
    }

    /// The intersection of two bitboards.
    /// The resulting bitmask has all the 1s of self, without the 1s of other.
    pub fn intersection(self, other: Self) -> Self {
        self & !other
    }

    /// Returns true if self and other intersect (share any 1s)
    pub fn intersects(self, other: Self) -> bool {
        self.intersection(other) != self
    }

    /// Returns a bitmask of the intersection if they intersect at all.
    pub fn intersects_then(self, other: Self) -> Option<Bitmask> {
        let intersection = self.intersection(other);

        if intersection != self {
            Some(intersection)
        } else {
            None
        }
    }

    /// This bitmask, with all bits in the rank set to 1.
    pub fn with_rank(self, rank: Rank) -> Self {
        self | match rank {
            Rank::_1 => Self::RANK1,
            Rank::_2 => Self::RANK2,
            Rank::_3 => Self::RANK3,
            Rank::_4 => Self::RANK4,
            Rank::_5 => Self::RANK5,
            Rank::_6 => Self::RANK6,
            Rank::_7 => Self::RANK7,
            Rank::_8 => Self::RANK8,
        }
    }

    /// This bitmask, with all bits in the file set to 1.
    pub fn with_file(self, file: File) -> Self {
        self | match file {
            File::A => Self::FILEA,
            File::B => Self::FILEB,
            File::C => Self::FILEC,
            File::D => Self::FILED,
            File::E => Self::FILEE,
            File::F => Self::FILEF,
            File::G => Self::FILEG,
            File::H => Self::FILEH,
        }
    }

    pub fn with_shared(mut self, sq1: Square, sq2: Square) -> Self {
        if sq1.shares_orthogonal(sq2) {
            if sq1.file() == sq2.file() {
                return self.with_file(sq1.file());
            } else {
                return self.with_rank(sq1.rank());
            }
        } else if sq1.shares_diagonal(sq2) {
        }

        self
    }

    /// The square in the mask with the lowest value.
    /// Where H8=63 and A1=0
    /// This function is particularly useful, since
    /// it can find the first (or last, depending on the direction)
    /// square in a line between two points.
    pub fn first(self) -> Option<Square> {
        Square::try_idx(self.0.trailing_zeros() as u8)
    }

    /// The square in the mask with the highest value.
    /// Where H8=63 and A1=0
    /// This function is particularly useful, since
    /// it can find the first (or last, depending on the direction)
    /// square in a line between two points.
    pub fn last(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Square::try_idx(63 - self.0.leading_zeros() as u8)
        }
    }
}

impl From<Square> for Bitmask {
    fn from(value: Square) -> Self {
        Self(1 << value as u8)
    }
}

impl From<u64> for Bitmask {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl IntoIterator for Bitmask {
    type IntoIter = BitmaskIter;
    type Item = Square;

    fn into_iter(self) -> Self::IntoIter {
        BitmaskIter(self)
    }
}

pub struct BitmaskIter(Bitmask);

impl Iterator for BitmaskIter {
    type Item = Square;

    // Forwards, A1-B1-C1...F8-G8-H8
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(square) = Square::try_idx(self.0 .0.trailing_zeros() as u8) {
            self.0.remove(square);
            Some(square)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for BitmaskIter {
    // Backwards, H8-G8-F8...C1-B1-A1
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }

        if let Some(square) = Square::try_idx(63 - self.0 .0.leading_zeros() as u8) {
            self.0.remove(square);
            Some(square)
        } else {
            None
        }
    }
}

impl BitOr for Bitmask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitmask {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for Bitmask {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitmask {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl BitXor for Bitmask {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitmask {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl Not for Bitmask {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl std::fmt::Debug for Bitmask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bitmask: \n")?;

        for rank in Rank::iter().rev() {
            write!(f, "\n   ")?;
            for file in File::iter() {
                if self.has(Square::new(file, rank)) {
                    write!(f, " X")?;
                } else {
                    write!(f, " .")?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmask_from_square() {
        assert_eq!(Bitmask::from(Square::A1), Bitmask::EMPTY.with(Square::A1));
        assert_eq!(Bitmask::from(Square::A8), Bitmask::EMPTY.with(Square::A8));
        assert_eq!(Bitmask::from(Square::H1), Bitmask::EMPTY.with(Square::H1));
        assert_eq!(Bitmask::from(Square::H8), Bitmask::EMPTY.with(Square::H8));
    }

    #[test]
    fn bitmask_iter() {
        let mut squares: std::collections::HashSet<Square> =
            std::collections::HashSet::from_iter([
                Square::A1,
                Square::H1,
                Square::C2,
                Square::F2,
                Square::C4,
                Square::D4,
                Square::F5,
                Square::H5,
                Square::H8,
                Square::A8,
                Square::D6,
                Square::F6,
            ]);

        let mut mask = Bitmask::EMPTY;

        for square in squares.iter() {
            mask.set(*square);
        }

        for square in mask {
            if !squares.remove(&square) {
                panic!("Bitmask iter returned {square}, but it was not found in the set.");
            }
        }

        if squares.len() > 0 {
            panic!("Bitmask iter did not empty its validation set.")
        }
    }

    #[test]
    fn bitmask_union() {
        assert_eq!(
            Bitmask::from(0b00001101).union(Bitmask::from(0b00110001)),
            Bitmask::from(0b00111101)
        );
        assert_eq!(
            Bitmask::from(0b10101011).union(Bitmask::from(0b01010101)),
            Bitmask::from(0b11111111)
        );
        assert_eq!(
            Bitmask::from(0b00000000).union(Bitmask::from(0b00000000)),
            Bitmask::from(0b00000000)
        );
    }

    #[test]
    fn bitmask_intersection() {
        assert_eq!(
            Bitmask::from(0b00001101).intersection(Bitmask::from(0b00110001)),
            Bitmask::from(0b00001100)
        );
        assert_eq!(
            Bitmask::from(0b10101011).intersection(Bitmask::from(0b01010101)),
            Bitmask::from(0b10101010)
        );
        assert_eq!(
            Bitmask::from(0b01000001).intersection(Bitmask::from(0b11111111)),
            Bitmask::from(0b00000000)
        );
    }

    #[test]
    fn bitmask_flip() {
        assert_eq!(
            Bitmask::from(0b0001101).flip(Square::A1),
            Bitmask::from(0b0001100)
        );
    }
}
