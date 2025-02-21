use std::net::{TcpListener, TcpStream};
use tungstenite::accept;

/// A WebSocket echo server
fn main() {
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    handle(stream);
                });
            }
            Err(error) => {
                eprintln!("Error listening socket: {error}");
            }
        }
    }
}

fn handle(stream: TcpStream) {
    let mut websocket = accept(stream).unwrap();
    loop {
        let msg = websocket.read().unwrap();

        // We do not want to send back ping/pong messages.
        if msg.is_binary() || msg.is_text() {
            websocket.send(msg).unwrap();
        }
    }
}
