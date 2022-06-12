use crate::GameID;

pub enum Game {
    Gathering(GameGathering),
    Running(GameRunning),
}

struct GameGathering {
    id: GameID,
    // List of players? List of websockets of players?
}

struct GameRunning {
    id: GameID,
    // List of players? List of websockets of players?
}
