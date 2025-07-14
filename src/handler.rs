use async_tungstenite::accept_async;
use async_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use log::{info, warn};
use serde_json::json;
use smol::{channel::Receiver, channel::Sender, future::try_zip, lock::Mutex, net::TcpStream};

use crate::song_queue::QueueEntry;
use crate::yt_dlp::{YoutubeInfo, get_ytdlp};
use crate::{AppState, BroadcastEvent, HandlerEvent};

pub async fn handle(
    stream: TcpStream,
    state: &Mutex<AppState<'_>>,
    event_recv: Receiver<BroadcastEvent>,
    handler_event_tx: Sender<HandlerEvent>,
) -> anyhow::Result<()> {
    let websocket = accept_async(stream).await?;

    let (writer, mut reader) = websocket.split();

    let writer = Mutex::new(writer);

    let send_snackbar = async |msg: &str| -> anyhow::Result<()> {
        let msg = serde_json::to_string(&json!({
            "msg": "snackbar",
            "text": msg,
        }))?;
        writer.lock().await.send(Message::Text(msg.into())).await?;
        Ok(())
    };

    let task1 = async {
        while let Ok(broadcast_event) = event_recv.recv().await {
            let msg = match broadcast_event {
                BroadcastEvent::UpdateQueue => {
                    let state = state.lock().await;
                    let info_to_json = |info: &YoutubeInfo| {
                        let url = format!("https://www.youtube.com/watch?v={}", info.id);
                        json!({"fetched": true, "title": info.title, "url": url, "time": info.duration})
                    };
                    let entry_to_json = |entry: &QueueEntry| match entry {
                        QueueEntry::Fetched(info) => info_to_json(info),
                        QueueEntry::Fetching(task) => {
                            json!({"fetched": false, "url": task.url()})
                        }
                        QueueEntry::Refetching(task) => {
                            json!({"fetched": false, "url": task.url(), "title": task.title()})
                        }
                    };
                    let now_playing = state.now_playing.as_ref().map(info_to_json);
                    let queue = state.queue.iter().map(entry_to_json).collect::<Vec<_>>();
                    json!({
                        "msg": "queue",
                        "now_playing": now_playing,
                        "queue": queue,
                    })
                }
                BroadcastEvent::UpdatePlayer => {
                    let state = state.lock().await;
                    json!({
                        "msg": "player",
                        "playing": state.player.playing,
                        "volume": state.player.volume,
                    })
                }
            };
            let msg = serde_json::to_string(&msg)?;
            writer.lock().await.send(Message::Text(msg.into())).await?;
        }
        Ok::<(), anyhow::Error>(())
    };

    let task2 = async {
        loop {
            let msg = match reader.next().await {
                Some(msg) => msg?,
                None => {
                    break;
                }
            };

            let msg = match msg {
                Message::Text(msg) => msg,
                Message::Close(_) => {
                    break;
                }
                _ => {
                    continue;
                }
            };

            use serde_json::Value::*;
            let obj = match serde_json::from_str(&msg) {
                Ok(Object(obj)) => obj,
                _ => {
                    continue;
                }
            };

            let msg = match obj.get("msg") {
                Some(String(msg)) => msg,
                _ => {
                    continue;
                }
            };

            match msg.as_str() {
                "yt" => {
                    send_snackbar("Request received! Please wait...").await?;
                    if let Some(String(link)) = obj.get("link") {
                        info!("Received link (url: {})", link);
                        let url = link.clone();
                        let future = get_ytdlp(link.clone());
                        state.lock().await.queue.push_task(future, url);
                        let _ = handler_event_tx.send(HandlerEvent::UpdateQueue).await;
                    }
                }
                "btn" => {
                    if let Some(String(action)) = obj.get("action") {
                        match action.as_str() {
                            "pause" => {
                                let _ = handler_event_tx.send(HandlerEvent::Pause).await;
                            }
                            "resume" => {
                                let _ = handler_event_tx.send(HandlerEvent::Resume).await;
                            }
                            "skip" => {
                                let _ = handler_event_tx.send(HandlerEvent::Skip).await;
                            }
                            _ => {
                                warn!("Unknown client message: msg = btn, action = {action}");
                            }
                        }
                    } else {
                        warn!("Malformed client message: msg = btn, action not found");
                    }
                }
                "volume" => match obj.get("volume") {
                    Some(Number(volume)) => {
                        if let Some(volume) = volume.as_f64() {
                            state.lock().await.player.volume = volume as f32;
                            let _ = handler_event_tx.send(HandlerEvent::SetVolume).await;
                        } else {
                            warn!(
                                "Malformed client message: msg = volume, value is not float ({volume})"
                            );
                        }
                    }
                    Some(_) => {
                        warn!("Malformed client message: msg = volume, value is not number");
                    }
                    None => {
                        warn!("Malformed client message: msg = volume, volume not found");
                    }
                },
                _ => {
                    warn!("Unknown client message: msg = {msg}");
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    try_zip(task1, task2).await?;

    Ok(())
}
