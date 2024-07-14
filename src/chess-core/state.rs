use crate::castle::CastleDir;
use crate::castle::CastleRights;
use crate::color::Color;
use crate::fen::FenParseError;
use crate::fen::FenParser;
use crate::generator::MoveGenerator;
use crate::piece::Piece;
use crate::position::Position;
use crate::record::MoveString;
use crate::square::Square;

/// All of the information in a FEN, in a struct.
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct BoardState {
    position: Position,
    castle: CastleRights,
    fullmoves: u16,
    turn: Color,
}

impl BoardState {
    pub fn new(position: Position, fullmoves: u16, turn: Color, castle: CastleRights) -> Self {
        Self {
            castle,
            position,
            fullmoves,
            turn,
        }
    }

    /// Get the piece locations in the state.
    pub fn position(&self) -> Position {
        self.position
    }

    /// The color of the piece up to play.
    pub fn turn(&self) -> Color {
        self.turn
    }

    /// The Castlerights available for the position.
    pub fn castle(&self) -> CastleRights {
        self.castle
    }

    /// The en passant square, if applicable.
    pub fn en_passant(&self) -> Option<Square> {
        self.position.en_passant()
    }

    /// The number of halfmoves. This number resets when a
    /// pawn is pushed or a piece is captured, and increments
    /// otherwise, and, unlike fullmoves, increments for each
    /// white and black move.
    pub fn halfmoves(&self) -> u8 {
        self.position.halfmoves()
    }

    /// How many fullmoves have been played, where
    /// a fullmove is 1 white move and 1 black move.
    /// This number only increments when black plays.
    pub fn fullmoves(&self) -> u16 {
        self.fullmoves
    }

    /// Get the move generator for this position.
    pub fn generator(&self) -> MoveGenerator {
        MoveGenerator::from_state(self)
    }

    /// Check if a move would require promotion, that is, if a pawn moves to the enemy back rank.
    pub fn move_requires_promotion(&self, from: Square, dest: Square) -> bool {
        if let Some((_, piece)) = self.position.piece_at(from) {
            if let Piece::Pawn = piece {
                // if the piece is a pawn moving to the opponents' back rank,
                // then the move requires promotion since pawns on the backrank
                // must promote.
                if dest.rank() == (!self.turn).back_rank() {
                    return true;
                }
            }
        }

        false
    }

    /// Play a move, assuming that it has been validated by a MoveGenerator.
    pub fn play_unchecked(&self, from: Square, dest: Square, promote: Option<Piece>) -> BoardState {
        let mut result = self.position.clone();
        let mut castle = self.castle.clone();

        // reset the en passant state.
        *result.en_passant_mut() = None;

        // remove the piece off its from square.
        result.remove(from);

        // get the piece at the from square.
        if let Some((_, piece)) = self.position.piece_at(from) {
            match piece {
                // special case for en passant, promotion, and double pawn pushes.
                Piece::Pawn => {
                    // all pawn moves reset the halfmoves.
                    *result.halfmoves_mut() = 0;

                    // if this is a capture en-passant, then remove the en passant'd pawn from the position.
                    if let Some(en_passant_sq) = self.position.en_passant() {
                        if en_passant_sq == dest {
                            result.remove(from.with_file(en_passant_sq.file()));
                        }
                    }

                    // if the pawn has moved 2 squares, it is a double
                    // pawn push and enps needs to be updated accordingly.
                    if (from.rank() as i8 - dest.rank() as i8).abs() > 1 {
                        *result.en_passant_mut() = Some(
                            from.try_offset(0, self.turn.pawn_dir())
                                .expect("Failed to compute the en passant square!"),
                        );
                    }

                    // if a promotion is requested, set the destination
                    // square to occupied by the requested piece.
                    if let Some(promotion) = promote {
                        result.set(dest, promotion, self.turn);
                    } else {
                        result.set(dest, piece, self.turn);
                    }
                }
                Piece::King => {
                    let mut castled = false;

                    // !TODO! - Should this be fullmoves or fullmoves+1?
                    // all king moves lose castle rights in both directions.
                    for dir in [CastleDir::Short, CastleDir::Long] {
                        // all king moves lose castling, in both directions.
                        castle.lose(self.turn, dir, self.fullmoves);

                        // if you have not lost castle in this direction,
                        if castle.has_castle(self.turn, self.fullmoves, dir) {
                            // if the destination square is one of the squares identified
                            // as part of the squares that request castling in this direction,
                            // then the move is a castle request.
                            if castle.castle_play_mask(self.turn, dir).has(dest) {
                                let rook = castle.rook_square(self.turn, dir);

                                // remove the king and the rook from their home squares.
                                result.remove(from);
                                result.remove(rook);

                                // set the king and rook on their castle target squares.
                                let (king_target, rook_target) =
                                    castle.target_squares(self.turn, dir);
                                result.set(king_target, Piece::King, self.turn);
                                result.set(rook_target, Piece::Rook, self.turn);

                                // inform this section that we did castle,
                                // so we can avoid updating the king position
                                // unecessarily.
                                castled = true;
                            }
                        }
                    }

                    // Set the king to its target square, but not if
                    // castling occured, which would be problematic.
                    // also increment the halfmoves if the move
                    // was not a capture.
                    if !castled {
                        if result.set(dest, Piece::King, self.turn).is_some() {
                            // if it is not castling, and there is a piece on
                            // the destination square, then the move is a capture
                            // and halfmoves can be reset.
                            *result.halfmoves_mut() = 0;
                        } else {
                            // if it is not castling, and there is no piece on
                            // the destination square, then the move is not a
                            // capture and halfmoves must be incremented.
                            *result.halfmoves_mut() += 1;
                        }
                    } else {
                        // castling increments the halfmoves.
                        *result.halfmoves_mut() += 1;
                    }
                }
                _ => {
                    // rook moves may lose long/short castle.
                    if let Piece::Rook = piece {
                        for dir in [CastleDir::Long, CastleDir::Short] {
                            // we only really care about this if you haven't lost castling yet.
                            if self.castle.has_castle(self.turn, self.fullmoves, dir) {
                                // if the rook is moving off of the rook home square in this
                                // direction, the move forfeits castle in that direction.
                                if from == self.castle.rook_square(self.turn, dir) {
                                    castle.lose(self.turn, dir, self.fullmoves);
                                    break;
                                }
                            }
                        }
                    }

                    if result.set(dest, piece, self.turn).is_some() {
                        // if this is a capture, reset the halfmoves.
                        *result.halfmoves_mut() = 0;
                    } else {
                        // if this is not a capture, increment the halfmoves.
                        *result.halfmoves_mut() += 1;
                    }
                }
            }
        }

        // fullmoves increment when black moves.
        let fullmoves = match self.turn {
            Color::White => self.fullmoves,
            Color::Black => self.fullmoves + 1,
        };

        Self {
            position: result,
            castle,
            fullmoves,
            turn: !self.turn,
        }
    }

    /// Get the notation of the move, assuming that the move is valid. This does NOT include '#' or '+'.
    pub fn notation(&self, from: Square, dest: Square, promote: Option<Piece>) -> MoveString {
        MoveString::from(
            &if let Some((color, piece)) = self.position.piece_at(from) {
                match piece {
                    Piece::Pawn => {
                        // if the files aren't the same, this is a capture.
                        // I'm doing this instead of self.position.piece_at().is_some() because
                        // this might be a capture en passant, which that wouldn't detect.
                        if from.file() != dest.file() {
                            format!(
                                "{}x{}{}",
                                // captures only include the capturing file.
                                from.file().to_char_lower(),
                                // pawn captures always include the destination square after the 'x'.
                                dest.to_string_lower(),
                                // promotions are included as '=' + the id of the piece.
                                if let Some(promotion) = promote {
                                    format!("={}", promotion.id(color))
                                } else {
                                    String::new()
                                }
                            )
                        } else {
                            format!(
                                "{}{}",
                                // pawn moves are notated by just the target square.
                                dest.to_string_lower(),
                                // if its a promotion, add '=' + the id of the piece.
                                if let Some(promotion) = promote {
                                    format!("={}", promotion.id(color))
                                } else {
                                    String::new()
                                }
                            )
                        }
                    }
                    Piece::King => {
                        // castling has custom notation.
                        for dir in [CastleDir::Long, CastleDir::Short] {
                            if self.castle.has_castle(color, self.fullmoves, dir) {
                                // the move is castle in the direction if the king
                                // is moving to a castle destination square.
                                if self.castle.castle_play_mask(color, dir).has(dest) {
                                    let o = if color.is_white() { 'O' } else { 'o' };

                                    return MoveString::from(&format!(
                                        "{}-{}{}",
                                        o,
                                        o,
                                        if let CastleDir::Long = dir {
                                            format!("-{}", o)
                                        } else {
                                            String::new()
                                        }
                                    ))
                                    .unwrap_or_default();
                                }
                            }
                        }

                        // if its' not castle, check for captures
                        // unlike the other peices, we don't need to
                        // include a prefix since there is only ever one
                        // king on the board of each color.
                        if self.position.piece_at(dest).is_some() {
                            format!("{}x{}", piece.id(color), dest.to_string_lower())
                        } else {
                            format!("{}{}", piece.id(color), dest.to_string_lower())
                        }
                    }
                    _ => {
                        // every other piece that could see the destination square.
                        let conflicts = self
                            .position
                            .pieces_that_see_square(dest, piece, color)
                            .without(from);

                        let mut prefix = String::new();

                        // in the event other pieces of the same type/color could
                        // also move to the square, calculate what info needs to
                        // be provided to distinguish between the pieces.
                        if !conflicts.is_empty() {
                            if conflicts.count() == 1 {
                                // if the conflicting piece shares a file with the piece,
                                if from.file() == conflicts.first().unwrap().file() {
                                    // you have to use the rank to distinguish.
                                    prefix.push(from.rank().to_char());
                                } else {
                                    // else, you have to use the file to distinguish.
                                    prefix.push(from.file().to_char_lower());
                                }
                            } else {
                                // if there are more than 1 conflicting piece,
                                // just go ahead and provide all the info.
                                // I don't feel like implementing the checks for
                                // if we need both.
                                prefix = from.to_string_lower();
                            }
                        }

                        // put it all together, including an 'x' if the move is a capture.
                        if self.position.piece_at(dest).is_some() {
                            format!("{}{}x{}", prefix, piece.id(color), dest.to_string_lower())
                        } else {
                            format!("{}{}{}", prefix, piece.id(color), dest.to_string_lower())
                        }
                    }
                }
            } else {
                String::new()
            },
        )
        .unwrap_or_default()
    }

    /// Parse a FEN into a BoardState.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        let parser = FenParser::parse(fen)?;

        let position = parser.position()?;

        let castle = if parser.castle_is_shredder() {
            let white_kings = position.kings() & position.color_mask(Color::White);
            let black_kings = position.kings() & position.color_mask(Color::Black);

            if white_kings.count() == 0 || black_kings.count() == 0 {
                return Err(FenParseError::MissingKings);
            }

            parser.castle_as_shredder(
                white_kings.first().unwrap().file(),
                black_kings.first().unwrap().file(),
            )?
        } else {
            parser.castle()?
        };

        Ok(Self {
            position,
            castle,
            fullmoves: parser.fullmoves()?,
            turn: parser.turn()?,
        })
    }

    /// Serialize the board state to a fen.
    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.position.board_as_fen_str(),
            self.turn.to_char(),
            self.castle.to_fen_string(),
            self.position
                .en_passant()
                .map(|ok| ok.to_string_lower())
                .unwrap_or(String::from('-')),
            self.position.halfmoves().to_string(),
            self.fullmoves.to_string(),
        )
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            position: Position::default(),
            castle: CastleRights::default(),
            fullmoves: 1,
            turn: Color::White,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn notation_piece_capture() {
        let board = BoardState::from_fen(
            "r2qkb1r/pbp1p2p/1pnp1n2/1B3pB1/2PP4/4PN2/PP3PPP/RN1QK2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(
            board.notation(Square::B5, Square::C6, None).to_string(),
            "Bxc6".to_string()
        );
    }

    #[test]
    fn notation_long_castle_target_request() {
        let board = BoardState::from_fen(
            "r2qkb1r/pbp1p3/1pnp1n2/1B3pBp/2PP4/2N1PN2/PP2QPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(
            board.notation(Square::E1, Square::C1, None).to_string(),
            "O-O-O".to_string()
        );
    }

    #[test]
    fn notation_long_castle_rook_request() {
        let board = BoardState::from_fen(
            "r2qkb1r/pbp1p3/1pnp1n2/1B3pBp/2PP4/2N1PN2/PP2QPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(
            board.notation(Square::E1, Square::A1, None).to_string(),
            "O-O-O".to_string()
        );
    }

    #[test]
    fn notation_short_castle_rook_request() {
        let board = BoardState::from_fen(
            "r2qkb1r/pbp1p3/1pnp1n2/1B3pBp/2PP4/2N1PN2/PP2QPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(
            board.notation(Square::E1, Square::H1, None).to_string(),
            "O-O".to_string()
        );
    }

    #[test]
    fn notation_short_castle_target_request() {
        let board = BoardState::from_fen(
            "r2qkb1r/pbp1p3/1pnp1n2/1B3pBp/2PP4/2N1PN2/PP2QPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        assert_eq!(
            board.notation(Square::E1, Square::G1, None).to_string(),
            "O-O".to_string()
        );
    }

    #[test]
    fn notation_pawn_promotion_knight() {
        let board =
            BoardState::from_fen("2r2k1r/p1pPp1b1/1p1p1n2/5pBp/2P5/2N1PN2/PP2QPPP/R3K2R w - - 0 1")
                .unwrap();

        assert_eq!(
            board
                .notation(Square::D7, Square::C8, Some(Piece::Knight))
                .to_string(),
            "dxc8=N".to_string()
        )
    }

    #[test]
    fn notation_en_passant() {
        let board =
            BoardState::from_fen("2r2k1r/p1p3b1/1p1p1n2/3PppBp/2P5/2N2N2/PP2QPPP/R3K2R w - e6 0 1")
                .unwrap();

        assert_eq!(
            board.notation(Square::D5, Square::E6, None).to_string(),
            "dxe6".to_string()
        )
    }
}
