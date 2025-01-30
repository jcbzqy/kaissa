use crate::board::{Board, Piece};
use crate::movegen::{generate_legal_moves, make_move};
use crate::params::PositionParams;

pub fn set_to_starting_position(board: &mut Board) {
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

pub fn parse_fen(board: &mut Board, fen: String) -> Result<(), String> {
    let tokens: Vec<&str> = fen.split_whitespace().collect();
    if tokens.len() < 6 {
        return Err("FEN must have at least 6 parts: \
        [pieces] [side] [castling] [enpassant] [halfmove] [fullmove]"
            .to_string());
    }

    board.board = [Piece::Empty; 64];
    let ranks: Vec<&str> = tokens[0].split('/').collect();
    if ranks.len() != 8 {
        return Err("Invalid FEN: expected 8 ranks in piece placement".to_string());
    }
    for (row, rank_data) in ranks.iter().enumerate() {
        let mut file = 0usize;
        for ch in rank_data.chars() {
            if ch.is_ascii_digit() {
                let skip = ch.to_digit(10).unwrap();
                for _ in 0..skip {
                    board.board[row * 8 + file] = Piece::Empty;
                    file += 1;
                }
            } else {
                let p = char_to_piece(ch)?;
                board.board[row * 8 + file] = p;
                file += 1;
            }
        }
        if file != 8 {
            return Err("Invalid FEN: rank does not have 8 squares".to_string());
        }
    }

    board.white_to_move = match tokens[1] {
        "w" => true,
        "b" => false,
        _ => return Err(format!("Invalid FEN side to move: {}", tokens[1])),
    };

    let castling = tokens[2];
    board.can_white_castle_kingside = castling.contains('K');
    board.can_white_castle_queenside = castling.contains('Q');
    board.can_black_castle_kingside = castling.contains('k');
    board.can_black_castle_queenside = castling.contains('q');

    let ep = tokens[3];
    if ep == "-" {
        board.en_passant_square = -1;
    } else {
        let sq = algebraic_to_square(ep)?;
        board.en_passant_square = sq as i32;
    }

    board.half_move_capture_or_pawn_clock = tokens[4]
        .parse::<i32>()
        .map_err(|e| format!("Invalid halfmove clock: {}", e))?;

    board.full_move_number = tokens[5]
        .parse::<i32>()
        .map_err(|e| format!("Invalid fullmove number: {}", e))?;

    Ok(())
}

fn char_to_piece(ch: char) -> Result<Piece, String> {
    Ok(match ch {
        'p' => Piece::BP,
        'n' => Piece::BN,
        'b' => Piece::BB,
        'r' => Piece::BR,
        'q' => Piece::BQ,
        'k' => Piece::BK,
        'P' => Piece::WP,
        'N' => Piece::WN,
        'B' => Piece::WB,
        'R' => Piece::WR,
        'Q' => Piece::WQ,
        'K' => Piece::WK,
        _ => return Err(format!("Unknown piece character in FEN: {}", ch)),
    })
}

fn algebraic_to_square(s: &str) -> Result<usize, String> {
    if s.len() != 2 {
        return Err(format!("Algebraic square must have 2 chars: got {}", s));
    }
    let file_char = s.chars().next().unwrap();
    let rank_char = s.chars().nth(1).unwrap();
    if !(('a'..='h').contains(&file_char)) {
        return Err(format!("File must be a..h: got '{}'", file_char));
    }
    if !(('1'..='8').contains(&rank_char)) {
        return Err(format!("Rank must be 1..8: got '{}'", rank_char));
    }
    let col = (file_char as u8 - b'a') as usize;
    let row = (rank_char as u8 - b'1') as usize;
    let board_row = 7 - row;
    let index = board_row * 8 + col;
    Ok(index)
}

pub fn set_board_position(board: &mut Board, params: &PositionParams) -> Result<(), String> {
    if params.is_fen {
        parse_fen(board, params.position.clone())?;
    } else {
        set_to_starting_position(board);
    }
    for mv_str in &params.moves {
        if mv_str.len() < 4 {
            return Err(format!("Invalid move string '{}': too short", mv_str));
        }
        let from_sq = algebraic_to_square(&mv_str[0..2])? as i32;
        let to_sq = algebraic_to_square(&mv_str[2..4])? as i32;
        let mut promo_piece = Piece::Empty;
        if mv_str.len() == 5 {
            promo_piece = match mv_str.chars().nth(4).unwrap().to_ascii_lowercase() {
                'q' => {
                    if board.white_to_move {
                        Piece::WQ
                    } else {
                        Piece::BQ
                    }
                }
                'r' => {
                    if board.white_to_move {
                        Piece::WR
                    } else {
                        Piece::BR
                    }
                }
                'n' => {
                    if board.white_to_move {
                        Piece::WN
                    } else {
                        Piece::BN
                    }
                }
                'b' => {
                    if board.white_to_move {
                        Piece::WB
                    } else {
                        Piece::BB
                    }
                }
                x => return Err(format!("Invalid promotion char '{}'", x)),
            }
        }
        let legal_moves = generate_legal_moves(board);
        let found_move = legal_moves
            .iter()
            .find(|m| m.from == from_sq && m.to == to_sq && m.promoted_piece == promo_piece);
        if let Some(chess_mv) = found_move {
            make_move(board, chess_mv);
        } else {
            return Err(format!("Illegal move encountered: {}", mv_str));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Piece};

    #[test]
    fn test_startpos() {
        let mut board = Board {
            board: [Piece::Empty; 64],
            white_to_move: false,
            can_white_castle_kingside: false,
            can_white_castle_queenside: false,
            can_black_castle_kingside: false,
            can_black_castle_queenside: false,
            en_passant_square: 0,
            half_move_capture_or_pawn_clock: 7,
            full_move_number: 66,
        };
        set_to_starting_position(&mut board);
        assert_eq!(board.board[0], Piece::BR);
        assert_eq!(board.board[1], Piece::BN);
        assert_eq!(board.board[2], Piece::BB);
        assert_eq!(board.board[3], Piece::BQ);
        assert_eq!(board.board[4], Piece::BK);
        assert_eq!(board.board[5], Piece::BB);
        assert_eq!(board.board[6], Piece::BN);
        assert_eq!(board.board[7], Piece::BR);
        assert_eq!(board.board[8], Piece::BP);
        for file in 0..8 {
            assert_eq!(board.board[8 + file], Piece::BP);
        }
        for file in 0..8 {
            assert_eq!(board.board[48 + file], Piece::WP);
        }
        assert_eq!(board.board[56], Piece::WR);
        assert_eq!(board.board[57], Piece::WN);
        assert_eq!(board.board[58], Piece::WB);
        assert_eq!(board.board[59], Piece::WQ);
        assert_eq!(board.board[60], Piece::WK);
        assert_eq!(board.board[61], Piece::WB);
        assert_eq!(board.board[62], Piece::WN);
        assert_eq!(board.board[63], Piece::WR);
        assert_eq!(board.white_to_move, true);
        assert_eq!(board.can_white_castle_kingside, true);
        assert_eq!(board.can_black_castle_queenside, true);
        assert_eq!(board.en_passant_square, -1);
        assert_eq!(board.full_move_number, 1);
    }

    #[test]
    fn test_parse_fen_basic() {
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
        let fen = "5rk1/pp4pp/4p3/2R3Q1/3n4/2q4r/P1P2PPP/5RK1 b - - 1 23".to_string();
        let res = parse_fen(&mut board, fen);
        assert!(res.is_ok());
        assert_eq!(board.white_to_move, false);
        assert_eq!(board.can_white_castle_kingside, false);
        assert_eq!(board.can_white_castle_queenside, false);
        assert_eq!(board.can_black_castle_kingside, false);
        assert_eq!(board.can_black_castle_queenside, false);
        assert_eq!(board.en_passant_square, -1);
        assert_eq!(board.half_move_capture_or_pawn_clock, 1);
        assert_eq!(board.full_move_number, 23);
    }

    #[test]
    fn test_set_board_position_with_moves() {
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
            position: "5rk1/pp4pp/4p3/2R3Q1/3n4/2q4r/P1P2PPP/5RK1 b - - 1 23".to_string(),
            moves: vec!["c3g3", "g5g3", "d4e2", "g1h1", "e2g3", "f2g3", "f8f1"]
                .into_iter()
                .map(String::from)
                .collect(),
        };
        let result = set_board_position(&mut board, &params);
        assert!(result.is_ok());
        assert_eq!(board.white_to_move, true);
        assert_eq!(board.can_white_castle_kingside, false);
        assert_eq!(board.can_white_castle_queenside, false);
        assert_eq!(board.can_black_castle_kingside, false);
        assert_eq!(board.can_black_castle_queenside, false);
        assert_eq!(board.en_passant_square, -1);
        assert_eq!(board.half_move_capture_or_pawn_clock, 0);
        assert_eq!(board.full_move_number, 26);
    }
}
