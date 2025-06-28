mod decoder;
mod handler;
mod http_stream;
mod opus_decoder;
mod player;
mod song_queue;
mod yt_dlp;

use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use smol::prelude::*;
use smol::{Executor, block_on, channel, future::zip, lock::Mutex, net::TcpListener};

use log::{LevelFilter, error};
use systemd_journal_logger::{JournalLog, connected_to_journal};

use handler::handle;
use player::player;
use song_queue::{SongQueue, process_queue};
use yt_dlp::YoutubeInfo;

#[derive(Debug, Default)]
struct AppState<'ex> {
    now_playing: Option<YoutubeInfo>,
    queue: SongQueue<'ex>,
    player: PlayerState,
}

#[derive(Debug)]
struct PlayerState {
    playing: bool,
    volume: f32,
}

#[derive(Debug, Clone, Copy)]
enum BroadcastEvent {
    UpdateQueue,
    UpdatePlayer,
}

#[derive(Debug, Clone, Copy)]
enum HandlerEvent {
    UpdateQueue,
    Pause,
    Resume,
    Skip,
    SetVolume,
}

#[derive(Debug, Clone, Copy)]
enum PlayerEvent {
    Pause,
    Resume,
    Skip,
    SetVolume,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
            playing: true,
            volume: 0.7,
        }
    }
}

fn main() {
    if connected_to_journal() {
        JournalLog::new().unwrap().install().unwrap();
    } else {
        let config = ConfigBuilder::new()
            .set_time_format_rfc2822()
            .set_time_offset_to_local()
            .unwrap()
            .build();
        TermLogger::init(
            LevelFilter::Trace,
            config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )
        .unwrap();
    }
    log::set_max_level(LevelFilter::Info);

    let state = Mutex::new(AppState::default());
    let event_listeners = Mutex::new(Vec::new());
    let (broadcast_tx, broadcast_rx) = channel::unbounded::<BroadcastEvent>();
    let (handler_event_tx, handler_event_rx) = channel::unbounded::<HandlerEvent>();
    let (player_event_tx, player_event_rx) = channel::unbounded::<PlayerEvent>();

    let server = block_on(TcpListener::bind("0.0.0.0:9001")).unwrap();

    let ex = Executor::new();
    let task = player(&state, player_event_rx, broadcast_tx.clone());
    let task = zip(task, process_queue(&state, handler_event_tx.clone()));
    let task = zip(task, async {
        let mut incoming = server.incoming();
        while let Some(stream) = incoming.next().await {
            match stream {
                Ok(stream) => {
                    let (tx, rx) = channel::unbounded();
                    let _ = tx.send(BroadcastEvent::UpdatePlayer).await;
                    let _ = tx.send(BroadcastEvent::UpdateQueue).await;
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
                HandlerEvent::UpdateQueue => {
                    let _ = broadcast_tx.send(BroadcastEvent::UpdateQueue).await;
                }
                HandlerEvent::Pause => {
                    state.lock().await.player.playing = false;
                    let _ = player_event_tx.send(PlayerEvent::Pause).await;
                    let _ = broadcast_tx.send(BroadcastEvent::UpdatePlayer).await;
                }
                HandlerEvent::Resume => {
                    state.lock().await.player.playing = true;
                    let _ = player_event_tx.send(PlayerEvent::Resume).await;
                    let _ = broadcast_tx.send(BroadcastEvent::UpdatePlayer).await;
                }
                HandlerEvent::Skip => {
                    let _ = player_event_tx.send(PlayerEvent::Skip).await;
                }
                HandlerEvent::SetVolume => {
                    let _ = player_event_tx.send(PlayerEvent::SetVolume).await;
                    let _ = broadcast_tx.send(BroadcastEvent::UpdatePlayer).await;
                }
            }
        }
    });
    block_on(ex.run(task));
}
