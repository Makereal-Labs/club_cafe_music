use symphonia::core::{
    audio::AudioBufferRef,
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

pub struct OpusDecoder {}

impl Decoder for OpusDecoder {
    fn try_new(_params: &CodecParameters, _options: &DecoderOptions) -> SymphoniaResult<Self>
    where
        Self: Sized,
    {
        Ok(OpusDecoder {})
    }

    fn supported_codecs() -> &'static [CodecDescriptor]
    where
        Self: Sized,
    {
        &[OPUS_CODEC_DESCRIPTOR]
    }

    fn reset(&mut self) {
        todo!()
    }

    fn codec_params(&self) -> &CodecParameters {
        todo!()
    }

    fn decode(&mut self, _packet: &SymphoniaPacket) -> SymphoniaResult<AudioBufferRef> {
        todo!()
    }

    fn finalize(&mut self) -> FinalizeResult {
        todo!()
    }

    fn last_decoded(&self) -> AudioBufferRef {
        todo!()
    }
}

fn convert_error(error: opus::Error) -> SymphoniaError {
    SymphoniaError::DecodeError(error.description())
}
