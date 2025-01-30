#![allow(dead_code)]

mod board;
mod chessmove;
mod engine;
mod movegen;
mod params;
mod position_utils;
mod search;
mod uci;

fn main() {
    let mut engine = engine::Engine::new();
    let mut uci = uci::UCI::new(&mut engine);
    uci.main_loop();
}
