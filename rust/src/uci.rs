use std::io::{self, BufRead};
use std::str::FromStr;

use crate::engine::Engine;
use crate::params::{GoParams, OptionParams, PositionParams, SearchInfo};
use std::time::Duration;

pub struct UCI<'a> {
    running: bool,
    engine: &'a mut Engine,
}

impl<'a> UCI<'a> {
    pub fn new(engine: &'a mut Engine) -> Self {
        UCI {
            running: false,
            engine,
        }
    }

    pub fn main_loop(&mut self) {
        self.running = true;
        let stdin = io::stdin();

        for line_result in stdin.lock().lines() {
            if !self.running {
                break;
            }

            let line = match line_result {
                Ok(s) => s.trim().to_string(),
                Err(_) => break,
            };

            let recognized = self.process_command(&line);
            if !recognized {
                eprintln!("Unknown command: {}", line);
            }
        }
    }

    fn process_command(&mut self, cmd: &str) -> bool {
        if cmd == "uci" {
            self.uci_command();
        } else if cmd == "isready" {
            self.is_ready_command();
        } else if cmd == "quit" {
            self.quit_command();
        } else if cmd == "ucinewgame" {
            self.uci_new_game_command();
        } else if cmd.starts_with("debug") {
            let on = cmd.contains("on");
            self.debug_command(on);
        } else if cmd.starts_with("setoption") {
            let option_params = self.parse_option_command(&cmd["setoption".len()..]);
            self.set_option_command(&option_params.name, &option_params.value);
        } else if cmd.starts_with("register") {
            self.register_command(&cmd["register".len()..]);
        } else if cmd.starts_with("position") {
            let params = self.parse_position_command(&cmd["position".len()..]);
            self.position_command(params);
        } else if cmd.starts_with("go") {
            let go_params = self.parse_go_command(&cmd["go".len()..]);
            self.go_command(go_params);
        } else if cmd == "stop" {
            self.stop_command();
        } else if cmd == "ponderhit" {
            self.ponder_hit_command();
        } else {
            return false;
        }
        true
    }

    fn uci_command(&self) {
        println!("id name Kaissa");
        println!("id author kw");
        println!("uciok");
    }

    fn is_ready_command(&self) {
        println!("readyok");
    }

    fn quit_command(&mut self) {
        self.running = false;
    }

    fn uci_new_game_command(&self) {
        println!("ucinewgame unsupported");
    }

    fn debug_command(&self, on: bool) {
        println!("debug {}unsupported", if on { "on " } else { "off " });
    }

    fn stop_command(&mut self) {
        self.engine.stop();
    }

    fn ponder_hit_command(&self) {
        println!("ponder unsupported");
    }

    fn register_command(&self, _params: &str) {
        println!("registration ok");
    }

    fn set_option_command(&self, _name: &str, _value: &str) {
        println!("no options available");
    }

    fn position_command(&mut self, params: PositionParams) {
        self.engine.set_position(&params);
    }

    fn go_command(&mut self, params: GoParams) {
        self.engine.go(&params);
    }

    fn parse_position_command(&self, args: &str) -> PositionParams {
        let tokens: Vec<&str> = args.split_whitespace().collect();
        if tokens.is_empty() {
            return PositionParams {
                is_fen: false,
                position: "".to_string(),
                moves: vec![],
            };
        }

        let is_fen = tokens[0] == "fen";
        let mut params = PositionParams {
            is_fen,
            position: "".to_string(),
            moves: vec![],
        };

        if is_fen {
            let mut i = 1;
            let mut fen_string = String::new();
            while i < tokens.len() && tokens[i] != "moves" {
                if !fen_string.is_empty() {
                    fen_string.push(' ');
                }
                fen_string.push_str(tokens[i]);
                i += 1;
            }
            params.position = fen_string;
            if i < tokens.len() && tokens[i] == "moves" {
                i += 1;
                // Convert &str to String
                let the_moves = tokens[i..].iter().map(|m| m.to_string()).collect();
                params.moves = the_moves;
            }
        } else {
            params.is_fen = false;
            params.position = "startpos".to_string();
            let mut i = 1;
            if i < tokens.len() && tokens[i] == "moves" {
                i += 1;
                // Convert &str to String
                params.moves = tokens[i..].iter().map(|m| m.to_string()).collect();
            }
        }

        params
    }

    fn parse_go_command(&self, args: &str) -> GoParams {
        let tokens: Vec<&str> = args.split_whitespace().collect();
        let mut params = GoParams::default();

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "infinite" => {
                    params.infinite = true;
                    i += 1;
                }
                "wtime" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(ms) = tokens[i].parse::<u64>() {
                            params.wtime = Some(Duration::from_millis(ms));
                        }
                        i += 1;
                    }
                }
                "btime" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(ms) = tokens[i].parse::<u64>() {
                            params.btime = Some(Duration::from_millis(ms));
                        }
                        i += 1;
                    }
                }
                "winc" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(ms) = tokens[i].parse::<u64>() {
                            params.winc = Some(Duration::from_millis(ms));
                        }
                        i += 1;
                    }
                }
                "binc" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(ms) = tokens[i].parse::<u64>() {
                            params.binc = Some(Duration::from_millis(ms));
                        }
                        i += 1;
                    }
                }
                "movestogo" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(mt) = i32::from_str(tokens[i]) {
                            params.movestogo = Some(mt);
                        }
                        i += 1;
                    }
                }
                "depth" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(d) = i32::from_str(tokens[i]) {
                            params.depth = Some(d);
                        }
                        i += 1;
                    }
                }
                "nodes" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(n) = i32::from_str(tokens[i]) {
                            params.nodes = Some(n);
                        }
                        i += 1;
                    }
                }
                "mate" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(m) = i32::from_str(tokens[i]) {
                            params.mate = Some(m);
                        }
                        i += 1;
                    }
                }
                "movetime" => {
                    i += 1;
                    if i < tokens.len() {
                        if let Ok(ms) = tokens[i].parse::<u64>() {
                            params.movetime = Some(Duration::from_millis(ms));
                        }
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }

        params
    }

    fn parse_option_command(&self, args: &str) -> OptionParams {
        let tokens: Vec<&str> = args.split_whitespace().collect();
        let mut name = String::new();
        let mut value = String::new();

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "name" => {
                    i += 1;
                    if i < tokens.len() {
                        name = tokens[i].to_string();
                    }
                    i += 1;
                }
                "value" => {
                    i += 1;
                    if i < tokens.len() {
                        value = tokens[i].to_string();
                    }
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        OptionParams { name, value }
    }

    #[allow(dead_code)]
    fn send_option(
        &self,
        name: &str,
        kind: &str,
        default_value: Option<&str>,
        min: Option<&str>,
        max: Option<&str>,
    ) {
        print!("option name {} type {}", name, kind);
        if let Some(d) = default_value {
            print!(" default {}", d);
        }
        if let Some(mi) = min {
            print!(" min {}", mi);
        }
        if let Some(ma) = max {
            print!(" max {}", ma);
        }
        println!();
    }

    #[allow(dead_code)]
    fn send_best_move(&self, mv: &str, ponder: Option<&str>) {
        if let Some(pond) = ponder {
            println!("bestmove {} ponder {}", mv, pond);
        } else {
            println!("bestmove {}", mv);
        }
    }

    #[allow(dead_code)]
    fn send_info(&self, info: &SearchInfo) {
        print!("info");
        if let Some(d) = info.depth {
            print!(" depth {}", d);
        }
        if let Some(sd) = info.seldepth {
            print!(" seldepth {}", sd);
        }
        if let Some(t) = info.time {
            print!(" time {}", t.as_millis());
        }
        if let Some(n) = info.nodes {
            print!(" nodes {}", n);
        }
        if let Some(cp) = info.score_cp {
            print!(" score cp {}", cp);
        }
        if let Some(mate) = info.score_mate {
            print!(" score mate {}", mate);
        }
        if let Some(pv) = info.pv {
            print!(" pv {}", pv);
        }
        println!();
    }
}
