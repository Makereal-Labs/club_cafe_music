mod decoder;
mod handler;
mod http_stream;
mod opus_decoder;
mod player;
mod yt_dlp;

use std::collections::VecDeque;

use smol::prelude::*;
use smol::{Executor, block_on, channel, lock::Mutex, net::TcpListener};

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
    let state: &_ = Box::leak(Box::new(Mutex::new(AppState::default())));
    let event_listeners = Mutex::new(Vec::new());
    let (broadcast_tx, broadcast_rx) = channel::unbounded::<Event>();

    let server = block_on(TcpListener::bind("0.0.0.0:9001")).unwrap();

    std::thread::scope(|s| {
        s.spawn(move || {
            block_on(player(state, broadcast_tx));
        });
        let ref_event_listeners = &event_listeners;
        s.spawn(|| {
            let ex = Executor::new();
            block_on(ex.run(async {
                let mut incoming = server.incoming();
                while let Some(stream) = incoming.next().await {
                    match stream {
                        Ok(stream) => {
                            let (tx, rx) = channel::unbounded();
                            let _ = tx.send(Event).await;
                            ref_event_listeners.lock().await.push(tx);
                            ex.spawn(async move {
                                if let Err(error) = handle(stream, state, rx).await {
                                    eprintln!("Error while handling socket: {error}");
                                }
                            })
                            .detach();
                        }
                        Err(error) => {
                            eprintln!("Error listening socket: {error}");
                        }
                    }
                }
            }));
        });
        let ref_event_listeners = &event_listeners;
        s.spawn(move || {
            block_on(async {
                while let Ok(event) = broadcast_rx.recv().await {
                    for listener in ref_event_listeners.lock().await.iter() {
                        let _ = listener.send(event).await;
                    }
                }
            });
        });
    });
}
