use futures::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::{server::State, log::format_chat};

macro_rules! log {
    [$to:ident: $format:literal] => {
        log!($to: $format,);
    };
    [$to:ident: $format:literal, $($arg:tt)*] => {
        let _ = $to.send(format!($format, $($arg)*)).await.unwrap();
    };
}

pub struct ChatMessage {
    pub content: String,
    pub sender: String,
}

pub async fn handle_message(state: &State, message: ChatMessage) {
    let log = &state.log;

    if message.content.starts_with('/') {
        let mut parts = message.content.split_whitespace().collect::<Vec<_>>();
        let command = parts.remove(0)[1..].to_lowercase();
        let command = command.as_str();
        match command {
            "kick" => {
                let id = parts[0];
                let sender = {
                    let mut lock = state.clients.lock().unwrap();
                    lock.iter_mut().find(|(_, c)| c.0 == id).map(|(_, c)| c.1.clone())
                };
                match sender {
                    Some(mut c) => {
                        c.send(Message::Close(None)).await.unwrap();
                    }
                    None => {
                        let log = &state.log;
                        log!(log: "\x1b[31m[COMMAND] No client with id \x1b[1m{}\x1b[0;31m.\x1b[0m", id);
                    }
                }
            }
            "tell" => {
                let id = parts[0];
                let msg = parts[1..].join(" ");
                let sender = {
                    let mut lock = state.clients.lock().unwrap();
                    lock.iter_mut().find(|(_, c)| c.0 == id).map(|(_, c)| c.1.clone())
                };
                match sender {
                    Some(mut c) => {
                        // c.send(Message::Text(msg)).await.unwrap();
                    }
                    None => {
                        let log = &state.log;
                        log!(log: "\x1b[31m[COMMAND] No client with id \x1b[1m{}\x1b[0;31m.\x1b[0m", id);
                    }
                }
            }
            _ => {}
        }
    }
    else {
        log.send(format_chat(&message.sender, &message.content)).await.unwrap();
    }
}
