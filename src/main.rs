use serde::Deserialize;
use serde_json::json;
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::{Mutex, mpsc};
use std::thread::{sleep, yield_now};
use std::time::Duration;
use tungstenite::util::NonBlockingError;
use tungstenite::{Message, accept};

#[derive(Debug, Default)]
struct AppState {
    now_playing: Option<YoutubeInfo>,
    queue: VecDeque<YoutubeInfo>,
}

#[derive(Debug)]
struct Event;

fn main() {
    let state: Mutex<AppState> = Mutex::new(AppState::default());
    let mut event_listeners = Vec::new();

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            loop {
                let info = {
                    let mut state = state.lock().unwrap();
                    let info = state.queue.pop_front();
                    state.now_playing = info.clone();
                    info
                };
                if let Some(info) = info {
                    let format = info
                        .formats
                        .iter()
                        .filter(|m| m.acodec.clone().is_some_and(|s| s != "none"))
                        .reduce(|acc, e| {
                            std::cmp::max_by_key(acc, e, |v| v.quality.unwrap_or(-10.0) as i32)
                        });

                    let format = match format {
                        Some(format) => format,
                        None => {
                            eprintln!("No usable format when playing id: {}", info.id);
                            continue;
                        }
                    };

                    if let Err(err) = vlc(&format.url) {
                        eprintln!("{}", err);
                    }
                }
                sleep(Duration::from_millis(200));
            }
        });
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
                    if let Ok(()) = stream.set_nonblocking(true) {
                        let ref_state = &state;
                        let (tx, rx) = mpsc::channel();
                        let _ = tx.send(Event);
                        event_listeners.push(tx);
                        s.spawn(move || {
                            if let Err(error) = handle(stream, ref_state, rx) {
                                eprintln!("Error while handling socket: {error}");
                            }
                        });
                    } else {
                        eprintln!("set_nonblocking failed");
                    }
                }
                Err(error) => {
                    eprintln!("Error listening socket: {error}");
                }
            }
        }
    });
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
struct YoutubeInfo {
    id: String,
    title: String,
    description: Option<String>,
    channel: String,
    channel_url: String,
    duration: u32,
    playlist: Option<String>,
    thumbnail: String,
    formats: Vec<MediaFormat>,
}

#[allow(unused)]
#[derive(Debug, Clone, Deserialize)]
struct MediaFormat {
    format_note: Option<String>,
    quality: Option<f32>,
    vcodec: Option<String>,
    acodec: Option<String>,
    video_ext: String,
    audio_ext: String,
    ext: String,
    url: String,
}

fn get_ytdlp(url: &str) -> anyhow::Result<Vec<YoutubeInfo>> {
    if matches!(url.chars().next(), None | Some('-')) {
        return Err(anyhow::anyhow!("Invalid URL :{}", url));
    }

    let output = Command::new("yt-dlp")
        .arg("-j")
        .arg("--skip-download")
        .arg("--no-warning")
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Call to yt-dlp failed: {}", output.status));
    }

    let result = std::str::from_utf8(&output.stdout)?;

    let list = result
        .lines()
        .map(serde_json::from_str::<YoutubeInfo>)
        .collect::<serde_json::Result<_>>()?;

    Ok(list)
}

fn vlc(url: &str) -> anyhow::Result<()> {
    let output = Command::new("cvlc")
        .arg("-A")
        .arg("alsa,none")
        .arg("--no-video")
        .arg(url)
        .arg("vlc://quit")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let result = std::str::from_utf8(&output.stderr)?;
        return Err(anyhow::anyhow!(
            "Call to vlc failed: {}\n{}",
            output.status,
            result
        ));
    }
    Ok(())
}

fn handle(
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
                    let now_playing = state
                        .now_playing
                        .as_ref()
                        .map(|info| json!({"title": info.title}));
                    let queue = state
                        .queue
                        .iter()
                        .map(|info| json!({"title": info.title}))
                        .collect::<Vec<_>>();
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
