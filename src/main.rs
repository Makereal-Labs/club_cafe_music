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
use systemd_journal_logger::{JournalLog, connected_to_journal};

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

#[derive(Debug, Clone, Copy)]
enum HandlerEvent {
    StateUpdate,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy)]
enum PlayerEvent {
    Pause,
    Resume,
}

fn main() {
    if connected_to_journal() {
        JournalLog::new().unwrap().install().unwrap();
    } else {
        TermLogger::init(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .unwrap();
    }
    log::set_max_level(LevelFilter::Info);

    let state = Mutex::new(AppState::default());
    let event_listeners = Mutex::new(Vec::new());
    let (broadcast_tx, broadcast_rx) = channel::unbounded::<Event>();
    let (handler_event_tx, handler_event_rx) = channel::unbounded::<HandlerEvent>();
    let (player_event_tx, player_event_rx) = channel::unbounded::<PlayerEvent>();

    let server = block_on(TcpListener::bind("0.0.0.0:9001")).unwrap();

    let ex = Executor::new();
    let task = player(&state, player_event_rx, broadcast_tx.clone());
    let task = zip(task, async {
        let mut incoming = server.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Ok(stream) => {
                    let (tx, rx) = channel::unbounded();
                    let _ = tx.send(Event).await;
                    event_listeners.lock().await.push(tx);
                    ex.spawn(async {
                        if let Err(error) =
                            handle(stream, &state, rx, handler_event_tx.clone()).await
                        {
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
    let task = zip(task, async {
        while let Ok(event) = handler_event_rx.recv().await {
            match event {
                HandlerEvent::StateUpdate => {
                    let _ = broadcast_tx.send(Event).await;
                }
                HandlerEvent::Pause => {
                    let _ = player_event_tx.send(PlayerEvent::Pause).await;
                }
                HandlerEvent::Resume => {
                    let _ = player_event_tx.send(PlayerEvent::Resume).await;
                }
            }
        }
    });
    block_on(ex.run(task));
}
