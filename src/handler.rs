use async_tungstenite::accept_async;
use async_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use smol::{channel::Receiver, future::try_zip, lock::Mutex, net::TcpStream};

use crate::yt_dlp::{YoutubeInfo, get_ytdlp};
use crate::{AppState, Event};

pub async fn handle(
    stream: TcpStream,
    state: &Mutex<AppState>,
    event_recv: Receiver<Event>,
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
        while let Ok(_event) = event_recv.recv().await {
            let msg = {
                let state = state.lock().await;
                let to_json = |info: &YoutubeInfo| {
                    let url = format!("https://www.youtube.com/watch?v={}", info.id);
                    json!({"title": info.title, "url": url, "time": info.duration})
                };
                let now_playing = state.now_playing.as_ref().map(to_json);
                let queue = state.queue.iter().map(to_json).collect::<Vec<_>>();
                serde_json::to_string(&json!({
                    "msg": "queue",
                    "now_playing": now_playing,
                    "queue": queue,
                }))?
            };
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

            send_snackbar("Request received! Please wait...").await?;

            if msg == "yt" {
                if let Some(String(link)) = obj.get("link") {
                    let list = get_ytdlp(link).unwrap();
                    let mut state = state.lock().await;
                    let snackbar_msg = if list.len() == 1 {
                        "Song added to queue!".to_string()
                    } else {
                        format!("Playlist (len = {}) added to queue!", list.len())
                    };
                    send_snackbar(&snackbar_msg).await?;
                    for info in list {
                        state.queue.push_back(info);
                    }
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    };

    try_zip(task1, task2).await?;

    Ok(())
}
