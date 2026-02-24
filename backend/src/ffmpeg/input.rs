use std::{
    io::{BufRead, Seek, SeekFrom},
    ptr::{null, null_mut},
};

use ffmpeg_next::{
    ffi::{
        AVDictionary, AVERROR_EOF, AVERROR_UNKNOWN, AVFMT_FLAG_CUSTOM_IO, AVFormatContext,
        AVInputFormat, AVProbeData, AVSEEK_SIZE, SEEK_CUR, SEEK_END, SEEK_SET,
        av_probe_input_format, av_strerror, avformat_find_stream_info, avformat_open_input,
    },
    format::context::Input,
};
use libc::{c_char, c_int, c_uchar, c_void};
use log::error;

use crate::ffmpeg::wrapper::{
    AVBufferOwned, AVFormatContextOwned, AVIOContextInitParameter, AVIOContextOwned, IoReadFn,
    IoSeekFn,
};

pub struct BufferInput<T: BufRead + Seek> {
    _data: Box<DataBuffer<T>>,
    _buffer: AVBufferOwned,
    _context: AVIOContextOwned,
}

unsafe impl<T: BufRead + Seek + Send> Send for BufferInput<T> {}

struct DataBuffer<T: BufRead + Seek> {
    data: T,
}

type OpaquePointer<T> = *mut DataBuffer<T>;

impl<T: BufRead + Seek> BufferInput<T> {
    pub fn new(data: T) -> Result<(Self, Input), i32> {
        let mut data = Box::new(DataBuffer { data });
        let buffer_size = 4096;
        let buffer = AVBufferOwned::new(buffer_size)?;
        let context = AVIOContextOwned::new(AVIOContextInitParameter {
            buffer: buffer.as_ptr() as *mut c_uchar,
            buffer_size: buffer_size as c_int,
            write_flag: 0,
            opaque: data.as_mut() as OpaquePointer<T> as *mut c_void,
            read_packet: Some(io_read::<T> as IoReadFn),
            write_packet: None,
            seek: Some(io_seek::<T> as IoSeekFn),
        })?;

        let mut format_context = AVFormatContextOwned::new()?;
        format_context.pb = context.as_ptr();
        format_context.flags |= AVFMT_FLAG_CUSTOM_IO;

        // take a 256 byte sample for probing
        let mut head_buf = [0u8; 256];
        let head_buf_len = data
            .data
            .read(&mut head_buf)
            .expect("Failed to read buffer");
        data.data.rewind().expect("Failed to seek buffer");
        let probe_data = AVProbeData {
            filename: c"".as_ptr(),
            buf: head_buf.as_mut_ptr(),
            buf_size: head_buf_len as i32,
            mime_type: null(),
        };
        let is_opened: c_int = 1;
        format_context.iformat = unsafe { av_probe_input_format(&probe_data, is_opened) };

        let ps: *mut *mut AVFormatContext = &mut (format_context.as_mut() as *mut AVFormatContext);
        let url: *const c_char = c"".as_ptr();
        let fmt: *const AVInputFormat = null();
        let options: *mut *mut AVDictionary = null_mut();
        let error_code: c_int = unsafe { avformat_open_input(ps, url, fmt, options) };
        if error_code != 0 {
            eprintln!("Could not open input: {}", averror_to_string(error_code));
            return Err(error_code);
        }

        let error_code: c_int =
            unsafe { avformat_find_stream_info(format_context.as_mut(), null_mut()) };
        if error_code != 0 {
            eprintln!(
                "Could not find stream information: {}",
                averror_to_string(error_code)
            );
            return Err(error_code);
        }

        Ok((
            BufferInput {
                _data: data,
                _buffer: buffer,
                _context: context,
            },
            unsafe { Input::wrap(format_context.into_ptr()) },
        ))
    }
}

unsafe extern "C" fn io_read<T: BufRead + Seek>(
    opaque: *mut c_void,
    buf: *mut u8,
    buf_size: c_int,
) -> c_int {
    let data = unsafe { (opaque as OpaquePointer<T>).as_mut() }.expect("Caught null ptr");
    let buffer = unsafe { std::slice::from_raw_parts_mut(buf, buf_size as usize) };

    let len = match data.data.read(buffer) {
        Ok(len) => len,
        Err(error) => {
            error!("{error}");
            return AVERROR_UNKNOWN;
        }
    };

    if len == 0 { AVERROR_EOF } else { len as c_int }
}

unsafe extern "C" fn io_seek<T: BufRead + Seek>(
    opaque: *mut c_void,
    offset: i64,
    whence: c_int,
) -> i64 {
    let data = unsafe { (opaque as OpaquePointer<T>).as_mut() }.expect("Caught null ptr");

    let seek_from = match whence {
        AVSEEK_SIZE => {
            return match get_stream_len(&mut data.data) {
                Ok(len) => len as i64,
                Err(_error) => {
                    return -1;
                }
            };
        }
        SEEK_SET => SeekFrom::Start(offset as u64),
        SEEK_CUR => SeekFrom::Current(offset),
        SEEK_END => SeekFrom::End(offset),
        _ => unreachable!(),
    };
    match data.data.seek(seek_from) {
        Ok(pos) => pos as i64,
        Err(_error) => -1,
    }
}

// code borrowed from nightly rust (`seek_stream_len`)
fn get_stream_len<T: Seek>(seek: &mut T) -> std::io::Result<u64> {
    let old_pos = seek.stream_position()?;
    let len = seek.seek(SeekFrom::End(0))?;

    if old_pos != len {
        seek.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

pub fn averror_to_string(error_code: c_int) -> String {
    let mut buf = [0u8; 128];
    unsafe { av_strerror(error_code, buf.as_mut_ptr() as *mut c_char, 128) };
    String::from_utf8_lossy(&buf).to_string()
}
