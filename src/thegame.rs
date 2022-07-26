use crate::GameID;

#[derive(Debug)]
pub enum Game {
    Gathering(GameGathering),
    Running(GameRunning),
}

#[derive(Debug)]
struct GameGathering {
    id: GameID,
    // List of players? List of websockets of players?
}

#[derive(Debug)]
struct GameRunning {
    id: GameID,
    // List of players? List of websockets of players?
}
