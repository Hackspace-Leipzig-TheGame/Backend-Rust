use futures::{FutureExt, StreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;
use warp::{header, http::Response, hyper::body::Bytes, reply::with_header, Filter, Reply};

use std::{collections::HashMap, sync::Arc};

mod event;
mod thegame;

use event::{Event, EventKind};
use thegame::Game;

type GameID = Uuid;
type State = Arc<RwLock<StateRaw>>;

#[derive(Debug)]
struct NotUtf8;
impl warp::reject::Reject for NotUtf8 {}

#[derive(Debug)]
struct StateRaw {
    games: HashMap<GameID, Game>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum WsMessage {
    State { hand: Vec<u8>, decks: [u8; 4] },
}

impl StateRaw {
    pub fn init() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            games: HashMap::new(),
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
            info!("Someone created a session");
            // TODO:
            Response::builder()
                .status(201)
                .header("x-session-id", Uuid::new_v4().to_string())
                .header("x-client-id", Uuid::new_v4().to_string())
                .body("")
        });

    let join_session = warp::path("join")
        .and(warp::get())
        .and(state.clone())
        .and(header::<Uuid>("x-session-id"))
        .map(|state: State, session_id: Uuid| {
            info!("Someone joined a session");
            // TODO:
            Response::builder()
                .status(200)
                .header("x-session-id", session_id.to_string())
                .header("x-client-id", Uuid::new_v4().to_string())
                .body("")
        });

    let send_message = warp::path("send")
        .and(warp::post())
        .and(state.clone())
        .and(header::<Uuid>("x-session-id"))
        .and(header::<Uuid>("x-client-id"))
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::bytes().and_then(|body: Bytes| async move {
            std::str::from_utf8(&body)
                .map(String::from)
                .map_err(|_| warp::reject::custom(NotUtf8))
        }))
        .map(
            |state: State, session_id: Uuid, client_id: Uuid, msg: String| {
                info!("Someone send {msg}");
                // TODO: All the things
                warp::reply::reply()
            },
        );

    let get_session_state = warp::path("state")
        .and(warp::get())
        .and(state.clone())
        .and(header::<Uuid>("x-session-id"))
        .map(|state: State, session_id: Uuid| {
            info!("Someone requested the state");
            // TODO: All the things
            let msg = WsMessage::State {
                hand: vec![12, 1, 65, 87, 43, 29],
                decks: [1, 1, 100, 100],
            };
            warp::reply::json(&msg)
        })
        .map(|reply| with_header(reply, "x-mvp-version", "0.1"));

    warp::serve(
        ws.or(new_session)
            .or(join_session)
            .or(send_message)
            .or(get_session_state),
    )
    .run(([127, 0, 0, 1], 3032))
    .await;
}
