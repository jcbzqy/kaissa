use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

use crate::board::Board;
use crate::chessmove::ChessMove;
use crate::params::{GoParams, PositionParams};
use crate::position_utils::set_board_position;
use crate::search::move_to_uci;
use crate::search::Search;

pub struct Engine {
    board: Board,
    search: Search,

    stop_requested: Arc<AtomicBool>,

    search_thread: Option<JoinHandle<()>>,

    best_move: Arc<Mutex<Option<ChessMove>>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            board: Board {
                board: [crate::board::Piece::Empty; 64],
                white_to_move: true,
                can_white_castle_kingside: true,
                can_white_castle_queenside: true,
                can_black_castle_kingside: true,
                can_black_castle_queenside: true,
                en_passant_square: -1,
                half_move_capture_or_pawn_clock: 0,
                full_move_number: 1,
            },
            search: Search::new(),
            stop_requested: Arc::new(AtomicBool::new(false)),
            search_thread: None,
            best_move: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_position(&mut self, params: &PositionParams) {
        set_board_position(&mut self.board, params).expect("Invalid position command");
    }

    pub fn go(&mut self, params: &GoParams) {
        self.stop();
        self.stop_requested.store(false, Ordering::Relaxed);

        let board_clone = self.board.clone();
        let stop_clone = Arc::clone(&self.stop_requested);
        let best_move_clone = Arc::clone(&self.best_move);
        let depth = params.depth.unwrap_or(5);
        let movetime = params.movetime;

        let mut search_obj = self.search.clone();

        self.search_thread = Some(thread::spawn(move || {
            let result =
                search_obj.find_best_move(&mut board_clone.clone(), depth, &stop_clone, movetime);
            if let Some(m) = result {
                {
                    let mut locked = best_move_clone.lock().unwrap();
                    *locked = Some(m);
                }
                println!("bestmove {}", move_to_uci(&m));
            } else {
                println!("bestmove 0000");
            }
        }));
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::Relaxed);
        if let Some(handle) = self.search_thread.take() {
            let _ = handle.join();
        }
    }

    pub fn get_best_move(&self) -> Option<ChessMove> {
        self.best_move.lock().unwrap().clone()
    }
}
