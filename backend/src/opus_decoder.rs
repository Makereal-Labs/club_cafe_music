use std::sync::Mutex;

use symphonia::core::{
    audio::{AsAudioBufferRef, AudioBuffer, AudioBufferRef, Layout, Signal, SignalSpec},
    codecs::{
        CODEC_TYPE_OPUS, CodecDescriptor, CodecParameters, Decoder, DecoderOptions, FinalizeResult,
    },
    errors::{Error as SymphoniaError, Result as SymphoniaResult},
    formats::Packet as SymphoniaPacket,
};

pub const OPUS_CODEC_DESCRIPTOR: CodecDescriptor = CodecDescriptor {
    codec: CODEC_TYPE_OPUS,
    short_name: "opus",
    long_name: "Opus",
    inst_func: |params, options| Ok(Box::new(OpusDecoder::try_new(params, options)?)),
};

pub struct OpusDecoder {
    params: CodecParameters,
    spec: SignalSpec,
    channels: opus::Channels,
    decoder: Mutex<opus::Decoder>,
    buf: AudioBuffer<f32>,
}

impl Decoder for OpusDecoder {
    fn try_new(params: &CodecParameters, _options: &DecoderOptions) -> SymphoniaResult<Self>
    where
        Self: Sized,
    {
        let params = params.clone();
        let sample_rate = params
            .sample_rate
            .ok_or(SymphoniaError::Unsupported("Sample rate unknown"))?;
        let channels = params
            .channel_layout
            .ok_or(SymphoniaError::Unsupported("Channel layout unknown"))?;
        let spec = SignalSpec::new(sample_rate, channels.into_channels());
        let channels = match channels {
            Layout::Mono => opus::Channels::Mono,
            Layout::Stereo => opus::Channels::Stereo,
            _ => {
                return Err(SymphoniaError::Unsupported("Unsupported channel layout"));
            }
        };
        let decoder = opus::Decoder::new(sample_rate, channels).map_err(convert_error)?;
        let decoder = Mutex::new(decoder);
        let buf = AudioBuffer::unused();
        Ok(OpusDecoder {
            params,
            spec,
            channels,
            decoder,
            buf,
        })
    }

    fn supported_codecs() -> &'static [CodecDescriptor]
    where
        Self: Sized,
    {
        &[OPUS_CODEC_DESCRIPTOR]
    }

    fn reset(&mut self) {
        self.decoder
            .lock()
            .expect("Failed to lock decoder")
            .reset_state()
            .expect("Failed to reset decoder");
    }

    fn codec_params(&self) -> &CodecParameters {
        &self.params
    }

    fn decode(&mut self, packet: &SymphoniaPacket) -> SymphoniaResult<AudioBufferRef> {
        let mut decoder = self.decoder.lock().expect("Failed to lock decoder");
        let input_buf = packet.buf();
        let num_channels = match self.channels {
            opus::Channels::Mono => 1,
            opus::Channels::Stereo => 2,
        };
        let mut output_buf = vec![0.0; 5760 * num_channels].into_boxed_slice();
        let n_samples = decoder
            .decode_float(input_buf, &mut output_buf, false)
            .map_err(convert_error)?;
        self.buf = AudioBuffer::new(n_samples as u64, self.spec);
        self.buf.render(Some(n_samples), |planes, idx| {
            for ch in 0..num_channels {
                planes.planes()[ch][idx] = output_buf[idx * num_channels + ch];
            }
            Ok(())
        })?;
        Ok(self.buf.as_audio_buffer_ref())
    }

    fn finalize(&mut self) -> FinalizeResult {
        FinalizeResult { verify_ok: None }
    }

    fn last_decoded(&self) -> AudioBufferRef {
        self.buf.as_audio_buffer_ref()
    }
}

fn convert_error(error: opus::Error) -> SymphoniaError {
    SymphoniaError::DecodeError(error.description())
}
