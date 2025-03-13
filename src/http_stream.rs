use reqwest::blocking::Client;
use std::cmp::min;
use std::io::{self, Write};

pub struct HttpStream {
    client: Client,
    url: String,
    len: usize,
    progress: usize,
}

impl HttpStream {
    pub fn new(client: Client, url: impl Into<String>) -> anyhow::Result<Self> {
        let url = url.into();
        let response = client.get(&url).send()?;
        let len = response.content_length().ok_or(anyhow::anyhow!("Content-Length unknown"))? as usize;
        Ok(HttpStream { client, url, len, progress: 0 })
    }
}

impl io::Read for HttpStream {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        if self.progress >= self.len {
            return Ok(0);
        }
        let section_end = min(self.progress + buf.len(), self.len);
        println!("Range: {:8} {:8}", self.progress, section_end);
        let response = self.client.get(&self.url)
            .header("Range", format!("bytes={}-{}", self.progress, section_end))
            .send()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, Box::new(err)))?;
        let bytes = response.bytes()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, Box::new(err)))?;
        let actual_len = buf.write(&bytes)?;
        self.progress += actual_len;
        Ok(actual_len)
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
        self.progress = pos.try_into()
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
