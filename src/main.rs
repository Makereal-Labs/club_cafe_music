use serde::Deserialize;
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tungstenite::{accept, Message};

fn main() {
    let mut queue: VecDeque<String> = VecDeque::new();

    let url = "https://www.youtube.com/watch?v=ertwyT4gnc0";

    let list = get_ytdlp(url).unwrap();

    //queue.push_back("/home/makereal/forever.mp3".to_string());
    let info = list.into_iter().next().unwrap();

    let format = info
        .formats
        .iter()
        .filter(|m| m.acodec.clone().map(|s| s != "none").unwrap_or(false))
        .reduce(|acc, e| std::cmp::max_by_key(acc, e, |v| v.quality.unwrap_or(-10.0) as i32))
        .unwrap();
    println!("{format:?}");
    let url = format.url.to_owned();
    queue.push_back(url);

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            loop {
                if let Some(url) = queue.pop_front() {
                    if let Err(err) = vlc(&url) {
                        eprintln!("{}", err);
                    }
                }
                sleep(Duration::from_millis(200));
            }
        });
        for stream in server.incoming() {
            match stream {
                Ok(stream) => {
                    s.spawn(move || {
                        if let Err(error) = handle(stream) {
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

fn handle(stream: TcpStream) -> anyhow::Result<()> {
    let mut websocket = accept(stream)?;
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
        if let Message::Text(msg) = msg {
            if let Ok(Object(obj)) = serde_json::from_str(&msg) {
                if let Some(msg) = obj.get("msg") {
                    if msg == "yt" {
                        if let Some(String(link)) = obj.get("link") {
                            websocket.send(Message::text(link))?;
                        }
                    }
                }
            }
        }
    }
}
