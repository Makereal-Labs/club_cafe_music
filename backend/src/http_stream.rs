use std::cmp::min;
use std::io;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};
use ureq::Agent;
use ureq::http::header::CONTENT_LENGTH;

const BUFFER_SIZE: usize = 4 * 1024 * 1024;
const REQ_CHUNK_SIZE: usize = 64 * 1024;

pub struct HttpStream {
    len: usize,
    progress: usize,
    rx: Mutex<Receiver<io::Result<u8>>>,
    buffer: Vec<u8>,
}

impl HttpStream {
    pub fn new(agent: Agent, url: impl Into<String>) -> anyhow::Result<Self> {
        let url = url.into();
        let response = agent.get(&url).call()?;
        let len = response
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or(anyhow::anyhow!("Content-Length unknown"))?
            .to_str()?
            .parse()?;
        let (tx, rx) = sync_channel(BUFFER_SIZE);
        {
            let url = url.clone();
            std::thread::spawn(move || {
                let mut progress = 0;
                while progress < len {
                    let chunk_end = min(progress + REQ_CHUNK_SIZE, len);
                    let response = agent
                        .get(&url)
                        .header("Range", format!("bytes={}-{}", progress, chunk_end - 1))
                        .call()
                        .map_err(|err| io::Error::other(Box::new(err)));
                    let mut response = match response {
                        Ok(response) => response,
                        Err(error) => {
                            let _ = tx.send(Err(error));
                            return;
                        }
                    };
                    let bytes = response
                        .body_mut()
                        .read_to_vec()
                        .map_err(|err| io::Error::other(Box::new(err)));
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
        let mut buffer = Vec::new();
        buffer.reserve_exact(len);
        Ok(HttpStream {
            len,
            progress: 0,
            rx,
            buffer,
        })
    }
}

impl io::Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.progress >= self.len {
            return Ok(0);
        }
        let read_end = min(self.progress + buf.len(), self.len);
        let read_size = read_end - self.progress;
        if self.buffer.len() < read_end {
            let rx = self.rx.lock().expect("rx failed to lock");
            while self.buffer.len() < read_end {
                self.buffer.push(rx.recv().unwrap()?);
            }
        }
        buf[..read_size].copy_from_slice(&self.buffer[self.progress..read_end]);
        self.progress += read_size;
        Ok(read_size)
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
