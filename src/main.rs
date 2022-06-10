#![deny(warnings)]

use std::collections::HashMap;

use tokio::sync::RwLock;
use lazy_static::lazy_static;
use warp::Filter;
use futures::StreamExt;
use futures::FutureExt;

type GameID = u64;

lazy_static! {
    static ref STATE: RwLock<State> = RwLock::new(State::init());
}

struct State {
    games: HashMap<GameID, Game>,
}

enum Game {
    Gathering(GameGathering),
    Running(GameRunning),
}

struct GameGathering {}
struct GameRunning {}

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

    let route1 = warp::path("ws")
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
    let route2 = warp::path!("sum" / u32 / u32)
        .map(|a, b| {
            format!("{} + {} = {}", a, b, a + b)
        });

    warp::serve(route1.or(route2)).run(([127, 0, 0, 1], 3030)).await;
}
