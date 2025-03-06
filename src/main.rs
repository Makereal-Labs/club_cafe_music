use serde::Deserialize;
use serde_json::json;
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
use tungstenite::{Message, accept};

fn main() {
    let queue: Mutex<VecDeque<YoutubeInfo>> = Mutex::new(VecDeque::new());

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            loop {
                let info = queue.lock().unwrap().pop_front();
                if let Some(info) = info {
                    let format = info
                        .formats
                        .iter()
                        .filter(|m| m.acodec.clone().map(|s| s != "none").unwrap_or(false))
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
                    let ref_queue = &queue;
                    s.spawn(move || {
                        if let Err(error) = handle(stream, ref_queue) {
                            eprintln!("Error while handling socket: {error}");
                        }
                    });
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
        .arg("--flat-playlist")
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

fn handle(stream: TcpStream, queue: &Mutex<VecDeque<YoutubeInfo>>) -> anyhow::Result<()> {
    let mut websocket = accept(stream)?;

    websocket.send(Message::Text(
        serde_json::to_string(&json!({
            "msg": "queue",
            "queue": queue.lock().unwrap().iter()
                .map(|info| json!({"title": info.title})).collect::<Vec<_>>()
        }))?
        .into(),
    ))?;

    loop {
        let msg = match websocket.read() {
            Ok(msg) => msg,
            Err(tungstenite::Error::ConnectionClosed) => {
                return Ok(());
            }
            Err(err) => {
                return Err(err.into());
            }
        };

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
                let info = list.into_iter().next().unwrap();
                queue.lock().unwrap().push_back(info);
            }
        }
    }
}
