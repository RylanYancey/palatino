use crate::bitmask::Bitmask;
use crate::castle::CastleDir;
use crate::castle::CastleRights;
use crate::color::Color;
use crate::piece::Piece;
use crate::position::Position;
use crate::square::File;
use crate::square::Square;

pub struct FenParser<'a>([&'a str; 6]);

impl<'a> FenParser<'a> {
    /// Parse a FEN string into a FenParser struct.
    /// This function will return an error if the
    /// number of spaces in the string is not 5.
    pub fn parse(fen: &'a str) -> Result<Self, FenParseError> {
        // check out my super epic one-liner
        fen.split_ascii_whitespace()
            .collect::<Vec<&'a str>>()
            .try_into()
            .map(|ok| Self(ok))
            .map_err(|_| FenParseError::MissingInfo)
    }

    /// Get the position from the fen, complete with
    /// the en passant square and the halfmoves number.
    pub fn position(&self) -> Result<Position, FenParseError> {
        let mut masks = [Bitmask::EMPTY; 8];

        // start at 64 since fens' start at H8 for some reason.
        let mut index: u8 = 0;

        for c in self.0[0].chars() {
            if c == '/' {
                continue;
            }

            if let Some(digit) = c.to_digit(10) {
                index += digit as u8;
            } else {
                // if this is a piece, reflect it in
                // the masks and subtract by 1.
                if let Some(piece) = Piece::from_id(c) {
                    if let Some(square) = Square::try_idx(index) {
                        let file = square.file() as u8;
                        let rank = 7 - square.rank() as u8;

                        if let Some(square) = Square::try_new(file, rank) {
                            masks[2 + piece.index()].set(square);
                            masks[Color::of_char(c) as usize].set(square);
                            index += 1;
                            continue;
                        }
                    }
                }

                return Err(FenParseError::BadPosition);
            }
        }

        Ok(Position::from_raw_parts(
            masks,
            self.halfmoves()?,
            self.en_passant()?,
        ))
    }

    /// Parse the color of the color up to play, either 'w' or 'b'.
    pub fn turn(&self) -> Result<Color, FenParseError> {
        match self.0[1] {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(FenParseError::BadTurn),
        }
    }

    /// Parse the castle rights from a string in the format
    /// KQkq.
    pub fn castle(&self) -> Result<CastleRights, FenParseError> {
        let mut rights = CastleRights::none();

        // '-' indicates there is no castling available.
        if self.0[2] == "-" {
            return Ok(rights);
        }

        for c in self.0[2].chars() {
            rights.give(
                Color::of_char(c),
                match c.to_ascii_lowercase() {
                    'k' => CastleDir::Short,
                    'q' => CastleDir::Long,
                    _ => return Err(FenParseError::BadCastle),
                },
            )
        }

        Ok(rights)
    }

    /// A FEN is Shredder if the castle state uses
    /// rook start files instead of KQkq, for example
    /// AHah.
    pub fn castle_is_shredder(&self) -> bool {
        !self.0[2].contains(&['K', 'Q', 'k', 'q', '-'])
    }

    /// ShredderFENs', developed for Chess960, use the
    /// rook start files instead of KQkq, for example
    /// AHah. The problem is they require the king locations.
    pub fn castle_as_shredder(
        &self,
        white_king: File,
        black_king: File,
    ) -> Result<CastleRights, FenParseError> {
        let mut rights = CastleRights::none();

        if self.0[2] == "-" {
            return Ok(rights);
        }

        for c in self.0[2].chars() {
            if let Some(file) = File::from_char(c) {
                let dir = match Color::of_char(c) {
                    Color::White => white_king,
                    Color::Black => black_king,
                };

                // if true, this is the kingside rook file because
                // it is to the right of the king.
                if (file as i8 - dir as i8).is_positive() {
                    rights.give(Color::of_char(c), CastleDir::Short);
                } else {
                    rights.give(Color::of_char(c), CastleDir::Long);
                }
            } else {
                // error if the character can't be parsed into a file.
                return Err(FenParseError::BadCastle);
            }
        }

        Ok(rights)
    }

    /// Get the en passant square available in the position.
    /// This should be '-' if en passant is not available.
    pub fn en_passant(&self) -> Result<Option<Square>, FenParseError> {
        if self.0[3] == "-" {
            return Ok(None);
        }

        if let Some(square) = Square::try_from_string(self.0[3]) {
            Ok(Some(square))
        } else {
            Err(FenParseError::BadEnPassant)
        }
    }

    /// Get the halfmoves of the position.
    pub fn halfmoves(&self) -> Result<u8, FenParseError> {
        if let Ok(halfmoves) = self.0[4].parse::<u8>() {
            if halfmoves > 50 {
                Err(FenParseError::BadHalfmoves)
            } else {
                Ok(halfmoves)
            }
        } else {
            Err(FenParseError::BadHalfmoves)
        }
    }

    /// Get the fullmoves number
    pub fn fullmoves(&self) -> Result<u16, FenParseError> {
        if let Ok(fullmoves) = self.0[5].parse::<u16>() {
            Ok(fullmoves)
        } else {
            Err(FenParseError::BadFullmoves)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FenParseError {
    MissingInfo,
    BadCastle,
    BadPosition,
    BadTurn,
    BadEnPassant,
    BadHalfmoves,
    BadFullmoves,
    MissingKings,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pos() -> Result<(), FenParseError> {
        let parser = FenParser::parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")?;

        let position = parser.position()?;
        let turn = parser.turn()?;
        let castle = parser.castle()?;
        let en_passant = parser.en_passant()?;
        let halfmoves = parser.halfmoves()?;
        let fullmoves = parser.fullmoves()?;

        assert_eq!(position, Position::default());
        assert_eq!(turn, Color::White);
        assert_eq!(castle, CastleRights::default());
        assert_eq!(en_passant, None);
        assert_eq!(halfmoves, 0);
        assert_eq!(fullmoves, 1);

        Ok(())
    }
}
