use futures::{StreamExt, FutureExt};
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{Filter};

use std::{sync::Arc, collections::HashMap};

mod thegame;

use thegame::Game;

type GameID = Uuid;
type State = Arc<RwLock<StateRaw>>;

#[derive(Debug)]
struct StateRaw {
    games: HashMap<GameID, Game>,
}

impl StateRaw {
    pub fn init() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            games: HashMap::new()
        }))
    }
}

// TODO: Limit the amount of existing games
// TODO: Regularly clear dead games

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let state = StateRaw::init();
    let state = warp::any().map(move || state.clone());

    let ws = warp::path("ws")
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

    let new_session = warp::path("new")
        .and(warp::get())
        .and(state.clone())
        .map(|state: State| {
            warp::reply::reply()
        });

    warp::serve(ws.or(new_session))
        .run(([127, 0, 0, 1], 3030))
        .await;
}
