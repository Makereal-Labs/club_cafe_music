mod decoder;
mod handler;
mod http_stream;
mod opus_decoder;
mod player;
mod yt_dlp;

use std::collections::VecDeque;
use std::io;
use std::net::TcpListener;
use std::sync::{Mutex, mpsc};

use handler::handle;
use player::player;
use yt_dlp::YoutubeInfo;

#[derive(Debug, Default)]
struct AppState {
    now_playing: Option<YoutubeInfo>,
    queue: VecDeque<YoutubeInfo>,
}

#[derive(Debug, Clone, Copy)]
struct Event;

fn main() {
    let state: Mutex<AppState> = Mutex::new(AppState::default());
    let event_listeners = Mutex::new(Vec::new());
    let (broadcast_tx, broadcast_rx) = mpsc::channel::<Event>();

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    server
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    std::thread::scope(|s| {
        let ref_broadcast_tx = broadcast_tx.clone();
        let ref_state = &state;
        s.spawn(move || {
            player(ref_state, ref_broadcast_tx);
        });
        let ref_state = &state;
        let ref_event_listeners = &event_listeners;
        s.spawn(move || {
            for stream in server.incoming() {
                match stream {
                    Ok(stream) => {
                        if let Ok(()) = stream.set_nonblocking(true) {
                            let (tx, rx) = mpsc::channel();
                            let _ = tx.send(Event);
                            ref_event_listeners.lock().unwrap().push(tx);
                            s.spawn(move || {
                                if let Err(error) = handle(stream, ref_state, rx) {
                                    eprintln!("Error while handling socket: {error}");
                                }
                            });
                        } else {
                            eprintln!("set_nonblocking failed");
                        }
                    }
                    Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    Err(error) => {
                        eprintln!("Error listening socket: {error}");
                    }
                }
            }
        });
        let ref_event_listeners = &event_listeners;
        s.spawn(move || {
            while let Ok(event) = broadcast_rx.recv() {
                ref_event_listeners
                    .lock()
                    .unwrap()
                    .retain(|listener| listener.send(event).is_ok());
            }
        });
    });
}
