#![feature(async_closure)]
use std::sync::Mutex;
use tokio::net::TcpListener;
use lazy_static::lazy_static;
use clap::Parser;
use crate::{grid::Grid, server::{handle_connection, State}, chat::handle_message};

mod binary_io;
mod messages;
mod cellformat;
mod grid;
mod ui;
mod log;
mod server;
mod chat;

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
    let listener = TcpListener::bind(&addr).await.expect("Error listening on socket");

    // io stuff
    let (log, messages) = ui::create_ui();

    log!(log: "\x1b[33m[SERVER] Listening on \x1b[1m{}\x1b[0;33m.\x1b[0m", addr);
    let state = State::new(log);

    // chat messages
    let state1 = state.clone();
    tokio::spawn(async move {
        while let Ok(message) = messages.recv().await {
            handle_message(&state1, message).await;
        }
    });

    // accept connections
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, state.clone()));
    }
}
