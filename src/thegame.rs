use crate::GameID;

#[derive(Debug)]
pub enum Game {
    Gathering(GameGathering),
    Running(GameRunning),
}

impl Game {
    pub fn new() -> Self {
        Game::Gathering(GameGathering::new())
    }

    pub fn begin(self) -> Self {
        match self {
            Game::Gathering(g) => Game::Running(g.into()),
            Game::Running(g) => Game::Running(g),
        }
    }
}

#[derive(Debug)]
pub struct GameGathering {
    id: GameID,
    // List of players? List of websockets of players?
}

impl GameGathering {
    pub fn new() -> Self {
        GameGathering {
            id: GameID::new_v4(),
        }
    }
}

#[derive(Debug)]
pub struct GameRunning {
    id: GameID,
    // List of players? List of websockets of players?
}

impl From<GameGathering> for GameRunning {
    fn from(g: GameGathering) -> Self {
        Self { id: g.id }
    }
}
