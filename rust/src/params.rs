use std::time::Duration;

#[derive(Debug, Default)]
pub struct GoParams {
    pub searchmoves: Option<i32>,
    pub ponder: Option<bool>,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<i32>,
    pub depth: Option<i32>,
    pub nodes: Option<i32>,
    pub mate: Option<i32>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

#[derive(Debug)]
pub struct PositionParams {
    pub is_fen: bool,
    pub position: String,
    pub moves: Vec<String>,
}

#[derive(Debug)]
pub struct SearchInfo<'a> {
    pub depth: Option<i32>,
    pub seldepth: Option<i32>,
    pub time: Option<Duration>,
    pub nodes: Option<u64>,
    pub multipv: Option<i32>,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub pv: Option<&'a str>,
}

#[derive(Debug)]
pub struct OptionParams {
    pub name: String,
    pub value: String,
}
