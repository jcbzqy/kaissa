use once_cell::sync::Lazy;
use rand::Rng;

pub type ZobristKey = u64;

struct ZobristTables {
    piece: [[u64; 64]; 13],
    castling: [u64; 4],
    en_passant: [u64; 64],
    white_to_move: u64,
}

static ZOBRIST: Lazy<ZobristTables> = Lazy::new(|| {
    let mut rng = rand::thread_rng();
    let mut piece = [[0u64; 64]; 13];
    let mut castling = [0u64; 4];
    let mut en_passant = [0u64; 64];
    #[allow(unused_assignments)]
    let mut white_to_move = 0u64;

    for p in 0..13 {
        for sq in 0..64 {
            piece[p][sq] = rng.gen();
        }
    }
    for i in 0..4 {
        castling[i] = rng.gen();
    }
    for sq in 0..64 {
        en_passant[sq] = rng.gen();
    }
    white_to_move = rng.gen();

    ZobristTables {
        piece,
        castling,
        en_passant,
        white_to_move,
    }
});

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Empty = 0,
    WP = 1,
    WN = 2,
    WB = 3,
    WR = 4,
    WQ = 5,
    WK = 6,
    BP = 7,
    BN = 8,
    BB = 9,
    BR = 10,
    BQ = 11,
    BK = 12,
}

pub fn piece_to_char(piece: Piece) -> char {
    match piece {
        Piece::Empty => '.',
        Piece::WP => 'P',
        Piece::WN => 'N',
        Piece::WB => 'B',
        Piece::WR => 'R',
        Piece::WQ => 'Q',
        Piece::WK => 'K',
        Piece::BP => 'p',
        Piece::BN => 'n',
        Piece::BB => 'b',
        Piece::BR => 'r',
        Piece::BQ => 'q',
        Piece::BK => 'k',
    }
}

pub fn square_to_algebraic(sq_index: usize) -> String {
    let row = sq_index / 8;
    let col = sq_index % 8;
    let file_char = (b'a' + col as u8) as char;
    let rank_char = (b'8' - row as u8) as char;
    format!("{}{}", file_char, rank_char)
}

#[derive(Clone, Debug)]
pub struct Board {
    pub board: [Piece; 64],
    pub white_to_move: bool,
    pub can_white_castle_kingside: bool,
    pub can_white_castle_queenside: bool,
    pub can_black_castle_kingside: bool,
    pub can_black_castle_queenside: bool,
    pub en_passant_square: i32,
    pub half_move_capture_or_pawn_clock: i32,
    pub full_move_number: i32,
}

impl Board {
    pub fn compute_zobrist_key(&self) -> ZobristKey {
        let mut key = 0u64;
        for square in 0..64 {
            let piece = self.board[square];
            if piece != Piece::Empty {
                let piece_index = piece as usize; // 0..12
                key ^= ZOBRIST.piece[piece_index][square];
            }
        }
        if self.can_white_castle_kingside {
            key ^= ZOBRIST.castling[0];
        }
        if self.can_white_castle_queenside {
            key ^= ZOBRIST.castling[1];
        }
        if self.can_black_castle_kingside {
            key ^= ZOBRIST.castling[2];
        }
        if self.can_black_castle_queenside {
            key ^= ZOBRIST.castling[3];
        }
        if self.en_passant_square >= 0 && self.en_passant_square < 64 {
            let sq = self.en_passant_square as usize;
            key ^= ZOBRIST.en_passant[sq];
        }
        if self.white_to_move {
            key ^= ZOBRIST.white_to_move;
        }
        key
    }
}

pub fn board_to_string(board_obj: &Board) -> String {
    let mut out = String::new();

    out.push_str("  +-----------------+\n");
    for rank in (0..8).rev() {
        out.push_str(&format!("{} | ", rank + 1));
        for file in 0..8 {
            let sq_index = (7 - rank) * 8 + file;
            let p = board_obj.board[sq_index];
            out.push(piece_to_char(p));
            out.push(' ');
        }
        out.push_str("|\n");
    }
    out.push_str("  +-----------------+\n");
    out.push_str("    a b c d e f g h\n\n");

    if board_obj.white_to_move {
        out.push_str("White to move\n");
    } else {
        out.push_str("Black to move\n");
    }

    out.push_str("Castling rights: ");
    let mut has_any_castling_right = false;
    if board_obj.can_white_castle_kingside {
        out.push('K');
        has_any_castling_right = true;
    }
    if board_obj.can_white_castle_queenside {
        out.push('Q');
        has_any_castling_right = true;
    }
    if board_obj.can_black_castle_kingside {
        out.push('k');
        has_any_castling_right = true;
    }
    if board_obj.can_black_castle_queenside {
        out.push('q');
        has_any_castling_right = true;
    }
    if !has_any_castling_right {
        out.push('-');
    }
    out.push('\n');

    if board_obj.en_passant_square >= 0 && board_obj.en_passant_square < 64 {
        let sq_str = square_to_algebraic(board_obj.en_passant_square as usize);
        out.push_str(&format!("En passant target: {}\n", sq_str));
    } else {
        out.push_str("En passant target: -\n");
    }

    out.push_str(&format!(
        "Halfmove clock: {}\n",
        board_obj.half_move_capture_or_pawn_clock
    ));

    out.push_str(&format!(
        "Fullmove number: {}\n",
        board_obj.full_move_number
    ));

    out
}
