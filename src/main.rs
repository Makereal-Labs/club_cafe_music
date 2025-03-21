use reqwest::blocking::Client;
use rodio::{Sink, source::Source};
use std::collections::VecDeque;
use std::mem::forget;
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tungstenite::accept;

mod http_stream;
use http_stream::HttpStream;

mod decoder;
use decoder::decode;

mod opus_decoder;

fn main() {
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    forget(stream);
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut queue: VecDeque<Box<dyn Source<Item = f32> + Send>> = VecDeque::new();

    let client = Client::new();

    let url = "https://www.youtube.com/watch?v=ertwyT4gnc0";
    let list = get_ytdlp(url).unwrap();
    let link = list[0].clone();

    let http_stream = HttpStream::new(client.clone(), link).unwrap();

    let source = decode(Box::new(http_stream)).unwrap();
    //println!("{}", source.into_iter().collect::<Vec<_>>().len());
    sink.append(source);
    println!("debug1");
    sink.sleep_until_end();
    println!("debug2");
    //queue.push_back(Box::new(source.convert_samples()));

    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    std::thread::scope(|s| {
        s.spawn(|| {
            sink.sleep_until_end();
            sleep(Duration::from_millis(200));
            if let Some(source) = queue.pop_front() {
                sink.append(source);
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
