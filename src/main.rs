mod decoder;
mod handler;
mod http_stream;
mod opus_decoder;
mod player;
mod yt_dlp;

use std::collections::VecDeque;

use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};
use smol::prelude::*;
use smol::{Executor, block_on, channel, future::zip, lock::Mutex, net::TcpListener};

use log::{LevelFilter, error};
use systemd_journal_logger::{connected_to_journal, JournalLog};

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
    if connected_to_journal() {
        JournalLog::new().unwrap().install().unwrap();
    } else {
        TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Mixed, ColorChoice::Auto).unwrap();
    }
    log::set_max_level(LevelFilter::Info);

    let state = Mutex::new(AppState::default());
    let event_listeners = Mutex::new(Vec::new());
    let (broadcast_tx, broadcast_rx) = channel::unbounded::<Event>();

    let server = block_on(TcpListener::bind("0.0.0.0:9001")).unwrap();

    let ex = Executor::new();
    let task = player(&state, broadcast_tx.clone());
    let task = zip(task, async {
        let mut incoming = server.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Ok(stream) => {
                    let (tx, rx) = channel::unbounded();
                    let _ = tx.send(Event).await;
                    event_listeners.lock().await.push(tx);
                    ex.spawn(async {
                        if let Err(error) = handle(stream, &state, rx, broadcast_tx.clone()).await {
                            error!("Error while handling socket: {error}");
                        }
                    })
                    .detach();
                }
                Err(error) => {
                    error!("Error listening socket: {error}");
                }
            }
        }
    });
    let task = zip(task, async {
        while let Ok(event) = broadcast_rx.recv().await {
            for listener in event_listeners.lock().await.iter() {
                let _ = listener.send(event).await;
            }
        }
    });
    block_on(ex.run(task));
}
