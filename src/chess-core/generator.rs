use crate::bitmask::Bitmask;
use crate::cached;
use crate::cached::BETWEEN;
use crate::cached::BISHOP;
use crate::cached::ROOK;
use crate::castle::CastleDir;
use crate::castle::CastleRights;
use crate::color::Color;
use crate::piece::Piece;
use crate::position::Position;
use crate::square::Square;
use crate::state::BoardState;

/// A struct that contains information required to
/// efficiently generate possible moves in a position
/// and check for end conditions like checkmate
/// and stalemate.
#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct MoveGenerator {
    /// The position moves will be generated for.
    position: Position,
    /// The color of the player up to move.
    turn: Color,
    /// The castle rights in the position.
    castle: CastleRights,
    /// The number of fullmoves since the start position.
    /// We need the fullmoves to get the castle rights.
    fullmoves: u16,
    /// The mask of squares defended by
    /// the opponent, where sliders
    /// can see through the king.
    defense: Bitmask,
    /// The mask of squares occupied by
    /// pieces that are being pinned by
    /// enemy sliders, either orthogonally
    /// or diagonally.
    pinned: Bitmask,
    /// The mask of squares occupied by
    /// enemy pieces that are actively
    /// checking the king, either
    /// blockable or nonblockable.
    checking: Bitmask,
}

impl MoveGenerator {
    pub fn new(position: Position, turn: Color, castle: CastleRights, fullmoves: u16) -> Self {
        let defense = compute_defense_mask(&position, turn);
        let (pinned, checking) = compute_pinned_and_checking_masks(&position, turn);

        Self {
            fullmoves,
            defense,
            pinned,
            checking,
            position,
            turn,
            castle,
        }
    }

    pub fn from_state(state: &BoardState) -> Self {
        Self::new(
            state.position(),
            state.turn(),
            state.castle(),
            state.fullmoves(),
        )
    }

    /// Generate the valid moves for a piece at the square.
    /// This function will return Bitmask::EMPTY if it is not
    /// the pieces' turn to move.
    pub fn generate(&self, square: Square) -> Bitmask {
        if let Some((color, piece)) = self.position.piece_at(square) {
            if color == self.turn {
                return self.generate_internal(piece, square, self.king());
            }
        }

        Bitmask::EMPTY
    }

    /// Whether the king is in check.
    pub fn is_check(&self) -> bool {
        !self.checking.is_empty()
    }

    /// Returns true if ANY piece in the position has a valid move.
    pub fn has_any_moves(&self) -> bool {
        let friendly = self.position.color_mask(self.turn);
        let king = self.king();

        for (piece, mask) in self.position.pieces() {
            for square in mask & friendly {
                if !self.generate_internal(piece, square, king).is_empty() {
                    return true;
                }
            }
        }

        false
    }

    /// Private function for generating moves for a piece, assuming it
    /// exists in the position at the square and with the color.
    fn generate_internal(&self, piece: Piece, square: Square, king: Square) -> Bitmask {
        let blockers = self.position.occupied();

        // get the candidate moves from the piece.
        let (mut attacks, moves) = piece.moves(square, blockers, self.turn);

        // you can't capture your own pieces, ever, so remove
        // any candidate moves that are of the same color.
        attacks &= !self.position.color_mask(self.turn);

        // special moves of the piece, which is used for castling and en passant.
        let mut specials = Bitmask::EMPTY;

        match piece {
            // Pawns have special moves.
            Piece::Pawn => {
                // by default, the pawns' capturable squares are enemies.
                let mut capturable = self.position.color_mask(!self.turn);

                // if en passant is available in the position,
                if let Some(en_passant_sq) = self.position.en_passant() {
                    // if this pawn has the en passant sq in its attacks,
                    if attacks.has(en_passant_sq) {
                        // if the en passant capture would not move into a discovered check,
                        if !en_passant_would_move_into_discovered_check(
                            &self.position,
                            en_passant_sq,
                            square,
                            king,
                            self.turn,
                        ) {
                            let capture_sq = square.with_file(en_passant_sq.file());

                            match self.checking.count() {
                                // if there are no checks, we can just assume the en passant is valid.
                                0 => specials.set(en_passant_sq),
                                // if there is 1 check, and the capture square is the checking piece,
                                // assume en passant is valid.
                                1 if self.checking.has(capture_sq) => {
                                    // en passant is only valid if the pawn is not pinned.
                                    if !self.pinned.has(square) {
                                        specials.set(en_passant_sq)
                                    }
                                }
                                // if there is 1 check, and it is not the capture square,
                                // then add the en passant square to the capturable so the
                                // check and pin detection can handle the result.
                                1 => {
                                    capturable.set(en_passant_sq);
                                }
                                // if there are two checks, then en passant is not possible.
                                _ => {}
                            }
                        }
                    }
                }

                // pawns can only capture on squares occupied by enemy pieces, or the en passant
                // square in the event there is 1 check that is not the en passantable piece,
                // as calculated above.
                attacks &= capturable;

                // combine the attacks and moves into one.
                attacks |= moves;
            }
            // Kings have castling to check for.
            Piece::King => {
                // Can't castle if the king is in check.
                if !self.is_check() {
                    // for each possible castle direction,
                    for dir in [CastleDir::Short, CastleDir::Long] {
                        // if the player has no lost their right to castle in this direction,
                        if self.castle.has_castle(self.turn, self.fullmoves, dir) {
                            // check if the king would be castling into or through a defended square,
                            // or if there are any blocking pieces between the king and its target square,
                            // or between the rook and its target square, which would prevent castling.
                            if !self
                                .castle
                                .check_mask(king, self.turn, dir)
                                .intersects(self.defense)
                                && self
                                    .castle
                                    .block_mask(king, self.turn, dir)
                                    .intersects(blockers)
                            {
                                // if all checks are good, castle can be requested by
                                // moving the king to its target square or by dropping the king
                                // on the rook in the castle direction.
                                specials |= self.castle.castle_play_mask(self.turn, dir)
                            }
                        }
                    }
                }

                // King can't move to squares defended by the opponent.
                attacks &= !self.defense;
            }
            // all other pieces behave normally.
            _ => {}
        }

        // Moves must capture checking pieces
        // or block a checking peices' sightline
        // to the king.
        for checking in self.checking {
            attacks &= Bitmask(BETWEEN[king as usize][checking as usize]).with(checking)
        }

        // If the piece is pinned, then only moves that maintain the
        // pin by staying on the shared diagonal/orthogonal are valid.
        if self.pinned.has(square) {
            if square.shares_orthogonal(king) {
                attacks &= Bitmask(ROOK[king as usize] & ROOK[square as usize]);
            } else {
                attacks &= Bitmask(BISHOP[king as usize] & BISHOP[square as usize]);
            }
        }

        attacks | specials
    }

    /// Get the square the king is on.
    fn king(&self) -> Square {
        (self.position.kings() & self.position.color_mask(self.turn))
            .first()
            .expect("MoveGenerator expects the position to have a king.")
    }
}

/// Compute the mask of squares defended by the opponent.
fn compute_defense_mask(pos: &Position, turn: Color) -> Bitmask {
    let mut defense = Bitmask::EMPTY;

    // the king square of the turn color.
    let king = (pos.kings() & pos.color_mask(turn))
        .first()
        .expect("MoveGenerator::new() expects the position to have a king.");

    let friendly = pos.color_mask(turn);
    let blockers = pos.occupied().without(king);

    // Compute the squares defended by the enemy team.
    for (piece, mask) in pos.pieces() {
        for square in mask.intersection(friendly) {
            // we only care about attacks, not pawn moves, so
            // we add everything in moves.0 to the defense mask.
            defense |= piece.moves(square, blockers, !turn).0
        }
    }

    defense
}

/// Compute the mask of squares occupied by pieces which are pinned to the king, and
/// squares occupied by pieces that are actively checking the king.
fn compute_pinned_and_checking_masks(pos: &Position, turn: Color) -> (Bitmask, Bitmask) {
    let mut pinned = Bitmask::EMPTY;
    let mut checking = Bitmask::EMPTY;

    // the king square of the turn color.
    let king = (pos.kings() & pos.color_mask(turn))
        .first()
        .expect("MoveGenerator::new() expects the position to have a king.");

    // all occupied squares, which block slides.
    let blockers = pos.occupied();

    // all pieces occupied by friendly squares.
    let friendly = pos.color_mask(turn);

    // Compute pinned pieces and checking squares on the
    // diagonals and orthogonals by iterating the pieces that
    // are diagonal AND share a diagonal with the king OR
    // are orthogonal AND share an orthogonal with the king,
    // such that the mask we're iterating won't include any diagonal
    // sliders that share an orthogonal with the king and vice versa.
    for square in pos
        .diagonal_sliders(!turn)
        .intersection(!Bitmask(cached::BISHOP[king as usize]))
        .union(
            pos.orthogonal_sliders(!turn)
                .intersection(Bitmask(!cached::ROOK[king as usize])),
        )
    {
        // Squares between the King and the Diagonal Slider
        let between = Bitmask(cached::BETWEEN[king as usize][square as usize]);
        // Occupied squares in the squares between the king and the diagonal slider.
        let blocking = blockers & between;

        // if there are no squares blocking the
        // diagonal sliders' line of sight to the king,
        // then it is a checking square.
        if blocking.count() == 0 {
            checking.set(square);
            continue;
        }

        // if there is one square blocking the diagonal sliders' line
        // of sight to the king, and the color of that piece is
        // the same as the king, then the square is pinned.
        if blocking.count() == 1 && friendly.has(square) {
            pinned.set(square);
        }
    }

    // find enemy knights on squares that attack the king.
    for square in (pos.knights() & !friendly) & Bitmask(cached::KNIGHT[king as usize]) {
        checking.set(square)
    }

    // find enemy pawns on squares that attack the king.
    for square in (pos.pawns() & !friendly)
        & Bitmask(if turn == Color::White {
            cached::WHITE_PAWN_ATTACKS[king as usize]
        } else {
            cached::BLACK_PAWN_ATTACKS[king as usize]
        })
    {
        checking.set(square)
    }

    (pinned, checking)
}

fn en_passant_would_move_into_discovered_check(
    pos: &Position,
    epsq: Square,
    square: Square,
    king: Square,
    turn: Color,
) -> bool {
    // the square of the pawn that would be captured
    // if capture en passant took place.
    let capture_sq = square.with_file(epsq.file());

    // change blockers to reflect what the position would
    // look like after the capture en passant.
    let blockers = pos
        .occupied()
        .with(epsq)
        .without(square)
        .without(capture_sq);

    // If the capture sq and the king share an orthogonal,
    // then it is possible for en passant to result in a discovered check,
    // which is invalid. The same is true if they share a diagonal.
    for square in if capture_sq.shares_orthogonal(king) {
        pos.orthogonal_sliders(!turn) & Bitmask(cached::ROOK[king as usize])
    } else if epsq.shares_diagonal(king) {
        pos.diagonal_sliders(!turn) & Bitmask(cached::BISHOP[king as usize])
    } else {
        return true;
    } {
        // if no squares between the slider and the king are occupied, then en passant would
        // move into discovered check.
        if !(Bitmask(cached::BETWEEN[king as usize][square as usize]).intersects(blockers)) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::BoardState;

    #[test]
    fn generate_0() {
        let board =
            BoardState::from_fen("2r2k1r/p1p3b1/1p1p1n2/3PppBp/2P5/2N2N2/PP2QPPP/R3K2R w - e6 0 1")
                .unwrap();

        let generator = board.generator();

        assert_eq!(generator.generate(Square::D5), Square::E6.mask());
    }
}
