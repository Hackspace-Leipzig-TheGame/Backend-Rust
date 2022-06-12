//! # Current communication modell
//!
//! ## Initial game creation
//! ```text
//! | Get /                |
//! | -------------------> |
//! |                      |
//! | Redirect /game/<u64> |
//! | <------------------- |
//! ```
//!
//! ## Lobby
//! ```text
//! | Get /game/<u64>      |
//! | -------------------> |
//! |                      |
//! |                      |
//! | <------------------- |
//! ```
use futures::{StreamExt, FutureExt};
use tokio::sync::RwLock;
use warp::{Filter, hyper::Uri};
use lazy_static::lazy_static;
use regex::Regex;
use rand::{random, prelude::Distribution, distributions::{Standard, self}};

use core::fmt;
use std::{str::FromStr, collections::HashMap, rc::Rc};

mod thegame;

use thegame::Game;

lazy_static! {
    static ref GAME_ID_RE: Regex = Regex::new(r"[A-F0-9]{16}").unwrap();
}

// TODO: Limit the amount of existing games
// TODO: Regularly clear dead games

/// Unique identifier for a game.
/// Basically a u256 in hex format, should be unguessable.
#[derive(Debug, Clone)]
pub struct GameID(String);

struct State {
    games: HashMap<GameID, Game>,
}

impl State {
    pub fn init() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let state = Rc::new(RwLock::new(State::init()));
    let state = warp::any().map(move || state.clone());

    let websocket = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx.forward(tx).map(|result| {
                    if let Err(e) = result {
                        eprintln!("websocket error: {:?}", e);
                    }
                })
            })
        });

    let game = warp::path!("game" / GameID)
        .map(|id| {
            format!("{}", id)
        });

    let root = warp::filters::path::end()
        .map(|| {
            let id = random::<GameID>();
            let uri: Uri = format!("/game/{}", id).parse().unwrap();
            warp::redirect::found(dbg!(uri))
        });

    warp::serve(websocket.or(game).or(root))
        .run(([127, 0, 0, 1], 3030))
        .await;
}

#[derive(Debug)]
pub struct NotAValidGameID;

impl FromStr for GameID {
    type Err = NotAValidGameID;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if GAME_ID_RE.is_match(raw) {
            Ok(GameID(raw.into()))
        } else {
            Err(NotAValidGameID)
        }
    }
}

impl Distribution<GameID> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> GameID {
        let raw_id: String = rng.gen::<[u64; 1]>()
            .into_iter()
            .map(|raw| format!("{:X}", raw))
            .collect();
        raw_id.parse().expect("Not a valid game id")
    }
}

impl fmt::Display for GameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
