use crate::square::Square;
use arrayvec::ArrayString;

/// shorthand for ArrayString<7>.
pub type MoveString = ArrayString<7>;

/// A struct for recording moves.
#[derive(Clone, Debug, Hash)]
pub struct MoveRecord {
    moves: Vec<(Square, Square, MoveString)>,
}

impl MoveRecord {
    pub fn new() -> Self {
        Self { moves: Vec::new() }
    }

    /// Write a move to the internal buffer.
    pub fn write(&mut self, from: Square, dest: Square, notation: MoveString) {
        self.moves.push((from, dest, notation))
    }

    /// Get the last move written to the record, in the format (from, dest, notation).
    pub fn last(&self) -> Option<&(Square, Square, MoveString)> {
        self.moves.last()
    }

    /// Get the move that occured at the move index.
    pub fn index(&self, index: usize) -> Option<&(Square, Square, MoveString)> {
        if index >= self.moves.len() {
            None
        } else {
            Some(&self.moves[index])
        }
    }

    /// Fork the record, returning everything before the index, inclusive.
    pub fn fork_at(&self, index: usize) -> Self {
        Self {
            moves: self.moves[..=index].to_vec(),
        }
    }

    /// Pop off a move.
    pub fn pop(&mut self) -> Option<(Square, Square, MoveString)> {
        self.moves.pop()
    }
}
