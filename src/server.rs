use std::{net::SocketAddr, sync::{Arc, Mutex}, collections::HashMap};

use async_channel::Sender;
use futures::{future, StreamExt, pin_mut, SinkExt, TryStreamExt};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::{Message, Error, http::HeaderValue, handshake::server::{Request, Response}}, WebSocketStream, accept_hdr_async};

use crate::{binary_io::{OutputStream, InputStream}, messages::JMMessage, GRID};

macro_rules! log {
    [$to:ident: $format:literal] => {
        log!($to: $format,);
    };
    [$to:ident: $format:literal, $($arg:tt)*] => {
        let _ = $to.send(format!($format, $($arg)*)).await.unwrap();
    };
}

macro_rules! ok {
    // ($name:ident $(else $e:stmt)?) => {
    //     let $name = if let Some(d) = $name { d }
    //         else { $($e)? return; };
    //     $name
    // };
    ($name:ident $(else $e:stmt)?) => {
        let $name = if let Some(d) = $name { d }
            else { $($e)? return; };
    };
    ($data:expr => $name:ident $(else $e:stmt)?) => {
        let data = $data;
        let $name;
        if let Some(d) = data { $name = d; }
        else {
            $($e)?
            return;
        }
    }
}

macro_rules! send {
    ($sender:expr, $msg:expr) => {
        let _ = $sender.1.unbounded_send(Message::Binary({
            let mut stream = OutputStream::new();
            $msg.write_v1(&mut stream);
            stream.bytes
        }));
    };
}

macro_rules! respond {
    ($state:expr, $addr:expr, $msg:expr) => {
        let clients = $state.clients.lock().unwrap();
        if let Some(cl) = clients.get(&$addr.0) {
            send!(cl, $msg);
        }
    };
}

pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, state: State) {
    let (stream, ref version) = read_sec_header(stream).await;

    let log = state.log.clone();

    ok!(stream.ok() => stream else log!(log: "\x1b[32m[CLIENT] Connection from {} failed.\x1b[m", addr));
    ok!(version else log!(log: "\x1b[32m[CLIENT] Connection from {} failed: did not specify client version.\x1b[m", addr));

    let client_id = rand::random::<u64>();
    let client_id = format!("{:x}", client_id);
    log!(log: "\x1b[32m[CLIENT:{}] New connection from {} with version {}.\x1b[m", client_id, addr, version);

    let (tx, rx) = unbounded();
    state.clients.lock().unwrap().insert(addr, (client_id.clone(), tx));

    let (mut out, inp) = stream.split();

    let _ = out.send(Message::Binary({
        let mut stream = OutputStream::new();
        JMMessage::SetGrid(GRID.lock().unwrap().clone()).write_v1(&mut stream);
        stream.bytes
    })).await;

    let handle_input = inp.try_for_each(|msg| {
        if let Message::Ping(data) = msg {
            if let Some(cl) = state.clients.lock().unwrap().get(&addr) {
                let _ = cl.1.unbounded_send(Message::Pong(data));
            }
        }
        else if let Message::Binary(data) = msg {
            process_input(InputStream::new(data), version, (addr, client_id.clone()), state.clone());
        }

        future::ok(())
    });

    let fut_forward = rx.map(Ok).forward(out);

    pin_mut!(fut_forward, handle_input);
    future::select(fut_forward, handle_input).await;

    log!(log: "\x1b[31m[CLIENT:{}] {} disconnected\x1b[m", client_id, addr);
    state.clients.lock().unwrap().remove(&addr);
}

async fn read_sec_header(stream: TcpStream) -> (Result<WebSocketStream<TcpStream>, Error>, Option<String>) {
    let mut sec_websocket_protocol = None;
    let stream = accept_hdr_async(stream, |req: &Request, mut res: Response| {
		sec_websocket_protocol = req.headers().get("Sec-WebSocket-Protocol").and_then(|s| s.to_str().map(|v| v.to_string()).ok());
        if let Some(protocol) = &sec_websocket_protocol {
            res.headers_mut().insert("Sec-WebSocket-Protocol", HeaderValue::from_str(protocol).unwrap());
        }
        Ok(res)
	}).await;

    (stream, sec_websocket_protocol)
}

fn process_input(stream: InputStream, version: &str, addr: (SocketAddr, String), state: State) -> Option<()> {
    match version {
        "1" => process_v1(stream, addr, state),
        _ => { None },
    }
}

fn process_v1(mut stream: InputStream, client: (SocketAddr, String), state: State) -> Option<()> {
    let msg = JMMessage::parse_v1(&mut stream)?;

    match msg {
        JMMessage::GetGrid => {
            respond!(state, client, JMMessage::SetGrid(GRID.lock().unwrap().clone()));
        },
        JMMessage::SetGrid(_) => {},
        JMMessage::SetCell(x, y, cell_id, direction) => {
            let mut grid = GRID.lock().unwrap();
            if x >= grid.width || y >= grid.height {
                return None;
            }
            if cell_id.is_empty() {
                *grid.get(x, y) = None;
            }
            else {
                *grid.get(x, y) = Some((cell_id.clone(), direction));
            }
            drop(grid);

            let clients = state.clients.lock().unwrap();
            for (cur_addr, tx) in clients.iter() {
                if cur_addr == &client.0 { continue; }
                send!(tx, JMMessage::SetCell(x, y, cell_id.clone(), direction));
            }
        },
        _ => { return None; },
    }

    Some(())
}

type Client = (String, UnboundedSender<Message>);

#[derive(Clone)]
pub struct State {
    pub clients: Arc<Mutex<HashMap<SocketAddr, Client>>>,
    pub log: Sender<String>
}

impl State {
    pub fn new(log: Sender<String>) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            log
        }
    }
}
