use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::board::{Board, Piece, ZobristKey};
use crate::chessmove::{ChessMove, UndoInfo};
use crate::movegen::{generate_legal_moves, is_king_in_check, make_move, row_of, to_index};

const MAX_DEPTH: usize = 64;

static PIECE_VALUES: [f64; 13] = [
    0.0, 1.0, 3.2, 3.3, 5.0, 9.0, 1000.0, 1.0, 3.2, 3.3, 5.0, 9.0, 1000.0,
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    PVNode,  // Exact
    AllNode, // Alpha
    CutNode, // Beta
}

#[derive(Clone, Debug)]
pub struct TranspositionTableEntry {
    pub key: ZobristKey,
    pub depth: i32,
    pub value: f64,
    pub node_type: NodeType,
    pub best_move: ChessMove,
}

#[derive(Clone)]
pub struct TranspositionTable {
    table: HashMap<ZobristKey, TranspositionTableEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: HashMap::new(),
        }
    }

    pub fn probe(&self, key: ZobristKey) -> Option<TranspositionTableEntry> {
        self.table.get(&key).cloned()
    }

    pub fn store(
        &mut self,
        key: ZobristKey,
        value: f64,
        node_type: NodeType,
        depth: i32,
        best_move: ChessMove,
    ) {
        let entry = TranspositionTableEntry {
            key,
            depth,
            value,
            node_type,
            best_move,
        };
        self.table.insert(key, entry);
    }

    pub fn clear(&mut self) {
        self.table.clear();
    }
}

#[derive(Clone)]
struct KillerMoves {
    moves: [Option<ChessMove>; 2],
}

impl Default for KillerMoves {
    fn default() -> Self {
        KillerMoves {
            moves: [None, None],
        }
    }
}

fn unmake_move(board: &mut Board, mv: &ChessMove, undo: &UndoInfo) {
    board.white_to_move = undo.white_to_move_before;
    board.can_white_castle_kingside = undo.can_white_castle_kingside_before;
    board.can_white_castle_queenside = undo.can_white_castle_queenside_before;
    board.can_black_castle_kingside = undo.can_black_castle_kingside_before;
    board.can_black_castle_queenside = undo.can_black_castle_queenside_before;
    board.en_passant_square = undo.en_passant_square_before;
    board.half_move_capture_or_pawn_clock = undo.half_move_capture_or_pawn_clock_before;
    board.full_move_number = undo.full_move_number_before;

    let moving_piece = if mv.promoted_piece != Piece::Empty {
        mv.promoted_piece
    } else {
        undo.piece_moved
    };

    board.board[mv.to as usize] = Piece::Empty;

    if mv.is_castle {
        let king_side = (row_of(mv.to) == 7 && mv.to - mv.from == 2)
            || (row_of(mv.to) == 0 && mv.to - mv.from == 2)
            || (crate::movegen::col_of(mv.to) == 6);

        if moving_piece == Piece::WK {
            if king_side {
                board.board[to_index(7, 7) as usize] = Piece::WR;
                board.board[to_index(7, 5) as usize] = Piece::Empty;
            } else {
                board.board[to_index(7, 0) as usize] = Piece::WR;
                board.board[to_index(7, 3) as usize] = Piece::Empty;
            }
        } else if moving_piece == Piece::BK {
            if king_side {
                board.board[to_index(0, 7) as usize] = Piece::BR;
                board.board[to_index(0, 5) as usize] = Piece::Empty;
            } else {
                board.board[to_index(0, 0) as usize] = Piece::BR;
                board.board[to_index(0, 3) as usize] = Piece::Empty;
            }
        }
    }

    board.board[mv.from as usize] = undo.piece_moved;

    if mv.is_en_passant {
        let captured = mv.captured_piece; // e.g. WP or BP
        let direction = if captured == Piece::WP { -8 } else { 8 };
        board.board[(mv.to + direction) as usize] = captured;
    } else if mv.captured_piece != Piece::Empty {
        board.board[mv.to as usize] = mv.captured_piece;
    }
}

fn is_killer_move(killers: &KillerMoves, mv: &ChessMove) -> bool {
    for kopt in killers.moves.iter() {
        if let Some(k) = kopt {
            if k.from == mv.from && k.to == mv.to && k.promoted_piece == mv.promoted_piece {
                return true;
            }
        }
    }
    false
}

fn store_killer_move(killers: &mut KillerMoves, mv: &ChessMove) {
    if mv.captured_piece != Piece::Empty {
        return;
    }
    if killers.moves.iter().any(|&kopt| {
        kopt.map_or(false, |k| {
            k.from == mv.from && k.to == mv.to && k.promoted_piece == mv.promoted_piece
        })
    }) {
        return;
    }
    killers.moves[1] = killers.moves[0];
    killers.moves[0] = Some(*mv);
}

pub struct Search {
    tt: TranspositionTable,
    killer_moves: [KillerMoves; MAX_DEPTH],
    search_start_time: Instant,
    move_time_limit: Option<Duration>,
}

impl Search {
    pub fn new() -> Self {
        Search {
            tt: TranspositionTable::new(),
            killer_moves: core::array::from_fn(|_| KillerMoves::default()),
            search_start_time: Instant::now(),
            move_time_limit: None,
        }
    }

    fn evaluate(&self, board: &Board) -> f64 {
        let mut score = 0.0;
        for sq in 0..64 {
            let p = board.board[sq];
            if p != Piece::Empty {
                score += PIECE_VALUES[p as usize]
                    * if p as usize <= Piece::WK as usize {
                        1.0
                    } else {
                        -1.0
                    };
            }
        }
        score
    }

    fn alpha_beta(&mut self, board: &mut Board, depth: i32, mut alpha: f64, mut beta: f64) -> f64 {
        let original_alpha = alpha;
        if depth == 0 {
            return self.evaluate(board);
        }

        let key = board.compute_zobrist_key();
        if let Some(tt_entry) = self.tt.probe(key) {
            if tt_entry.depth >= depth {
                match tt_entry.node_type {
                    NodeType::PVNode => {
                        return tt_entry.value;
                    }
                    NodeType::AllNode => {
                        if tt_entry.value < beta {
                            beta = tt_entry.value;
                        }
                    }
                    NodeType::CutNode => {
                        if tt_entry.value > alpha {
                            alpha = tt_entry.value;
                        }
                    }
                }
                if alpha >= beta {
                    return tt_entry.value;
                }
            }
        }

        let mut moves = generate_legal_moves(board);

        let km = &self.killer_moves[depth as usize];
        let mut insert_pos = 0_usize;
        for i in 0..moves.len() {
            if is_killer_move(km, &moves[i]) {
                moves.swap(i, insert_pos);
                insert_pos += 1;
            }
        }

        if moves.is_empty() {
            if is_king_in_check(board, board.white_to_move) {
                return -999_999.0;
            }
            return 0.0;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_move = moves[0]; // fallback
        let node_type;

        for mv in &moves {
            let undo = UndoInfo {
                the_move: *mv,
                piece_moved: board.board[mv.from as usize],
                white_to_move_before: board.white_to_move,
                can_white_castle_kingside_before: board.can_white_castle_kingside,
                can_white_castle_queenside_before: board.can_white_castle_queenside,
                can_black_castle_kingside_before: board.can_black_castle_kingside,
                can_black_castle_queenside_before: board.can_black_castle_queenside,
                en_passant_square_before: board.en_passant_square,
                half_move_capture_or_pawn_clock_before: board.half_move_capture_or_pawn_clock,
                full_move_number_before: board.full_move_number,
                zobrist_key_before: key,
            };

            make_move(board, mv);
            let score = -self.alpha_beta(board, depth - 1, -beta, -alpha);
            unmake_move(board, mv, &undo);

            if score > best_score {
                best_score = score;
                best_move = *mv;
                if score > alpha {
                    alpha = score;
                }
            }

            if alpha >= beta {
                store_killer_move(&mut self.killer_moves[depth as usize], mv);
                break;
            }
        }

        if best_score <= original_alpha {
            node_type = NodeType::AllNode;
        } else if best_score >= beta {
            node_type = NodeType::CutNode;
        } else {
            node_type = NodeType::PVNode;
        }

        self.tt.store(key, best_score, node_type, depth, best_move);

        best_score
    }

    pub fn find_best_move(
        &mut self,
        board: &mut Board,
        depth: i32,
        stop_requested: &AtomicBool,
        move_time: Option<Duration>,
    ) -> Option<ChessMove> {
        self.search_start_time = Instant::now();
        self.move_time_limit = move_time;

        let moves = generate_legal_moves(board);
        if moves.is_empty() {
            return None;
        }
        let mut best_score = f64::NEG_INFINITY;
        let mut best_move = moves[0];

        let mut alpha = f64::NEG_INFINITY;
        let beta = f64::INFINITY;

        for mv in moves {
            if stop_requested.load(std::sync::atomic::Ordering::Relaxed) {
                return Some(best_move);
            }
            if let Some(limit) = self.move_time_limit {
                let elapsed = Instant::now().duration_since(self.search_start_time);
                if elapsed >= limit {
                    return Some(best_move);
                }
            }

            let key = board.compute_zobrist_key();
            let undo = UndoInfo {
                the_move: mv,
                piece_moved: board.board[mv.from as usize],
                white_to_move_before: board.white_to_move,
                can_white_castle_kingside_before: board.can_white_castle_kingside,
                can_white_castle_queenside_before: board.can_white_castle_queenside,
                can_black_castle_kingside_before: board.can_black_castle_kingside,
                can_black_castle_queenside_before: board.can_black_castle_queenside,
                en_passant_square_before: board.en_passant_square,
                half_move_capture_or_pawn_clock_before: board.half_move_capture_or_pawn_clock,
                full_move_number_before: board.full_move_number,
                zobrist_key_before: key,
            };

            make_move(board, &mv);
            let score = -self.alpha_beta(board, depth - 1, -beta, -alpha);
            unmake_move(board, &mv, &undo);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            if score > alpha {
                alpha = score;
            }
        }

        Some(best_move)
    }
}

impl Clone for Search {
    fn clone(&self) -> Self {
        Search {
            tt: self.tt.clone(),
            killer_moves: self.killer_moves.clone(),
            search_start_time: std::time::Instant::now(), // reset
            move_time_limit: self.move_time_limit,
        }
    }
}

pub fn move_to_uci(mv: &ChessMove) -> String {
    use crate::movegen::{col_of, row_of};
    let from_file = (col_of(mv.from) as u8 + b'a') as char;
    let from_rank = (7 - row_of(mv.from)) as u8 + b'1';
    let to_file = (col_of(mv.to) as u8 + b'a') as char;
    let to_rank = (7 - row_of(mv.to)) as u8 + b'1';
    let mut s = format!(
        "{}{}{}{}",
        from_file as char, from_rank as char, to_file as char, to_rank as char
    );
    if mv.promoted_piece != Piece::Empty {
        let c = match mv.promoted_piece {
            Piece::WQ | Piece::BQ => 'q',
            Piece::WR | Piece::BR => 'r',
            Piece::WN | Piece::BN => 'n',
            Piece::WB | Piece::BB => 'b',
            _ => '?',
        };
        s.push(c);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    use crate::params::PositionParams;
    use crate::position_utils::set_board_position;

    #[test]
    fn find_best_move_white_mate_in_one() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: true,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "4k3/8/4K3/8/8/8/8/7R w - - 0 1".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 2, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "h1h8");
    }

    #[test]
    fn find_best_move_black_mate_in_one() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: false,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "7r/8/8/8/8/4k3/8/4K3 b - - 0 1".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 2, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "h8h1");
    }

    #[test]
    fn find_best_move_hanging_queen() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: true,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "rnb1kbnr/pppp1ppp/8/4p1q1/4P3/5N2/PPPP1PPP/RNBQKB1R w - - 0 1".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 2, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "f3g5");
    }

    #[test]
    fn find_best_move_knight_fork() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: true,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "8/4k3/7q/8/8/4N3/4K3/4R3 w - - 0 1".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 4, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "e3f5");
    }

    #[test]
    fn find_best_move_queen_sac_smothered_mate() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: true,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "r6k/1p1b1Qbp/1n2B1pN/p7/Pq6/8/1P4PP/R6K w - - 1 27".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 4, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "f7g8");
    }

    #[test]
    fn find_best_move_opening_hanging_bishop() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: false,
            can_white_castle_kingside: true,
            can_white_castle_queenside: true,
            can_black_castle_kingside: true,
            can_black_castle_queenside: true,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        let params = PositionParams {
            is_fen: true,
            position: "rnb1kbnr/ppqppppp/2p5/1B6/3PP3/2P5/PP3PPP/RNBQK1NR b KQkq - 0 1".to_string(),
            moves: vec![],
        };
        set_board_position(&mut board, &params).unwrap();

        let mut search = Search::new();
        let stop = AtomicBool::new(false);

        let best_move = search.find_best_move(&mut board, 3, &stop, None);
        assert!(best_move.is_some());
        assert_eq!(move_to_uci(&best_move.unwrap()), "c6b5");
    }
}
