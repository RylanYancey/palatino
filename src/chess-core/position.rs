use std::cmp::Ordering;

use crate::bitmask::Bitmask;
use crate::cached;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::{File, Rank, Square};

/// Position stores information about the locations
/// of pieces within the board, the en passant square,
/// and the halfmoves. It is all the information that
/// must be stored for each turn when accessing history.
#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct Position {
    /// Masks for the Pieces, where 0 and 1 are
    /// squares occupied by white/black, and
    /// 2-7 are squares occupied by a given
    /// piece type, agnostic of color.
    /// 0 => White Pieces
    /// 1 => Black Pieces
    /// 2 => Pawns
    /// 3 => Kings
    /// 4 => Rooks
    /// 5 => Knights
    /// 6 => Bishops
    /// 7 => Queens
    masks: [Bitmask; 8],
    /// If en passant is available in
    /// the position, this field is Some(epsq)
    enps: Option<Square>,
    /// The number of moves since the last
    /// capture or pawn push, used for calculating
    /// draw by the 50 move rule.
    halfmoves: u8,
}

impl Position {
    /// Get the mask of all squares occupied by white pieces.
    pub fn white(&self) -> Bitmask {
        self.masks[0]
    }

    /// Get the mask of all squares occupied by black pieces.
    pub fn black(&self) -> Bitmask {
        self.masks[1]
    }

    /// Get the mask of all squares occupied by pawns.
    pub fn pawns(&self) -> Bitmask {
        self.masks[2]
    }

    /// Get the mask of all squares occupied by kings.
    pub fn kings(&self) -> Bitmask {
        self.masks[3]
    }

    /// Get the mask of all squares occupied by rooks.
    pub fn rooks(&self) -> Bitmask {
        self.masks[4]
    }

    /// Get the mask of all squares occupied by knights.
    pub fn knights(&self) -> Bitmask {
        self.masks[5]
    }

    /// Get the mask of all squares occupied by bishops.
    pub fn bishops(&self) -> Bitmask {
        self.masks[6]
    }

    /// Get the mask of all squares occupied by queens.
    pub fn queens(&self) -> Bitmask {
        self.masks[7]
    }

    /// Get the internal masks array.
    pub fn masks(&self) -> &[Bitmask; 8] {
        &self.masks
    }

    /// Get the en passant state from the position.
    pub fn en_passant(&self) -> Option<Square> {
        self.enps
    }

    /// Get the mask of all squares occupied by the given color.
    pub fn color_mask(&self, color: Color) -> Bitmask {
        match color {
            Color::White => self.white(),
            Color::Black => self.black(),
        }
    }

    /// Get a mask of all pieces of the given type/color on the specified rank.
    pub fn get_pieces_on_rank(&self, piece: Piece, color: Color, rank: Rank) -> Bitmask {
        (self.masks[piece.index()] & self.color_mask(color)) & Bitmask::EMPTY.with_rank(rank)
    }

    /// Get a mask of all pieces of the given type/color on the specified file.
    pub fn get_pieces_on_file(&self, piece: Piece, color: Color, file: File) -> Bitmask {
        (self.masks[piece.index()] & self.color_mask(color)) & Bitmask::EMPTY.with_file(file)
    }

    /// All squares occupied by a piece, of any type, of any color.
    pub fn occupied(&self) -> Bitmask {
        self.masks[0].union(self.masks[1])
    }

    /// The total number of occupied squares in the mask.
    pub fn count(&self) -> u8 {
        self.masks[0].count() + self.masks[1].count()
    }

    /// Returns a mask of all other pieces of the provided type/color that
    /// can see the square, respecting the blockers bitmask, but not pins/checks.
    pub fn pieces_that_see_square(&self, square: Square, piece: Piece, color: Color) -> Bitmask {
        let mut result = Bitmask::EMPTY;
        let blockers = self.occupied();

        // for all squares occupied by pieces that could see the square
        for candidate in piece.relevant_squares(square, color)
            & (self.masks[2 + piece.index()] & self.color_mask(color))
        {
            // if there are no blockers between the candidate and the square, it can see the square.
            if Bitmask(cached::BETWEEN[square as usize][candidate as usize]).intersects(blockers) {
                result.set(candidate);
            }
        }

        result
    }

    /// All pieces and their type, agnostic of color.
    pub fn pieces(&self) -> [(Piece, Bitmask); 6] {
        [
            (Piece::Pawn, self.masks[2]),
            (Piece::King, self.masks[3]),
            (Piece::Rook, self.masks[4]),
            (Piece::Knight, self.masks[5]),
            (Piece::Bishop, self.masks[6]),
            (Piece::Queen, self.masks[7]),
        ]
    }

    /// Get the piece type at the associated square.
    pub fn piece_at(&self, square: Square) -> Option<(Color, Piece)> {
        for (index, mask) in self.masks[2..].iter().enumerate() {
            if mask.has(square) {
                return Some((
                    self.color_of(square).unwrap(),
                    Piece::from_index(index).unwrap(),
                ));
            }
        }

        None
    }

    /// Get the color of the piece at the square.
    pub fn color_of(&self, square: Square) -> Option<Color> {
        if self.white().has(square) {
            Some(Color::White)
        } else if self.black().has(square) {
            Some(Color::Black)
        } else {
            None
        }
    }

    /// Mask of Bishops and Queens of the given color.
    pub fn diagonal_sliders(&self, color: Color) -> Bitmask {
        (self.masks[7] | self.masks[6]) & self.color_mask(color)
    }

    /// Mask of Rooks and Queens of the given color.
    pub fn orthogonal_sliders(&self, color: Color) -> Bitmask {
        (self.masks[7] | self.masks[4]) & self.color_mask(color)
    }

    /// The number of halfmoves since the last pawn push or capture.
    pub fn halfmoves(&self) -> u8 {
        self.halfmoves
    }

    /// Get the halfmoves square mutably (on available in-crate to avoid any issues.)
    pub(crate) fn halfmoves_mut(&mut self) -> &mut u8 {
        &mut self.halfmoves
    }

    /// Get the en passant square mutably (only available in-crate to avoid any issues.)
    pub(crate) fn en_passant_mut(&mut self) -> &mut Option<Square> {
        &mut self.enps
    }

    /// Remove all masks that have this square in them.
    pub(crate) fn remove(&mut self, square: Square) -> Option<(Color, Piece)> {
        let color = self.color_of(square);

        // Remove the piece from its color mask.
        match color? {
            Color::Black => self.masks[1].remove(square),
            Color::White => self.masks[0].remove(square),
        }

        // remove the piece from the piece type mask.
        for (i, mask) in self.masks[2..].iter_mut().enumerate() {
            if mask.has(square) {
                mask.remove(square);

                return Some((color?, Piece::from_index(i)?));
            }
        }

        None
    }

    /// Set the square to be occupied by the piece/color,
    /// returning the displaced peice if applicable.
    pub(crate) fn set(
        &mut self,
        square: Square,
        piece: Piece,
        color: Color,
    ) -> Option<(Color, Piece)> {
        let displaced = self.remove(square);

        match color {
            Color::White => self.masks[0].set(square),
            Color::Black => self.masks[1].set(square),
        };

        self.masks[2 + piece.index()].set(square);

        displaced
    }

    /// Change the board with a BoardChange enum.
    pub fn change(&mut self, change: BoardChange) {
        match change {
            // Remove a piece from a square.
            BoardChange::Remove(square) => {
                self.remove(square);
            }
            // Move whatever is on from to dest.
            // this will overwrite any existing pieces on dest.
            BoardChange::Move(from, dest) => {
                self.remove(dest);

                if let Some((color, piece)) = self.piece_at(from) {
                    // ensure the destination square is empty.
                    self.remove(dest);

                    // update the color mask to reflect the move,
                    // and then the piece mask.
                    self.masks[color as usize].remove(from);
                    self.masks[color as usize].set(dest);
                    self.masks[2 + piece.index()].remove(from);
                    self.masks[2 + piece.index()].set(dest);
                }
            }
            // Set a square to occupied, by a given piece, for a given color.
            // overwrites any existing pieces.
            BoardChange::Add(piece, square, color) => {
                self.set(square, piece, color);
            }
        }
    }

    /// The changes required for 'self' to turn into 'other', in
    /// the order they have to happen. NOTE: this does NOT include
    /// changes to the castle state, full/halfmoves, or en passant square.
    pub fn changes(&self, other: &Self) -> Vec<BoardChange> {
        let mut changes = Vec::new();

        // we only care about the piece type masks, for now.
        let fr_masks = self.masks[2..].iter();
        let to_masks = other.masks[2..].iter();

        // iterate the masks in lock-step.
        for (i, (fr_mask, to_mask)) in (fr_masks.zip(to_masks)).enumerate() {
            // if the masks are the same, no changes need to be made.
            if fr_mask == to_mask {
                continue;
            }

            for color in [Color::White, Color::Black] {
                // get the mask for this color/type
                let fr_mask = *fr_mask & self.color_mask(color);
                let to_mask = *to_mask & other.color_mask(color);

                // get the masks for the squares in one mask,
                // but not the other, these are the squares that need
                // to be moved or otherwise changed.
                let fr_only = fr_mask.intersection(to_mask);
                let to_only = to_mask.intersection(fr_mask);

                // compare the number of differences between the two.
                match fr_only.count().cmp(&to_only.count()) {
                    // if from has more, some squares
                    //  will need to be removed.
                    Ordering::Greater => {
                        let mut movable = fr_only;

                        // remove squares until the number of squares in movable matches to_only.
                        for _ in 0..(fr_only.count() - to_only.count()) {
                            movable
                                .remove(movable.first().expect("Unreachable 000003 was reached!"));
                        }

                        // for every other square (which can not be moved), push a delete.
                        for square in fr_only.intersection(movable) {
                            changes.push(BoardChange::Remove(square));
                        }

                        // we can zip movable and fr_only together, since
                        // we guaranteed they would be the same in the previous loop.
                        for (mv, to) in movable.into_iter().zip(to_only) {
                            changes.push(BoardChange::Move(mv, to));
                        }
                    }
                    // if they have the same amount,
                    // squares only need to be moved.
                    Ordering::Equal => {
                        for (fr, to) in fr_only.into_iter().zip(to_only) {
                            changes.push(BoardChange::Move(fr, to));
                        }
                    }
                    // if from has less, some
                    // pieces need to be added.
                    Ordering::Less => {
                        let mut movable = to_only;

                        for _ in 0..(to_only.count() - fr_only.count()) {
                            // remove squares until the number of squares in movable matches fr_only.
                            movable
                                .remove(movable.first().expect("Unreachable 000001 was Reached!"));
                        }

                        // we can zip movable and fr_only together, since
                        // we guaranteed they would be the same in the previous loop.
                        for (mv, fr) in movable.into_iter().zip(fr_only) {
                            changes.push(BoardChange::Move(fr, mv));
                        }

                        // for every other square, push an add.
                        for square in to_only.intersection(movable) {
                            changes.push(BoardChange::Add(
                                Piece::from_index(i).expect("Unreachable 000002 was reached!"),
                                square,
                                color,
                            ));
                        }
                    }
                }
            }
        }

        // sort the changes so they occur in the right order.
        changes.sort_unstable_by(|left, right| left.priority().cmp(&right.priority()));

        changes
    }

    /// Create a position from its raw parts, the masks, halfmoves, and en passant.
    pub const fn from_raw_parts(
        masks: [Bitmask; 8],
        halfmoves: u8,
        en_passant: Option<Square>,
    ) -> Self {
        Self {
            masks,
            halfmoves,
            enps: en_passant,
        }
    }

    /// Convert to a grid of chracters, denoted using
    /// their algebraic names.
    pub fn to_char_grid(&self) -> [[char; 8]; 8] {
        let mut grid = [[' '; 8]; 8];

        for (piece, mask) in self.pieces() {
            for color in [Color::White, Color::Black] {
                let color_mask = self.color_mask(color);
                let id = piece.id(color);

                for square in mask & color_mask {
                    let file = square.file() as usize;
                    let rank = square.rank() as usize;

                    grid[7 - rank][file] = id;
                }
            }
        }

        grid
    }

    /// Convert the board to a fen-formatted string.
    pub fn board_as_fen_str(&self) -> String {
        let mut result = String::new();

        for (index, rank) in self.to_char_grid().iter().enumerate() {
            let mut counter = 0;

            for id in rank {
                if *id == ' ' {
                    counter += 1;
                } else {
                    if counter != 0 {
                        result.push_str(&counter.to_string());
                        counter = 0;
                    }

                    result.push(*id);
                }
            }

            if counter != 0 {
                result.push_str(&counter.to_string());
            }

            if index != 7 {
                result.push('/');
            }
        }

        result
    }
}

/// A representation of a change on the board.
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum BoardChange {
    // Removes must happen first.
    Remove(Square),
    // followed by moves,
    Move(Square, Square),
    // then adds.
    Add(Piece, Square, Color),
}

impl BoardChange {
    pub fn priority(&self) -> u8 {
        match self {
            Self::Remove(_) => 2,
            Self::Move(_, _) => 1,
            Self::Add(_, _, _) => 0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            masks: [
                // white
                Bitmask::EMPTY.with_rank(Rank::_1).with_rank(Rank::_2),
                // black
                Bitmask::EMPTY.with_rank(Rank::_8).with_rank(Rank::_7),
                // pawns
                Bitmask::EMPTY.with_rank(Rank::_2).with_rank(Rank::_7),
                // kings
                Bitmask::EMPTY.with(Square::E1).with(Square::E8),
                // rooks
                Bitmask::EMPTY
                    .with(Square::A1)
                    .with(Square::A8)
                    .with(Square::H1)
                    .with(Square::H8),
                // knights
                Bitmask::EMPTY
                    .with(Square::B1)
                    .with(Square::B8)
                    .with(Square::G1)
                    .with(Square::G8),
                // bishops
                Bitmask::EMPTY
                    .with(Square::C1)
                    .with(Square::C8)
                    .with(Square::F1)
                    .with(Square::F8),
                // queen
                Bitmask::EMPTY.with(Square::D1).with(Square::D8),
            ],

            enps: None,
            halfmoves: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FenParser;

    use super::*;

    #[test]
    fn to_char_grid() {
        let expected = [
            ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
            ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
            ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
            ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
        ];

        assert_eq!(expected, Position::default().to_char_grid());
    }

    #[test]
    fn board_as_fen_string() {
        let expected = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

        assert_eq!(expected, Position::default().board_as_fen_str());
    }

    #[test]
    fn changes() {
        let mut from = FenParser::parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
            .unwrap()
            .position()
            .unwrap();

        let dest = FenParser::parse(
            "r1bqk1nr/1ppp1pbp/p1n1p3/1B4p1/3P4/2N1PN2/PPP2PPP/R1BQK2R w KQkq - 0 1",
        )
        .unwrap()
        .position()
        .unwrap();

        for change in from.changes(&dest) {
            from.change(change);
        }

        assert_eq!(from.to_char_grid(), dest.to_char_grid())
    }
}
