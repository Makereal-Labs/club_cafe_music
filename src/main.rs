use rodio::{Decoder, Sink, source::Source};
use std::collections::VecDeque;
use std::mem::forget;
use std::net::{TcpListener, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use tungstenite::accept;

/// A WebSocket echo server
fn main() {
    let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    forget(stream);
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut queue: VecDeque<Box<dyn Source<Item = f32> + Send>> = VecDeque::new();

    let file = std::io::BufReader::new(std::fs::File::open("/home/makereal/forever.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    queue.push_back(Box::new(source.convert_samples()));

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
