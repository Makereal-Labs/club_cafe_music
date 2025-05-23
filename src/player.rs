use std::mem::forget;
use std::time::Duration;

use log::error;
use reqwest::blocking::Client;
use rodio::Sink;
use smol::{Timer, channel::Sender, lock::Mutex};

use crate::{AppState, Event, decoder::decode, http_stream::HttpStream};

pub async fn player(state: &Mutex<AppState>, broadcast_tx: Sender<Event>) {
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    forget(stream);
    let sink = Sink::try_new(&stream_handle).unwrap();
    let client = Client::new();

    let mut queue_was_not_empty = true;
    loop {
        let info = {
            let mut state = state.lock().await;
            let info = state.queue.pop_front();
            state.now_playing = info.clone();
            info
        };

        if queue_was_not_empty || info.is_some() {
            let _ = broadcast_tx.send(Event).await;
        }

        queue_was_not_empty = info.is_some();

        if let Some(info) = info {
            let format = info
                .formats
                .iter()
                .filter(|m| m.acodec.clone().is_some_and(|s| s != "none"))
                .filter(|m| m.vcodec.clone().is_none_or(|s| s == "none"))
                .reduce(|acc, e| {
                    std::cmp::max_by_key(acc, e, |v| v.quality.unwrap_or(-10.0) as i32)
                });

            let format = match format {
                Some(format) => format,
                None => {
                    error!("No usable format when playing id: {}", info.id);
                    continue;
                }
            };

            let http_stream = match HttpStream::new(client.clone(), &format.url) {
                Ok(http_stream) => http_stream,
                Err(err) => {
                    error!("Fetch url failed: {}", err);
                    continue;
                }
            };

            let source = match decode(Box::new(http_stream)) {
                Ok(source) => source,
                Err(err) => {
                    error!("Audio decode failed: {}", err);
                    continue;
                }
            };

            sink.append(source);
            while !sink.empty() {
                Timer::after(Duration::from_millis(100)).await;
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
