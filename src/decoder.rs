use std::{collections::VecDeque, sync::LazyLock};

use rodio::Source;
use symphonia::{
    core::{
        audio::{AudioBuffer, Layout, SampleBuffer},
        codecs::{CODEC_TYPE_NULL, CodecParameters, CodecRegistry, Decoder, DecoderOptions},
        errors::Error as SymphoniaError,
        formats::{FormatOptions, FormatReader},
        io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions},
        meta::MetadataOptions,
        probe::{Hint, ProbeResult},
    },
    default::register_enabled_codecs,
};

use crate::opus_decoder::OPUS_CODEC_DESCRIPTOR;

static CODEC: LazyLock<CodecRegistry> = LazyLock::new(|| {
    let mut codec = CodecRegistry::new();
    register_enabled_codecs(&mut codec);
    codec.register(&OPUS_CODEC_DESCRIPTOR);
    codec
});

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
    let codec_descriptor = CODEC.get_codec(codec_params.codec).unwrap();
    println!(
        "Codec: {} ({})",
        codec_descriptor.short_name, codec_descriptor.long_name
    );
    DecodeSource::new(format, codec_params, track_id)
}

pub struct DecodeSource {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    buf: VecDeque<f32>,
    track_id: u32,
    audio_buf: AudioBuffer<f32>,
}

impl DecodeSource {
    fn new(
        format: Box<dyn FormatReader>,
        codec_params: CodecParameters,
        track_id: u32,
    ) -> Result<Self, SymphoniaError> {
        let options = DecoderOptions { verify: true };
        let decoder = CODEC.make(&codec_params, &options)?;

        let buf = VecDeque::new();
        let audio_buf = AudioBuffer::unused();
        Ok(DecodeSource {
            format,
            decoder,
            buf,
            track_id,
            audio_buf,
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
            if self.audio_buf.is_unused() {
                self.audio_buf = decoded.make_equivalent::<f32>();
            }
            decoded.convert(&mut self.audio_buf);
            let spec = self.audio_buf.spec();
            let mut buffer = SampleBuffer::new(decoded.capacity() as u64, *spec);
            buffer.copy_interleaved_typed(&self.audio_buf);
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
        let params = self.decoder.codec_params();
        params
            .channels
            .or(params.channel_layout.map(Layout::into_channels))
            .unwrap()
            .count() as u16
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.codec_params().sample_rate.unwrap()
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
