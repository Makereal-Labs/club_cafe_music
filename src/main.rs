use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tungstenite::accept;

fn main() {
    let mut queue: VecDeque<String> = VecDeque::new();

    let url = "https://www.youtube.com/watch?v=ertwyT4gnc0";

    let list = get_ytdlp(url).unwrap();

    queue.push_back("/home/makereal/forever.mp3".to_string());
    queue.push_back(list.into_iter().next().unwrap());

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

fn get_ytdlp(url: &str) -> anyhow::Result<Vec<String>> {
    if matches!(url.chars().next(), None | Some('-')) {
        return Err(anyhow::anyhow!("Invalid URL :{}", url));
    }

    let output = Command::new("yt-dlp")
        .arg("--get-url")
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
        .filter(|url| url.contains("mime=audio"))
        .map(String::from)
        .collect();

    Ok(list)
}

fn vlc(url: &str) -> anyhow::Result<()> {
    let output = Command::new("cvlc")
        .arg("-A")
        .arg("alsa,none")
        .arg(url)
        .arg("vlc://quit")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let result = std::str::from_utf8(&output.stderr)?;
        return Err(anyhow::anyhow!("Call to yt-dlp failed: {}\n{}", output.status, result));
    }
    Ok(())
}

fn handle(stream: TcpStream) -> anyhow::Result<()> {
    let mut websocket = accept(stream)?;
    loop {
        let msg = websocket.read()?;

        // We do not want to send back ping/pong messages.
        if msg.is_binary() || msg.is_text() {
            websocket.send(msg)?;
        }
    }
}
