use crate::board::Piece;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChessMove {
    pub from: i32,
    pub to: i32,
    pub promoted_piece: Piece,
    pub captured_piece: Piece,
    pub is_en_passant: bool,
    pub is_castle: bool,
}

#[derive(Clone, Debug)]
pub struct UndoInfo {
    pub the_move: ChessMove,
    pub piece_moved: Piece,

    pub white_to_move_before: bool,
    pub can_white_castle_kingside_before: bool,
    pub can_white_castle_queenside_before: bool,
    pub can_black_castle_kingside_before: bool,
    pub can_black_castle_queenside_before: bool,
    pub en_passant_square_before: i32,
    pub half_move_capture_or_pawn_clock_before: i32,
    pub full_move_number_before: i32,
    pub zobrist_key_before: u64,
}

impl fmt::Display for ChessMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Move {{ from: {}, to: {}, promoted: {:?}, captured: {:?}, en_passant: {}, castle: {} }}",
            self.from,
            self.to,
            self.promoted_piece,
            self.captured_piece,
            self.is_en_passant,
            self.is_castle
        )
    }
}
