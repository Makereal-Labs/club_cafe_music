use reqwest::blocking::Client;
use std::cmp::min;
use std::io;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};

const BUFFER_SIZE: usize = 4 * 1024 * 1024;
const REQ_CHUNK_SIZE: usize = 64 * 1024;

pub struct HttpStream {
    len: usize,
    progress: usize,
    rx: Mutex<Receiver<io::Result<u8>>>,
}

impl HttpStream {
    pub fn new(client: Client, url: impl Into<String>) -> anyhow::Result<Self> {
        let url = url.into();
        let response = client.get(&url).send()?;
        let len = response
            .content_length()
            .ok_or(anyhow::anyhow!("Content-Length unknown"))? as usize;
        let (tx, rx) = sync_channel(BUFFER_SIZE);
        {
            let url = url.clone();
            std::thread::spawn(move || {
                let mut progress = 0;
                while progress < len {
                    let chunk_size = min(progress + REQ_CHUNK_SIZE, len);
                    println!("Range: {:8} {:8}", progress, chunk_size);
                    let response = client
                        .get(&url)
                        .header("Range", format!("bytes={}-{}", progress, chunk_size))
                        .send()
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, Box::new(err)));
                    let response = match response {
                        Ok(response) => response,
                        Err(error) => {
                            let _ = tx.send(Err(error));
                            return;
                        }
                    };
                    let bytes = response
                        .bytes()
                        .map_err(|err| io::Error::new(io::ErrorKind::Other, Box::new(err)));
                    let bytes = match bytes {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            let _ = tx.send(Err(error));
                            return;
                        }
                    };
                    progress += bytes.len();
                    for b in bytes {
                        if tx.send(Ok(b)).is_err() {
                            return;
                        }
                    }
                }
            });
        }
        let rx = Mutex::new(rx);
        Ok(HttpStream {
            len,
            progress: 0,
            rx,
        })
    }
}

impl io::Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.progress >= self.len {
            return Ok(0);
        }
        let chunk_size = min(self.progress + buf.len(), self.len) - self.progress;
        let rx = self.rx.lock().expect("rx failed to lock");
        for b in &mut buf[0..chunk_size] {
            *b = rx.recv().unwrap()?;
        }
        self.progress += chunk_size;
        Ok(chunk_size)
    }
}

impl io::Seek for HttpStream {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        use io::SeekFrom;
        let pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.len as i64 + offset,
            SeekFrom::Current(offset) => self.progress as i64 + offset,
        };
        self.progress = pos
            .try_into()
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, Box::new(err)))?;
        Ok(self.progress as u64)
    }
}

impl symphonia::core::io::MediaSource for HttpStream {
    fn is_seekable(&self) -> bool {
        false
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.len as u64)
    }
}
