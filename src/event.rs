//! Implements RFC #2 v0.0.1
use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

lazy_static! {
    static ref PARTY_ID_RE: Regex = Regex::new("[a-zA-Z0-9]{4}").unwrap();
}

type Username = String;
type EventId = Uuid;
// TODO: JWT
type AuthToken = u8;

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct PartyId {
    raw: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Card {
    // TODO: Remove after commenting on RFC
    id: u8,
    value: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    id: u8,
    ascending: bool,
    current_value: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayAction {
    card_id: u8,
    stack_id: u8,
}

/// Encapsulates all possible events transmitted over the websocket.
#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    /// "Unique" id for this event
    pub id: EventId,
    /// The kind of event
    #[serde(flatten)]
    pub kind: EventKind,
}

/// All possible event kinds
#[derive(Debug, Deserialize, Serialize)]
#[non_exhaustive]
#[serde(tag = "type", content = "data")]
pub enum EventKind {
    /// Event to trigger the creation of a new party.
    #[serde(rename = "PARTY/CREATE_PARTY")]
    CreateParty { owner: Username },
    /// Server response after a party has successfully been created.
    ///
    /// **Note**: Only send to the user who send the `CreateParty` event.
    #[serde(rename = "PARTY/PARTY_CREATED")]
    PartyCreated {
        #[serde(rename = "resonseTo")]
        response_to: EventId,
        #[serde(rename = "partyId")]
        party_id: PartyId,
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
    },
    /// Register a new user for this party.
    #[serde(rename = "PARTY/JOIN")]
    JoinParty {
        username: Username,
        #[serde(rename = "partyId")]
        party_id: PartyId,
    },
    /// Used to rejoin a party after a lost connection using the authentication JWT.
    #[serde(rename = "PARTY/REJOIN")]
    RejoinParty {
        #[serde(rename = "partyId")]
        party_id: PartyId,
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
    },
    /// Server response when a party is successfully joined.
    ///
    /// **Note**: Only send to the user requesting a rejoin.
    #[serde(rename = "PARTY/PARTY_JOINED")]
    PartyJoined {
        #[serde(rename = "resonseTo")]
        response_to: EventId,
        #[serde(rename = "partyId")]
        party_id: PartyId,
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
    },
    /// Send by the owner of the game to start the game.
    #[serde(rename = "PARTY/START")]
    StartGame {
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
    },
    /// Gives the current game state to the users.
    ///
    /// Needs to be personalized since the hands of the players should not be accessable to other players.
    /// Gets sent on every step of the game.
    #[serde(rename = "GAME/STATE")]
    GameState {
        /// Username of the player whos turn it is.
        ///
        /// This is optional and may not exist, if the active player has not been decided on.
        /// See also: `PlayerStartVote`.
        #[serde(rename = "activePlayer")]
        active_player: Option<Username>,
        /// The current hand of the player who receives this event.
        #[serde(rename = "playerHand")]
        player_hand: Vec<Card>,
        /// Contains the number of cards each player has.
        ///
        /// This is represented in a map where the key is the username and the value is the amount of cards.
        #[serde(rename = "playersCardsCount")]
        players_cards_count: HashMap<Username, u8>,
        /// The card stacks in the game a player can deposit cards on.
        stacks: Vec<Stack>,
        /// Amount of cards left in the stack where players have to draw cards from.
        #[serde(rename = "drawStackCardCount")]
        draw_stack_card_count: u8,
    },
    /// Sent by a player to deposit cards on stacks.
    #[serde(rename = "GAME/PLAY")]
    PlayCards {
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
        /// Actions the user whishes to perform.
        actions: Vec<PlayAction>,
    },
    /// Send by a user to vote on which player should start.
    #[serde(rename = "GAME/VOTE")]
    PlayerStartVote {
        /// Username of the player that is voted for.
        nominee: Username,
    },
    /// Send by a user which, for whatever reason (reconnect etc ...), has lost the current game state.
    /// Triggers a resend of the `GameStateEvent` from the server.
    #[serde(rename = "GAME/REQUEST_STATE")]
    RequestState,
    /// Sent if a player is not able to play any other card and the game is lost.
    #[serde(rename = "GAME/PLAYER_LOST")]
    PlayersLost,
    /// Sent if the draw stack and every player hand is empty and the game is won.
    #[serde(rename = "GAME/PLAYER_WON")]
    PlayersWon,
    /// A user sends a chat message to the server.
    #[serde(rename = "CHAT/SEND_MESSAGE")]
    SendMessage {
        #[serde(rename = "authToken")]
        auth_token: AuthToken,
        /// Message the user would like to send.
        message: String,
    },
    /// Event send from the server to all users after someone sent a `SendMessage`.
    #[serde(rename = "CHAT/MESSAGE")]
    Message {
        /// Author of the message
        username: Username,
        /// Message the user sent.
        message: String,
    },
    /// Send if the party does not exist. Only gets sent to the author of the original event.
    #[serde(rename = "ERROR/PARTY_NOT_FOUND")]
    PartyNotFoundError {
        #[serde(rename = "resonseTo")]
        response_to: EventId,
        #[serde(rename = "partyId")]
        party_id: PartyId,
    },
    /// Send if the party does not exist. Only gets sent to the author of the original event.
    #[serde(rename = "ERROR/INVALID_PLAY")]
    InvalidPlayError {
        #[serde(rename = "resonseTo")]
        response_to: EventId,
        /// Textform Error message.
        reason: String,
    },
    /// Send if the party does not exist. Only gets sent to the author of the original event.
    #[serde(rename = "ERROR/AUTHENTICATION")]
    AuthError {
        #[serde(rename = "resonseTo")]
        response_to: EventId,
        /// Textform Error message.
        reason: String,
    },
}

#[derive(Debug)]
pub struct InvalidPartyId;
impl TryFrom<&'_ str> for PartyId {
    type Error = InvalidPartyId;

    fn try_from(raw: &'_ str) -> Result<Self, Self::Error> {
        if PARTY_ID_RE.is_match(raw) {
            Ok(PartyId {
                raw: raw.to_owned(),
            })
        } else {
            Err(InvalidPartyId)
        }
    }
}
