use std::collections::VecDeque;

use ffmpeg_next::{Error, codec, format::context::Input, media};
use rodio::Source;

pub struct Decoder<T> {
    decoder: codec::decoder::Audio,
    input: Input,
    stream_index: usize,
    _extra: Option<T>,
}

impl<T> Decoder<T> {
    /// The `extra` field carries data that needs to outlive Input
    pub fn new(input: Input, extra: Option<T>) -> Result<Self, Error> {
        let stream = input
            .streams()
            .best(media::Type::Audio)
            .expect("could not find best audio stream");
        let stream_index = stream.index();
        let context = codec::context::Context::from_parameters(stream.parameters())?;
        let mut decoder = context.decoder().audio()?;
        decoder.set_parameters(stream.parameters())?;
        Ok(Decoder {
            decoder,
            input,
            stream_index,
            _extra: extra,
        })
    }

    pub fn decode(self) -> Result<DecodeSource<T>, Error> {
        Ok(DecodeSource {
            decoder: self.decoder,
            input: self.input,
            stream_index: self.stream_index,
            buf: VecDeque::new(),
            _extra: self._extra,
        })
    }
}

pub struct DecodeSource<T> {
    decoder: codec::decoder::Audio,
    input: Input,
    stream_index: usize,
    buf: VecDeque<f32>,
    _extra: Option<T>,
}

impl<T> Iterator for DecodeSource<T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // relying on the fact that `PacketIter<'_>` does not store internal state
        let mut packet_iter = self
            .input
            .packets()
            .filter(|(stream, _)| stream.index() == self.stream_index)
            .map(|(_, packet)| packet);

        while self.buf.is_empty() {
            let Some(packet) = packet_iter.next() else {
                let _ = self.decoder.send_eof();
                return None;
            };

            if let Err(error) = self.decoder.send_packet(&packet) {
                todo!("{error}")
            }

            let mut decoded = ffmpeg_next::util::frame::Audio::empty();
            while self.decoder.receive_frame(&mut decoded).is_ok() {
                let mut s = 0;
                loop {
                    let sample: Option<VecDeque<f32>> = (0..decoded.planes())
                        .map(|p| decoded.plane::<f32>(p).get(s).cloned())
                        .try_fold(VecDeque::new(), |mut acc, e| {
                            acc.push_back(e?);
                            Some(acc)
                        });
                    if let Some(mut sample) = sample {
                        self.buf.append(&mut sample);
                    } else {
                        break;
                    }
                    s += 1;
                }
            }
        }
        self.buf.pop_front()
    }
}

impl<T> Source for DecodeSource<T> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
