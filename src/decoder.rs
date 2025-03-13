use std::collections::VecDeque;

use rodio::Source;
use symphonia::core::{
    audio::SampleBuffer,
    codecs::{CODEC_TYPE_NULL, CodecParameters, Decoder, DecoderOptions},
    errors::Error as SymphoniaError,
    formats::{FormatOptions, FormatReader},
    io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::{Hint, ProbeResult},
};

pub fn decode(source: Box<dyn MediaSource>) -> Result<DecodeSource, SymphoniaError> {
    let probe = symphonia::default::get_probe();

    let source_opts = MediaSourceStreamOptions::default();
    let stream = MediaSourceStream::new(source, source_opts);

    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    let ProbeResult {
        format,
        metadata: _,
    } = probe.format(&Hint::new(), stream, &format_opts, &metadata_opts)?;

    let tracks = format.tracks();

    let audio_track = tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or(SymphoniaError::Unsupported("No track found in format"))?;
    let track_id = audio_track.id;
    let codec_params = audio_track.codec_params.clone();
    DecodeSource::new(format, codec_params, track_id)
}

pub struct DecodeSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    buf: VecDeque<f32>,
    codec_params: CodecParameters,
    track_id: u32,
}

impl DecodeSource {
    fn new(
        format: Box<dyn FormatReader>,
        codec_params: CodecParameters,
        track_id: u32,
    ) -> Result<Self, SymphoniaError> {
        let codec = symphonia::default::get_codecs();

        let options = DecoderOptions { verify: true };
        let decoder = codec.make(&codec_params, &options)?;

        let buf = VecDeque::new();
        Ok(DecodeSource {
            format,
            decoder,
            buf,
            codec_params,
            track_id,
        })
    }
}

impl Iterator for DecodeSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buf.is_empty() {
            let packet = self.format.next_packet().ok()?;
            if packet.track_id() != self.track_id {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => decoded,
                Err(SymphoniaError::DecodeError(err)) => {
                    println!("DecodeError: {}", err);
                    continue;
                }
                Err(_) => {
                    break;
                }
            };
            let decoded = decoded.make_equivalent::<f32>();
            let spec = decoded.spec();
            let mut buffer = SampleBuffer::new(decoded.capacity() as u64, *spec);
            buffer.copy_interleaved_typed(&decoded);
            self.buf = buffer.samples().to_owned().into();
        }
        self.buf.pop_front()
    }
}

impl Source for DecodeSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.codec_params.channels.unwrap().count() as u16
    }

    fn sample_rate(&self) -> u32 {
        self.codec_params.sample_rate.unwrap()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
