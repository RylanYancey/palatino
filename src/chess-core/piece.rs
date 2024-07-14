use crate::bitmask::Bitmask;
use crate::cached;
use crate::color::Color;
use crate::square::{File, Rank, Square};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Piece {
    Pawn = 0,
    King,
    Rook,
    Knight,
    Bishop,
    Queen,
}

impl Piece {
    /// The Index of the Piece.
    /// Associates with an index in 'position'.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The ID of the piece, as a character.
    /// If the provided color is Color::White,
    /// then the result will be uppercase.
    pub fn id(self, color: Color) -> char {
        let id = match self {
            Self::Pawn => 'p',
            Self::King => 'k',
            Self::Rook => 'r',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Queen => 'q',
        };

        match color {
            Color::White => id.to_ascii_uppercase(),
            Color::Black => id,
        }
    }

    /// Convert an index 0-5 into a piece.
    pub fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::Pawn,
            1 => Self::King,
            2 => Self::Rook,
            3 => Self::Knight,
            4 => Self::Bishop,
            5 => Self::Queen,
            _ => return None,
        })
    }

    /// Convert a character ID to a Piece.
    /// Accepted inputs are pkrnbq and their
    /// uppercase variants.
    pub fn from_id(char: char) -> Option<Self> {
        Some(match char.to_ascii_lowercase() {
            'p' => Self::Pawn,
            'k' => Self::King,
            'r' => Self::Rook,
            'n' => Self::Knight,
            'b' => Self::Bishop,
            'q' => Self::Queen,
            _ => return None,
        })
    }

    /// Get relevant capture squares for this piece.
    pub fn relevant_squares(&self, square: Square, color: Color) -> Bitmask {
        Bitmask(match self {
            Self::Pawn => match color {
                Color::White => cached::WHITE_PAWN_ATTACKS[square as usize],
                Color::Black => cached::BLACK_PAWN_ATTACKS[square as usize],
            },
            Self::King => cached::KING[square as usize],
            Self::Knight => cached::KNIGHT[square as usize],
            _ => return self.sliding_attacks(square),
        })
    }

    /// The Squares a piece of this type at 'square' can attack / move to,
    /// provided a mask of squares which can block sliders.
    /// The resulting mask will include any blockers that intersect
    /// the pieces attacks.
    ///
    /// For pawns, the color parameter is required for the direction. The first
    /// bitmask is the capture moves, and the second is the push-only moves, taking the
    /// blockers into account.
    pub fn moves(&self, square: Square, blockers: Bitmask, color: Color) -> (Bitmask, Bitmask) {
        // sliders need extra processing.
        if self.is_slider() {
            // iterate the directions this piece can move in.
            (
                self.edges(square).iter().fold(
                    self.sliding_attacks(square),
                    |mut mask, (edge, nearest_fn)| {
                        // the squares between the piece and the edge of the board in
                        // a direction the piece is capable of moving in.
                        let between = between(square, *edge);

                        // Get all the squares that block the piece
                        // from sliding in this direction.
                        let blocking = between & blockers;

                        if let Some(nearest) = (nearest_fn)(blocking) {
                            // if there is a square blocking the slide, then
                            // exclude all squares between the nearest blocking
                            // square and the edge of the board.
                            mask.intersection(self::between(nearest, *edge))
                                .without(*edge)
                        } else {
                            // if there are no blocking squares,
                            // then the mask doesn't need to change
                            // for this direction.
                            mask
                        }
                    },
                ),
                Bitmask::EMPTY,
            )
        } else {
            (
                self.relevant_squares(square, color),
                if let Self::Pawn = *self {
                    let mut moves = Bitmask(match color {
                        Color::White => cached::WHITE_PAWN_MOVES[square as usize],
                        Color::Black => cached::BLACK_PAWN_MOVES[square as usize],
                    });

                    // one square.
                    if let Some(one) = square.try_offset(0, color.pawn_dir()) {
                        if blockers.has(one) {
                            moves.remove(one);
                        }

                        // two square.
                        if let Some(two) = one.try_offset(0, color.pawn_dir()) {
                            if !moves.has(one) || blockers.has(one) || blockers.has(two) {
                                moves.remove(two);
                            }
                        }
                    }

                    moves
                } else {
                    Bitmask::EMPTY
                },
            )
        }
    }

    /// Whether the piece is a Rook, Bishop, or a Queen.
    pub fn is_slider(&self) -> bool {
        match self {
            Self::Rook | Self::Queen | Self::Bishop => true,
            _ => false,
        }
    }

    /// Given a square a piece is standing on, get the edges of the board in each
    /// diagonal and orthogonal direction, and a function to get the nearest occupied square in a
    /// bitmask of squares between 'square' and the edge.
    fn edges(&self, square: Square) -> Vec<(Square, NearestFn)> {
        let indices = match self {
            Self::Bishop => 4..8,
            Self::Rook => 0..4,
            _ => 0..8,
        };

        [
            (square.with_file(File::A), Bitmask::last as NearestFn), // to the left  (-x dir)
            (square.with_file(File::H), Bitmask::first as NearestFn), // to the right (+x dir)
            (square.with_rank(Rank::_1), Bitmask::last as NearestFn), // down (-y dir)
            (square.with_rank(Rank::_8), Bitmask::first as NearestFn), // up (+y dir)
            (square.diag_edge((1, 1)), Bitmask::first as NearestFn), // up, right +x, +y
            (square.diag_edge((-1, -1)), Bitmask::last as NearestFn), // down, left -x -y
            (square.diag_edge((-1, 1)), Bitmask::first as NearestFn), // down, right, +x -y
            (square.diag_edge((1, -1)), Bitmask::last as NearestFn), // up, left -x +y
        ][indices]
            .to_vec()
    }

    /// Utility function for getting the candidates for a sliding piece.
    fn sliding_attacks(&self, square: Square) -> Bitmask {
        Bitmask(match self {
            Self::Bishop => cached::BISHOP[square as usize],
            Self::Rook => cached::ROOK[square as usize],
            _ => cached::QUEEN[square as usize],
        })
    }
}

type NearestFn = fn(Bitmask) -> Option<Square>;

fn between(sq1: Square, sq2: Square) -> Bitmask {
    Bitmask(cached::BETWEEN[sq1 as usize][sq2 as usize])
}
