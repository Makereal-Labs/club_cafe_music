use std::collections::VecDeque;

use ffmpeg_next::{Error, codec, format::context::Input, media};
use rodio::Source;

pub struct Decoder {
    decoder: codec::decoder::Audio,
    input: Input,
    stream_index: usize,
}

impl Decoder {
    pub fn new(input: Input) -> Result<Self, Error> {
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
        })
    }

    pub fn decode(mut self) -> Result<DecodeSource, Error> {
        println!("rate: {}", self.decoder.rate());
        let mut buf: VecDeque<f32> = VecDeque::new();
        for (stream, packet) in self.input.packets() {
            if stream.index() == self.stream_index {
                self.decoder.send_packet(&packet)?;

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
                            buf.append(&mut sample);
                        } else {
                            break;
                        }
                        s += 1;
                    }
                }
            }
        }
        self.decoder.send_eof()?;
        println!("len: {}", buf.len());
        Ok(DecodeSource { decoder: self, buf })
    }
}

pub struct DecodeSource {
    decoder: Decoder,
    buf: VecDeque<f32>,
}

impl Iterator for DecodeSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.pop_front()
    }
}

impl Source for DecodeSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.decoder.decoder.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.decoder.rate()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
