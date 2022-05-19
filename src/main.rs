use crate::{
    grid::Grid,
    server::{handle_connection, State},
};
use clap::Parser;
use futures::SinkExt;
use lazy_static::lazy_static;
use std::{sync::Mutex, thread};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;

mod binary_io;
mod cellformat;
mod grid;
mod log;
mod messages;
mod server;
mod ui;

// type State = (
//     /*connected clients*/ Arc<Mutex<HashMap<SocketAddr, UnboundedSender<Message>>>>,
//     /*log sender*/ Sender<String>,
// );

const GRID_WIDTH: u16 = 100;
const GRID_HEIGHT: u16 = 100;
lazy_static! {
    static ref GRID: Mutex<Grid> = Mutex::new(Grid::new(GRID_WIDTH, GRID_HEIGHT));
}

macro_rules! log {
    [$to:ident: $format:literal] => {
        log!($to: $format,);
    };
    [$to:ident: $format:literal, $($arg:tt)*] => {
        let _ = $to.send(format!($format, $($arg)*)).await.unwrap();
    };
}

#[derive(Parser, Debug, Clone)]
#[clap(version, about)]
struct Args {
    #[clap(short, long, default_value_t = 3001)]
    port: u16,

    #[clap(short, long, default_value = "127.0.0.1")]
    ip: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    drop(GRID.lock().unwrap());
    let args = Args::parse();

    let addr = format!("{}:{}", args.ip, args.port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Error listening on socket");

    // io stuff
    let (log, commands) = ui::create_ui();

    log!(log: "\x1b[33m[SERVER] Listening on \x1b[1m{}\x1b[0;33m.\x1b[0m", addr);
    let state = State::new(log);

    // command handling
    let state1 = state.clone();
    tokio::spawn(async move {
        while let Ok(command) = commands.recv().await {
            execute_command(&state1, &command).await;
        }
    });

    // accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, state.clone()));
    }
}

async fn execute_command(state: &State, command: &str) {
    let mut parts = command.split_whitespace().collect::<Vec<_>>();
    let command = parts.remove(0)[1..].to_lowercase();
    let command = command.as_str();
    match command {
        "kick" => {
            // let id = parts[0];
            // let lock = state.clients.lock().unwrap();
            // let client = lock.iter_mut().find(|(_, c)| c.0 == id);
            // match client {
            //     Some((_, c)) => {
            //         c.1.send(Message::Close(None)).await.unwrap();
            //     }
            //     None => {
            //         let log = &state.log;
            //         log!(log: "\x1b[31m[COMMAND] No client with id \x1b[1m{}\x1b[0;31m.\x1b[0m", id);
            //     }
            // }
        }
        _ => {}
    }
}
