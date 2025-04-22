use serde_json::json;
use std::net::TcpStream;
use std::sync::{Mutex, mpsc};
use std::thread::yield_now;
use tungstenite::util::NonBlockingError;
use tungstenite::{Message, accept};

use crate::yt_dlp::{YoutubeInfo, get_ytdlp};
use crate::{AppState, Event};

pub fn handle(
    stream: TcpStream,
    state: &Mutex<AppState>,
    event_recv: mpsc::Receiver<Event>,
) -> anyhow::Result<()> {
    let mut websocket = accept(stream)?;

    loop {
        match event_recv.try_recv() {
            Ok(_event) => {
                let msg = {
                    let state = state.lock().unwrap();
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
                websocket.send(Message::Text(msg.into()))?;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                return Ok(());
            }
        }

        let msg = match websocket.read().map_err(|e| e.into_non_blocking()) {
            Ok(msg) => Some(msg),
            Err(None) => {
                // No message has been sent yet (no error occured)
                None
            }
            Err(Some(tungstenite::Error::ConnectionClosed)) => {
                return Ok(());
            }
            Err(Some(err)) => {
                return Err(err.into());
            }
        };

        if let Some(msg) = msg {
            use serde_json::Value::*;
            let msg = match msg {
                Message::Text(msg) => msg,
                _ => {
                    continue;
                }
            };

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

            if msg == "yt" {
                if let Some(String(link)) = obj.get("link") {
                    websocket.send(Message::text(link))?;

                    let list = get_ytdlp(link).unwrap();
                    let mut state = state.lock().unwrap();
                    for info in list {
                        state.queue.push_back(info);
                    }
                }
            }
        } else {
            yield_now();
        }
    }
}
