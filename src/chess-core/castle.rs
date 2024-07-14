use crate::bitmask::Bitmask;
use crate::cached::BETWEEN;
use crate::color::Color;
use crate::square::{File, Rank, Square};

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub struct CastleRights {
    /// The File the King-side rook starts on.
    kingside_file: File,
    /// The File the Queen-side rook starts on.
    queenside_file: File,
    /// The turn kingside,queenside castle was lost for white.
    /// A negative number indicates castling has not been lost.
    white_lost: (i16, i16),
    /// The turn kingside,queenside castle was lost for black.
    /// A negative number indicates castling has not been lost.
    black_lost: (i16, i16),
}

impl CastleRights {
    /// Whether the color has kingside castling at a given turn.
    pub fn has_kingside_castle(&self, color: Color, turn: u16) -> bool {
        turn as i16 > self.rights(color).0
    }

    /// Whether the color has queenside castling at a given turn.
    pub fn has_queenside_castle(&self, color: Color, turn: u16) -> bool {
        turn as i16 > self.rights(color).1
    }

    /// Whether the color has castling in the given direction at the given turn.
    pub fn has_castle(&self, color: Color, turn: u16, dir: CastleDir) -> bool {
        match dir {
            CastleDir::Long => self.has_queenside_castle(color, turn),
            CastleDir::Short => self.has_kingside_castle(color, turn),
        }
    }

    /// The Square the kingside rook starts on, given a color.
    pub fn kingside_rook_square(&self, color: Color) -> Square {
        Square::new(self.kingside_file, color.back_rank())
    }

    /// The Square the Queenside rook starts on, given a color.
    pub fn queenside_rook_square(&self, color: Color) -> Square {
        Square::new(self.queenside_file, color.back_rank())
    }

    /// The square the rook starts on, given a color and direction.
    pub fn rook_square(&self, color: Color, dir: CastleDir) -> Square {
        match dir {
            CastleDir::Long => self.queenside_rook_square(color),
            CastleDir::Short => self.kingside_rook_square(color),
        }
    }

    /// Returns the squares the (king, rook) would move to when castling kingside.
    pub fn kingside_target_squares(&self, color: Color) -> (Square, Square) {
        (
            // king target square is G1 or G8
            Square::new(File::G, color.back_rank()),
            // rook target square is F1 or F8
            Square::new(File::F, color.back_rank()),
        )
    }

    /// Returns the squares the (king, rook) would move to when castling kingside.
    pub fn queenside_target_squares(&self, color: Color) -> (Square, Square) {
        (
            // king target square is C1 or C8
            Square::new(File::C, color.back_rank()),
            // rook target square is D1 or D8
            Square::new(File::D, color.back_rank()),
        )
    }

    /// Returns the squares the (king, rook) would move to when castling in the specified dir.
    pub fn target_squares(&self, color: Color, dir: CastleDir) -> (Square, Square) {
        match dir {
            CastleDir::Long => self.queenside_target_squares(color),
            CastleDir::Short => self.kingside_target_squares(color),
        }
    }

    /// Change the file the kingside rook starts on.
    pub fn with_kingside_rook_file(mut self, file: File) -> Self {
        self.kingside_file = file;
        self
    }

    /// Change the File the queenside rook starts on.
    pub fn with_queenside_rook_file(mut self, file: File) -> Self {
        self.queenside_file = file;
        self
    }

    /// Squares that, if defended by the opponent, would prevent kingside
    /// castle because it would mean castling through or into check.
    pub fn kingside_check_mask(&self, king: Square, color: Color) -> Bitmask {
        let king_target_sq = self.kingside_target_squares(color).0;
        Bitmask(BETWEEN[king as usize][king_target_sq as usize]).with(king_target_sq)
    }

    /// Squares that, if defended by the opponent, would prevent queenside
    /// castle because it would mean castling through or into check.
    pub fn queenside_check_mask(&self, king: Square, color: Color) -> Bitmask {
        let king_target_sq = self.queenside_target_squares(color).0;
        Bitmask(BETWEEN[king as usize][king_target_sq as usize]).with(king_target_sq)
    }

    /// Squares that, if defended by the opponent, would prevent castling in the
    /// specified direction because it would mean castling through or into check.
    pub fn check_mask(&self, king: Square, color: Color, dir: CastleDir) -> Bitmask {
        match dir {
            CastleDir::Long => self.queenside_check_mask(king, color),
            CastleDir::Short => self.kingside_check_mask(king, color),
        }
    }

    /// Squares that must not be occupied by any piece, since it would mean
    /// castling through a piece, which is not allowed. This mask will not
    /// include the king square or rook square, since they won't block themselves.
    pub fn kingside_block_mask(&self, king: Square, color: Color) -> Bitmask {
        let rook = self.kingside_rook_square(color);
        let (king_target, rook_target) = self.kingside_target_squares(color);

        // the resulting block mask is the squares between the king and its target and
        // the squares between the rook and its target, the target squares, but without
        // the king and rook start squares.
        Bitmask::EMPTY
            .union(Bitmask(BETWEEN[king as usize][king_target as usize]))
            .union(Bitmask(BETWEEN[rook as usize][rook_target as usize]))
            .with(rook_target)
            .with(king_target)
            .without(king)
            .without(rook)
    }

    /// Squares that must not be occupied by any piece, since it would mean
    /// castling through a piece, which is not allowed. This mask will not
    /// include the king square or rook square, since they won't block themselves.
    pub fn queenside_block_mask(&self, king: Square, color: Color) -> Bitmask {
        let rook = self.kingside_rook_square(color);
        let (king_target, rook_target) = self.queenside_target_squares(color);

        // the resulting block mask is the squares between the king and its target and
        // the squares between the rook and its target, the target squares, but without
        // the king and rook start squares.
        Bitmask::EMPTY
            .union(Bitmask(BETWEEN[king as usize][king_target as usize]))
            .union(Bitmask(BETWEEN[rook as usize][rook_target as usize]))
            .with(rook_target)
            .with(king_target)
            .without(king)
            .without(rook)
    }

    /// Squares that must not be occupied by any piece, since it would mean
    /// castling through a piece, which is not allowed. This mask will not
    /// include the king square or rook square, since they won't block themselves.
    pub fn block_mask(&self, king: Square, color: Color, dir: CastleDir) -> Bitmask {
        match dir {
            CastleDir::Long => self.queenside_block_mask(king, color),
            CastleDir::Short => self.kingside_block_mask(king, color),
        }
    }

    /// Squares the player could drop the king to tell the move generator
    /// to play long castle, which are the king square or the rook square.
    pub fn kingside_castle_play_mask(&self, color: Color) -> Bitmask {
        self.kingside_target_squares(color)
            .0
            .mask()
            .with(self.kingside_rook_square(color))
    }

    /// Squares the player could drop the king to tell the move generator
    /// to play long castle, which are the king square or the rook square.
    pub fn queenside_castle_play_mask(&self, color: Color) -> Bitmask {
        self.queenside_target_squares(color)
            .0
            .mask()
            .with(self.queenside_rook_square(color))
    }

    /// Squares the player could drop the king to tell the move generator
    /// to play castle for the given color, in the given direction, assuming
    /// that they have the rights to do so.
    pub fn castle_play_mask(&self, color: Color, dir: CastleDir) -> Bitmask {
        match dir {
            CastleDir::Long => self.queenside_castle_play_mask(color),
            CastleDir::Short => self.kingside_castle_play_mask(color),
        }
    }

    /// Inform the CastleRights that the color has lost kingside castle on the given turn.
    /// If the color has already lost kingside castling, then no changes are made.
    pub fn lose_kingside(&mut self, color: Color, turn: u16) {
        if self.rights(color).0.is_negative() {
            match color {
                Color::White => self.white_lost.0 = turn as i16,
                Color::Black => self.black_lost.0 = turn as i16,
            }
        }
    }

    /// Inform the CastleRights that the color has lost queenside castle on the given turn.
    /// If the color has already lost queenside castling, then no changes are made.
    pub fn lose_queenside(&mut self, color: Color, turn: u16) {
        if self.rights(color).1.is_negative() {
            match color {
                Color::White => self.white_lost.1 = turn as i16,
                Color::Black => self.black_lost.1 = turn as i16,
            }
        }
    }

    /// Inform the CastleRights that the color has lost castling on the given turn,
    /// in the given direction, for the given color.
    pub fn lose(&mut self, color: Color, side: CastleDir, turn: u16) {
        match side {
            CastleDir::Long => self.lose_queenside(color, turn),
            CastleDir::Short => self.lose_kingside(color, turn),
        }
    }

    /// Give the color kingside castle, setting the
    /// value associated with it to -1.
    pub fn give_kingside(&mut self, color: Color) {
        match color {
            Color::White => self.white_lost.0 = -1,
            Color::Black => self.black_lost.0 = -1,
        }
    }

    /// Give the color queenside castle,
    /// setting the value associated with it to -1.
    pub fn give_queenside(&mut self, color: Color) {
        match color {
            Color::White => self.white_lost.1 = -1,
            Color::Black => self.black_lost.1 = -1,
        }
    }

    /// Give the color the right to castle in the given direction.
    pub fn give(&mut self, color: Color, side: CastleDir) {
        match side {
            CastleDir::Long => self.give_queenside(color),
            CastleDir::Short => self.give_kingside(color),
        }
    }

    /// Get what the castle rights were at the given fullmove index.
    pub fn index(&self, fullmoves: u16) -> Self {
        let mut white_rights = self.white_lost;
        let mut black_rights = self.black_lost;

        if (fullmoves as i16) < white_rights.0 {
            white_rights.0 = -1;
        }

        if (fullmoves as i16) < white_rights.1 {
            white_rights.1 = -1;
        }

        if (fullmoves as i16) < black_rights.0 {
            black_rights.0 = -1;
        }

        if (fullmoves as i16) < black_rights.1 {
            black_rights.1 = -1;
        }

        Self {
            kingside_file: self.kingside_file,
            queenside_file: self.queenside_file,
            white_lost: white_rights,
            black_lost: black_rights,
        }
    }

    /// Get the castling rights for the color.
    fn rights(&self, color: Color) -> (i16, i16) {
        match color {
            Color::White => self.white_lost,
            Color::Black => self.black_lost,
        }
    }

    /// Creates a new CastleState object
    /// with the move castle was lost set
    /// to i16::max, indicating castling
    /// is lost in the start position.
    pub fn none() -> Self {
        Self {
            kingside_file: File::H,
            queenside_file: File::A,
            white_lost: (i16::MAX, i16::MAX),
            black_lost: (i16::MAX, i16::MAX),
        }
    }

    /// Returns the Castle State in FEN format.
    /// If the King/Queen castle files are not
    /// A & H, then the format is Shredder-FEN.
    pub fn to_fen_string(&self) -> String {
        if self.lost_all_castle(Color::White) && self.lost_all_castle(Color::Black) {
            String::from('-')
        } else {
            let mut result = String::new();

            for dir in [CastleDir::Short, CastleDir::Long] {
                for color in [Color::White, Color::Black] {
                    if self.has_castle(color, u16::MAX, dir) {
                        result.push(self.castle_dir_as_char(color, dir));
                    }
                }
            }

            result
        }
    }

    /// Does the color have any castling rights?
    pub fn lost_all_castle(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_lost.0 > -1 && self.white_lost.1 > -1,
            Color::Black => self.black_lost.0 > -1 && self.black_lost.1 > -1,
        }
    }

    fn castle_dir_as_char(&self, color: Color, dir: CastleDir) -> char {
        if self.kingside_file == File::H && self.queenside_file == File::A {
            if color.is_white() {
                dir.to_char().to_ascii_uppercase()
            } else {
                dir.to_char()
            }
        } else {
            if color.is_white() {
                self.rook_square(color, dir).file().to_char_upper()
            } else {
                self.rook_square(color, dir).file().to_char_lower()
            }
        }
    }
}

impl Default for CastleRights {
    fn default() -> Self {
        Self {
            kingside_file: File::H,
            queenside_file: File::A,
            white_lost: (-1, -1),
            black_lost: (-1, -1),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub enum CastleDir {
    Long,
    Short,
}

impl CastleDir {
    pub fn to_char(&self) -> char {
        match self {
            Self::Long => 'q',
            Self::Short => 'k',
        }
    }
}
