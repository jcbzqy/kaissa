use crate::board::{Board, Piece};
use crate::chessmove::{ChessMove, UndoInfo};

const KNIGHT_OFFSETS: [i32; 8] = [-17, -15, -10, -6, 6, 10, 15, 17];
const KING_OFFSETS: [i32; 8] = [-9, -8, -7, -1, 1, 7, 8, 9];
const BISHOP_DIRECTIONS: [i32; 4] = [-9, -7, 7, 9];
const ROOK_DIRECTIONS: [i32; 4] = [-8, -1, 1, 8];

#[inline]
pub fn to_index(row: i32, col: i32) -> i32 {
    row * 8 + col
}

#[inline]
pub fn row_of(index: i32) -> i32 {
    index / 8
}

#[inline]
pub fn col_of(index: i32) -> i32 {
    index % 8
}

#[inline]
fn on_board(index: i32) -> bool {
    (0..64).contains(&index)
}

#[inline]
fn is_white_piece(p: Piece) -> bool {
    matches!(
        p,
        Piece::WP | Piece::WN | Piece::WB | Piece::WR | Piece::WQ | Piece::WK
    )
}

#[inline]
fn is_black_piece(p: Piece) -> bool {
    matches!(
        p,
        Piece::BP | Piece::BN | Piece::BB | Piece::BR | Piece::BQ | Piece::BK
    )
}

pub fn is_king_in_check(board: &Board, white_king: bool) -> bool {
    let king_square = find_king_square(board, white_king);
    if king_square == -1 {
        return false;
    }
    is_square_attacked(board, king_square, !white_king)
}

pub fn is_checkmate(board: &Board, white_to_move: bool) -> bool {
    if is_king_in_check(board, white_to_move) {
        let moves = generate_legal_moves(board);
        moves.is_empty()
    } else {
        false
    }
}

pub fn is_stalemate(board: &Board, white_to_move: bool) -> bool {
    if !is_king_in_check(board, white_to_move) {
        let moves = generate_legal_moves(board);
        moves.is_empty()
    } else {
        false
    }
}

pub fn generate_legal_moves(board: &Board) -> Vec<ChessMove> {
    let pseudo_legal_moves = generate_pseudo_legal_moves(board);
    let mut legal_moves = Vec::with_capacity(pseudo_legal_moves.len());

    for mv in pseudo_legal_moves {
        let mut temp_board = board.clone();

        let undo_info = UndoInfo {
            the_move: mv,
            piece_moved: temp_board.board[mv.from as usize],
            white_to_move_before: temp_board.white_to_move,
            can_white_castle_kingside_before: temp_board.can_white_castle_kingside,
            can_white_castle_queenside_before: temp_board.can_white_castle_queenside,
            can_black_castle_kingside_before: temp_board.can_black_castle_kingside,
            can_black_castle_queenside_before: temp_board.can_black_castle_queenside,
            en_passant_square_before: temp_board.en_passant_square,
            half_move_capture_or_pawn_clock_before: temp_board.half_move_capture_or_pawn_clock,
            full_move_number_before: temp_board.full_move_number,
            zobrist_key_before: temp_board.compute_zobrist_key(),
        };

        make_move(&mut temp_board, &mv);

        let side_that_just_moved = !temp_board.white_to_move;
        if !is_king_in_check(&temp_board, side_that_just_moved) {
            legal_moves.push(mv);
        }

        drop(undo_info);
    }

    legal_moves
}

pub fn make_move(board: &mut Board, mv: &ChessMove) {
    let moving_piece = board.board[mv.from as usize];
    board.board[mv.from as usize] = Piece::Empty;

    if mv.is_en_passant {
        let direction = if moving_piece == Piece::WP { -8 } else { 8 };
        board.board[(mv.to + direction) as usize] = Piece::Empty;
    }

    if mv.captured_piece != Piece::Empty && !mv.is_en_passant {
        board.board[mv.to as usize] = Piece::Empty;
    }

    if mv.is_castle {
        let king_side = col_of(mv.to) == 6;
        if moving_piece == Piece::WK {
            if king_side {
                board.board[to_index(7, 7) as usize] = Piece::Empty;
                board.board[to_index(7, 5) as usize] = Piece::WR;
            } else {
                board.board[to_index(7, 0) as usize] = Piece::Empty;
                board.board[to_index(7, 3) as usize] = Piece::WR;
            }
        } else if moving_piece == Piece::BK {
            if king_side {
                board.board[to_index(0, 7) as usize] = Piece::Empty;
                board.board[to_index(0, 5) as usize] = Piece::BR;
            } else {
                board.board[to_index(0, 0) as usize] = Piece::Empty;
                board.board[to_index(0, 3) as usize] = Piece::BR;
            }
        }
    }

    if mv.promoted_piece != Piece::Empty {
        board.board[mv.to as usize] = mv.promoted_piece;
    } else {
        board.board[mv.to as usize] = moving_piece;
    }

    update_castling_rights(board, moving_piece, mv);

    if moving_piece == Piece::WP && (mv.to - mv.from == -16) {
        board.en_passant_square = mv.from - 8;
    } else if moving_piece == Piece::BP && (mv.to - mv.from == 16) {
        board.en_passant_square = mv.from + 8;
    } else {
        board.en_passant_square = -1;
    }

    if moving_piece == Piece::WP || moving_piece == Piece::BP || mv.captured_piece != Piece::Empty {
        board.half_move_capture_or_pawn_clock = 0;
    } else {
        board.half_move_capture_or_pawn_clock += 1;
    }

    board.white_to_move = !board.white_to_move;
    if !board.white_to_move {
        board.full_move_number += 1;
    }
}

fn generate_pseudo_legal_moves(board: &Board) -> Vec<ChessMove> {
    let mut moves = Vec::new();
    for square in 0..64 {
        let piece = board.board[square];
        if piece == Piece::Empty {
            continue;
        }
        if board.white_to_move && is_white_piece(piece) {
            generate_piece_moves(board, square as i32, &mut moves);
        } else if !board.white_to_move && is_black_piece(piece) {
            generate_piece_moves(board, square as i32, &mut moves);
        }
    }
    generate_castling_moves(board, &mut moves);
    moves
}

fn generate_piece_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let p = board.board[square as usize];
    match p {
        Piece::WP | Piece::BP => generate_pawn_moves(board, square, moves_out),
        Piece::WN | Piece::BN => generate_knight_moves(board, square, moves_out),
        Piece::WB | Piece::BB => generate_bishop_moves(board, square, moves_out),
        Piece::WR | Piece::BR => generate_rook_moves(board, square, moves_out),
        Piece::WQ | Piece::BQ => {
            generate_bishop_moves(board, square, moves_out);
            generate_rook_moves(board, square, moves_out);
        }
        Piece::WK | Piece::BK => generate_king_moves(board, square, moves_out),
        Piece::Empty => {}
    }
}

fn generate_pawn_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let p = board.board[square as usize];
    let white = p == Piece::WP;
    let forward = if white { -8 } else { 8 };
    let start_rank = if white { 6 } else { 1 };
    let promotion_rank = if white { 0 } else { 7 };

    let forward_one = square + forward;
    if on_board(forward_one) && board.board[forward_one as usize] == Piece::Empty {
        if row_of(forward_one) == promotion_rank {
            for &promo_piece in &promotion_pieces(white) {
                moves_out.push(ChessMove {
                    from: square,
                    to: forward_one,
                    promoted_piece: promo_piece,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: false,
                });
            }
        } else {
            moves_out.push(ChessMove {
                from: square,
                to: forward_one,
                promoted_piece: Piece::Empty,
                captured_piece: Piece::Empty,
                is_en_passant: false,
                is_castle: false,
            });
            if row_of(square) == start_rank {
                let forward_two = forward_one + forward;
                if on_board(forward_two) && board.board[forward_two as usize] == Piece::Empty {
                    moves_out.push(ChessMove {
                        from: square,
                        to: forward_two,
                        promoted_piece: Piece::Empty,
                        captured_piece: Piece::Empty,
                        is_en_passant: false,
                        is_castle: false,
                    });
                }
            }
        }
    }

    for &dc in &[-1, 1] {
        let capture_col = col_of(square) + dc;
        if capture_col < 0 || capture_col > 7 {
            continue;
        }
        let capture_square = square + forward + dc;
        if on_board(capture_square) {
            let target_piece = board.board[capture_square as usize];
            if is_enemy_piece(p, target_piece) {
                if row_of(capture_square) == promotion_rank {
                    for &promo_piece in &promotion_pieces(white) {
                        moves_out.push(ChessMove {
                            from: square,
                            to: capture_square,
                            promoted_piece: promo_piece,
                            captured_piece: target_piece,
                            is_en_passant: false,
                            is_castle: false,
                        });
                    }
                } else {
                    moves_out.push(ChessMove {
                        from: square,
                        to: capture_square,
                        promoted_piece: Piece::Empty,
                        captured_piece: target_piece,
                        is_en_passant: false,
                        is_castle: false,
                    });
                }
            }
            if board.en_passant_square >= 0
                && board.en_passant_square < 64
                && capture_square == board.en_passant_square
            {
                moves_out.push(ChessMove {
                    from: square,
                    to: capture_square,
                    promoted_piece: Piece::Empty,
                    captured_piece: if white { Piece::BP } else { Piece::WP },
                    is_en_passant: true,
                    is_castle: false,
                });
            }
        }
    }
}

fn promotion_pieces(white: bool) -> [Piece; 4] {
    if white {
        [Piece::WQ, Piece::WR, Piece::WB, Piece::WN]
    } else {
        [Piece::BQ, Piece::BR, Piece::BB, Piece::BN]
    }
}

fn is_enemy_piece(p: Piece, target: Piece) -> bool {
    if target == Piece::Empty {
        return false;
    }
    (is_white_piece(p) && is_black_piece(target)) || (is_black_piece(p) && is_white_piece(target))
}

fn generate_knight_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let from_row = row_of(square);
    let from_col = col_of(square);
    let piece = board.board[square as usize];
    for &offset in KNIGHT_OFFSETS.iter() {
        let target = square + offset;
        if on_board(target) {
            let to_row = row_of(target);
            let to_col = col_of(target);
            let drow = (to_row - from_row).abs();
            let dcol = (to_col - from_col).abs();
            // This check is from the original logic
            if (drow == 2 && dcol == 1) || (drow == 1 && dcol == 2) {
                let tgt_piece = board.board[target as usize];
                if tgt_piece == Piece::Empty || is_enemy_piece(piece, tgt_piece) {
                    moves_out.push(ChessMove {
                        from: square,
                        to: target,
                        promoted_piece: Piece::Empty,
                        captured_piece: if tgt_piece != Piece::Empty {
                            tgt_piece
                        } else {
                            Piece::Empty
                        },
                        is_en_passant: false,
                        is_castle: false,
                    });
                }
            }
        }
    }
}

fn generate_bishop_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let piece = board.board[square as usize];
    for &d in BISHOP_DIRECTIONS.iter() {
        let mut current = square;
        loop {
            let next = current + d;
            if !on_board(next) {
                break;
            }
            if (row_of(next) - row_of(current)).abs() != 1
                || (col_of(next) - col_of(current)).abs() != 1
            {
                break;
            }
            let tgt = board.board[next as usize];
            if tgt == Piece::Empty {
                moves_out.push(ChessMove {
                    from: square,
                    to: next,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: false,
                });
            } else {
                if is_enemy_piece(piece, tgt) {
                    moves_out.push(ChessMove {
                        from: square,
                        to: next,
                        promoted_piece: Piece::Empty,
                        captured_piece: tgt,
                        is_en_passant: false,
                        is_castle: false,
                    });
                }
                break;
            }
            current = next;
        }
    }
}

fn generate_rook_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let piece = board.board[square as usize];
    for &d in ROOK_DIRECTIONS.iter() {
        let mut current = square;
        loop {
            let next = current + d;
            if !on_board(next) {
                break;
            }
            if (d == -1 || d == 1) && row_of(current) != row_of(next) {
                break;
            }
            if (d == -8 || d == 8) && col_of(current) != col_of(next) {
                break;
            }
            let tgt = board.board[next as usize];
            if tgt == Piece::Empty {
                moves_out.push(ChessMove {
                    from: square,
                    to: next,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: false,
                });
            } else {
                if is_enemy_piece(piece, tgt) {
                    moves_out.push(ChessMove {
                        from: square,
                        to: next,
                        promoted_piece: Piece::Empty,
                        captured_piece: tgt,
                        is_en_passant: false,
                        is_castle: false,
                    });
                }
                break;
            }
            current = next;
        }
    }
}

fn generate_king_moves(board: &Board, square: i32, moves_out: &mut Vec<ChessMove>) {
    let piece = board.board[square as usize];
    for &offset in KING_OFFSETS.iter() {
        let target = square + offset;
        if !on_board(target) {
            continue;
        }
        if (col_of(target) - col_of(square)).abs() > 1 {
            continue;
        }
        let tgt_piece = board.board[target as usize];
        if tgt_piece == Piece::Empty || is_enemy_piece(piece, tgt_piece) {
            moves_out.push(ChessMove {
                from: square,
                to: target,
                promoted_piece: Piece::Empty,
                captured_piece: if tgt_piece != Piece::Empty {
                    tgt_piece
                } else {
                    Piece::Empty
                },
                is_en_passant: false,
                is_castle: false,
            });
        }
    }
}

fn generate_castling_moves(board: &Board, moves_out: &mut Vec<ChessMove>) {
    let white_to_move = board.white_to_move;
    if white_to_move {
        let king_square = find_king_square(board, true);
        if king_square == -1 {
            return;
        }
        if board.can_white_castle_kingside {
            let f1 = to_index(7, 5);
            let g1 = to_index(7, 6);
            if board.board[f1 as usize] == Piece::Empty && board.board[g1 as usize] == Piece::Empty
            {
                moves_out.push(ChessMove {
                    from: king_square,
                    to: g1,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: true,
                });
            }
        }
        if board.can_white_castle_queenside {
            let d1 = to_index(7, 3);
            let c1 = to_index(7, 2);
            let b1 = to_index(7, 1);
            if board.board[d1 as usize] == Piece::Empty
                && board.board[c1 as usize] == Piece::Empty
                && board.board[b1 as usize] == Piece::Empty
            {
                moves_out.push(ChessMove {
                    from: king_square,
                    to: c1,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: true,
                });
            }
        }
    } else {
        let king_square = find_king_square(board, false);
        if king_square == -1 {
            return;
        }
        if board.can_black_castle_kingside {
            let f8 = to_index(0, 5);
            let g8 = to_index(0, 6);
            if board.board[f8 as usize] == Piece::Empty && board.board[g8 as usize] == Piece::Empty
            {
                moves_out.push(ChessMove {
                    from: king_square,
                    to: g8,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: true,
                });
            }
        }
        if board.can_black_castle_queenside {
            let d8 = to_index(0, 3);
            let c8 = to_index(0, 2);
            let b8 = to_index(0, 1);
            if board.board[d8 as usize] == Piece::Empty
                && board.board[c8 as usize] == Piece::Empty
                && board.board[b8 as usize] == Piece::Empty
            {
                moves_out.push(ChessMove {
                    from: king_square,
                    to: c8,
                    promoted_piece: Piece::Empty,
                    captured_piece: Piece::Empty,
                    is_en_passant: false,
                    is_castle: true,
                });
            }
        }
    }
}

fn update_castling_rights(board: &mut Board, moving_piece: Piece, mv: &ChessMove) {
    if moving_piece == Piece::WK {
        board.can_white_castle_kingside = false;
        board.can_white_castle_queenside = false;
    } else if moving_piece == Piece::BK {
        board.can_black_castle_kingside = false;
        board.can_black_castle_queenside = false;
    }

    disable_rook_castle(board, mv.from);
    if mv.captured_piece != Piece::Empty {
        disable_rook_castle(board, mv.to);
    }
}

fn disable_rook_castle(board: &mut Board, sq: i32) {
    match sq {
        56 => board.can_white_castle_queenside = false,
        63 => board.can_white_castle_kingside = false,
        0 => board.can_black_castle_queenside = false,
        7 => board.can_black_castle_kingside = false,
        _ => {}
    }
}

fn find_king_square(board: &Board, white_king: bool) -> i32 {
    let wanted = if white_king { Piece::WK } else { Piece::BK };
    for i in 0..64 {
        if board.board[i] == wanted {
            return i as i32;
        }
    }
    -1
}

fn is_square_attacked(board: &Board, square: i32, attacked_by_white: bool) -> bool {
    // Pawn attacks
    if attacked_by_white {
        let r = row_of(square);
        let c = col_of(square);
        if r < 7 && c > 0 {
            let sq = square + 7;
            if on_board(sq) && board.board[sq as usize] == Piece::WP {
                return true;
            }
        }
        if r < 7 && c < 7 {
            let sq = square + 9;
            if on_board(sq) && board.board[sq as usize] == Piece::WP {
                return true;
            }
        }
    } else {
        let r = row_of(square);
        let c = col_of(square);
        if r > 0 && c > 0 {
            let sq = square - 9;
            if on_board(sq) && board.board[sq as usize] == Piece::BP {
                return true;
            }
        }
        if r > 0 && c < 7 {
            let sq = square - 7;
            if on_board(sq) && board.board[sq as usize] == Piece::BP {
                return true;
            }
        }
    }

    for &offset in KNIGHT_OFFSETS.iter() {
        let knight_sq = square + offset;
        if on_board(knight_sq) {
            let piece = board.board[knight_sq as usize];
            if attacked_by_white {
                if piece == Piece::WN {
                    return true;
                }
            } else {
                if piece == Piece::BN {
                    return true;
                }
            }
        }
    }

    for &offset in KING_OFFSETS.iter() {
        let king_sq = square + offset;
        if on_board(king_sq)
            && (row_of(king_sq) - row_of(square)).abs() <= 1
            && (col_of(king_sq) - col_of(square)).abs() <= 1
        {
            let piece = board.board[king_sq as usize];
            if attacked_by_white {
                if piece == Piece::WK {
                    return true;
                }
            } else {
                if piece == Piece::BK {
                    return true;
                }
            }
        }
    }

    if check_diagonal_attack(board, square, attacked_by_white) {
        return true;
    }
    if check_straight_attack(board, square, attacked_by_white) {
        return true;
    }

    false
}

fn check_diagonal_attack(board: &Board, square: i32, white: bool) -> bool {
    for &d in &BISHOP_DIRECTIONS {
        let mut current = square;
        loop {
            let next = current + d;
            if !on_board(next) {
                break;
            }
            if (row_of(next) - row_of(current)).abs() != 1
                || (col_of(next) - col_of(current)).abs() != 1
            {
                break;
            }
            let p = board.board[next as usize];
            if p != Piece::Empty {
                if white {
                    if p == Piece::WB || p == Piece::WQ {
                        return true;
                    }
                } else {
                    if p == Piece::BB || p == Piece::BQ {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }
    false
}

fn check_straight_attack(board: &Board, square: i32, white: bool) -> bool {
    for &d in &ROOK_DIRECTIONS {
        let mut current = square;
        loop {
            let next = current + d;
            if !on_board(next) {
                break;
            }
            if (d == -1 || d == 1) && row_of(next) != row_of(current) {
                break;
            }
            if (d == -8 || d == 8) && col_of(next) != col_of(current) {
                break;
            }
            let p = board.board[next as usize];
            if p != Piece::Empty {
                if white {
                    if p == Piece::WR || p == Piece::WQ {
                        return true;
                    }
                } else {
                    if p == Piece::BR || p == Piece::BQ {
                        return true;
                    }
                }
                break;
            }
            current = next;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    fn set_starting_position(board: &mut Board) {
        board.board = [Piece::Empty; 64];

        board.board[0] = Piece::BR;
        board.board[1] = Piece::BN;
        board.board[2] = Piece::BB;
        board.board[3] = Piece::BQ;
        board.board[4] = Piece::BK;
        board.board[5] = Piece::BB;
        board.board[6] = Piece::BN;
        board.board[7] = Piece::BR;
        for file in 0..8 {
            board.board[8 + file] = Piece::BP;
        }

        for file in 0..8 {
            board.board[48 + file] = Piece::WP;
        }
        board.board[56] = Piece::WR;
        board.board[57] = Piece::WN;
        board.board[58] = Piece::WB;
        board.board[59] = Piece::WQ;
        board.board[60] = Piece::WK;
        board.board[61] = Piece::WB;
        board.board[62] = Piece::WN;
        board.board[63] = Piece::WR;

        board.white_to_move = true;
        board.can_white_castle_kingside = true;
        board.can_white_castle_queenside = true;
        board.can_black_castle_kingside = true;
        board.can_black_castle_queenside = true;
        board.en_passant_square = -1;
        board.half_move_capture_or_pawn_clock = 0;
        board.full_move_number = 1;
    }

    #[test]
    fn test_starting_position_move_count() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: true,
            can_white_castle_kingside: true,
            can_white_castle_queenside: true,
            can_black_castle_kingside: true,
            can_black_castle_queenside: true,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 0,
            full_move_number: 1,
        };
        set_starting_position(&mut board);

        let moves = generate_legal_moves(&board);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn test_two_corner_king_position_move_count() {
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

        board.board[0] = Piece::BK;
        board.board[63] = Piece::WK;

        let moves = generate_legal_moves(&board);
        assert_eq!(moves.len(), 3);
    }

    #[test]
    fn test_most_possible_moves_position_move_count() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: false, // 'b' => black to move
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: -1,
            half_move_capture_or_pawn_clock: 1,
            full_move_number: 59,
        };
        board.board[to_index(0, 3) as usize] = Piece::BQ; // d8
        board.board[to_index(0, 7) as usize] = Piece::BR; // h8
        board.board[to_index(1, 6) as usize] = Piece::WK; // g7
        board.board[to_index(2, 2) as usize] = Piece::BN; // c6
        board.board[to_index(2, 4) as usize] = Piece::BB; // e6
        board.board[to_index(3, 2) as usize] = Piece::BQ; // c5
        board.board[to_index(3, 4) as usize] = Piece::BK; // e5
        board.board[to_index(4, 4) as usize] = Piece::BN; // e4
        board.board[to_index(6, 0) as usize] = Piece::BR; // a2
        board.board[to_index(7, 0) as usize] = Piece::BQ; // a1
        board.board[to_index(7, 1) as usize] = Piece::BQ; // b1
        board.board[to_index(7, 3) as usize] = Piece::BQ; // d1
        board.board[to_index(7, 4) as usize] = Piece::BQ; // e1
        board.board[to_index(7, 5) as usize] = Piece::BQ; // f1
        board.board[to_index(7, 7) as usize] = Piece::BQ; // h1

        let moves = generate_legal_moves(&board);
        assert_eq!(moves.len(), 147);
    }
}
