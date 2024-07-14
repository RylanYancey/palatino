use crate::castle::CastleRights;
use crate::color::Color;
use crate::piece::Piece;
use crate::position::Position;
use crate::square::Square;
use crate::state::BoardState;

/// A Representation of a chess game.
/// NOTE: this does not record moves, only positions.
#[derive(Clone, Debug, Hash)]
pub struct ChessGame {
    /// The initial (starting position) of the game.
    /// Correlates with index 0 in 'history'.
    first: BoardState,
    /// The most recent position, correlating with
    /// the last element in 'history'.
    last: BoardState,
    /// The position at every halfmove.
    history: Vec<Position>,
}

impl ChessGame {
    /// Get the starting position.
    pub fn first(&self) -> &BoardState {
        &self.first
    }

    /// Get the last position.
    pub fn last(&self) -> &BoardState {
        &self.last
    }

    /// The number of moves stored in the game's history.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Get the board state at an index in history.
    pub fn state_at_index(&self, index: usize) -> Option<BoardState> {
        if index < self.history.len() {
            Some(BoardState::new(
                self.history[index],
                self.fullmoves_at_index(index),
                self.turn_at_index(index),
                self.castle_rights_at_index(index),
            ))
        } else {
            None
        }
    }

    /// Fork this game at the given index, creating a
    /// new ChessGame struct with everything before and at the index.
    pub fn fork(&self, index: usize) -> Option<Self> {
        if index >= self.history.len() {
            None
        } else {
            Some(Self {
                first: self.first,
                last: self.state_at_index(index)?,
                history: self.history[..index].to_vec(),
            })
        }
    }

    /// Clear all moves after the index, exclusive.
    pub fn clear_after(&mut self, index: usize) {
        if index < self.history.len() {
            self.last = self.state_at_index(index).unwrap();
            self.history = self.history[index..].to_vec();
        }
    }

    /// Get the number of fullmoves at the index in history.
    pub fn fullmoves_at_index(&self, index: usize) -> u16 {
        // if black went first, offset by 1.
        self.first.fullmoves()
            + if self.first.turn() == Color::Black {
                (index as u16 + 3) / 2
            } else {
                (index as u16 + 2) / 2
            }
    }

    /// Get the castle rights at the index.
    pub fn castle_rights_at_index(&self, index: usize) -> CastleRights {
        let fullmoves = self.fullmoves_at_index(index);
        self.last.castle().index(index as u16)
    }

    /// Get the color of the turn at the index.
    pub fn turn_at_index(&self, index: usize) -> Color {
        if self.first.turn() == Color::White {
            if index % 2 != 0 {
                return Color::Black;
            }
        } else {
            if index % 2 == 0 {
                return Color::Black;
            }
        }

        Color::White
    }

    /// Play a move, assuming it has been validated by a MoveGenerator.
    pub fn play(&mut self, from: Square, dest: Square, promotion: Option<Piece>) {
        self.last = self.last.play_unchecked(from, dest, promotion);
        self.history.push(self.last.position());
    }

    /// Get the previous position.
    pub fn prev(&self) -> Option<BoardState> {
        if self.history.len() > 1 {
            self.state_at_index(self.history.len() - 2)
        } else {
            None
        }
    }

    /// This function will return true if the same
    /// position occurs 3 times, only checking for
    /// the most recent position.
    pub fn is_draw_by_repetition(&self) -> bool {
        let mut one = false;

        for pos in self.history.iter().rev().skip(1) {
            // pawn moves can't be reversed.
            if pos.pawns() != self.last.position().pawns() {
                return false;
            }

            // captures can't be reversed.
            if pos.count() != self.last.position().count() {
                return false;
            }

            // detect equal positions.
            if pos.masks() == self.last.position().masks() {
                if one {
                    return true;
                } else {
                    one = true;
                }
            }
        }

        false
    }
}

impl Default for ChessGame {
    fn default() -> Self {
        let default_pos = BoardState::default();

        Self {
            first: default_pos,
            last: default_pos,
            history: vec![Position::default()],
        }
    }
}
