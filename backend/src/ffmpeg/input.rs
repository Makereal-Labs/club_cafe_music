use std::{
    ffi::{CString, c_int, c_void},
    io::{BufRead, Seek, SeekFrom},
    ptr::{null, null_mut},
};

use ffmpeg_next::{
    error::ENOMEM,
    ffi::{
        AVERROR, AVERROR_EOF, AVFMT_FLAG_CUSTOM_IO, AVIOContext, AVProbeData, AVSEEK_SIZE,
        SEEK_CUR, SEEK_END, SEEK_SET, av_free, av_malloc, av_probe_input_format, av_strerror,
        avformat_alloc_context, avformat_find_stream_info, avformat_open_input, avio_alloc_context,
        avio_context_free,
    },
    format::context::Input,
};

pub struct BufferInput<T: BufRead + Seek> {
    _data: Box<DataBuffer<T>>,
    buffer: *mut u8,
    context: *mut AVIOContext,
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
        let buffer = unsafe { av_malloc(buffer_size) } as *mut u8;
        if buffer.is_null() {
            return Err(AVERROR(ENOMEM));
        }
        let write_flag = 0;
        let opaque = data.as_mut() as OpaquePointer<T>;
        let read_packet = Some(io_read::<T> as IoReadFn);
        let write_packet = None;
        let seek = Some(io_seek::<T> as IoSeekFn);
        // Documentation: https://www.ffmpeg.org/doxygen/8.0/avio_8h.html#a50c588d3c44707784f3afde39e1c181c
        let context = unsafe {
            avio_alloc_context(
                buffer,
                buffer_size as i32,
                write_flag,
                opaque as *mut c_void,
                read_packet,
                write_packet,
                seek,
            )
        };
        if context.is_null() {
            unsafe { av_free(buffer as *mut c_void) };
            return Err(AVERROR(ENOMEM));
        }

        let mut format_context = unsafe { avformat_alloc_context() };
        if format_context.is_null() {
            return Err(AVERROR(ENOMEM));
        }
        let format_context_ref = unsafe { format_context.as_mut() }.unwrap();
        format_context_ref.pb = context;
        format_context_ref.flags |= AVFMT_FLAG_CUSTOM_IO;

        let filename = CString::from(c"");
        // take a 256 byte sample for probing
        let mut head_buf = [0u8; 256];
        let head_buf_len = data
            .data
            .read(&mut head_buf)
            .expect("Failed to read buffer");
        data.data.rewind().expect("Failed to seek buffer");
        let probe_data = AVProbeData {
            filename: filename.as_ptr(),
            buf: head_buf.as_mut_ptr(),
            buf_size: head_buf_len as i32,
            mime_type: null(),
        };
        let is_opened = 1;
        format_context_ref.iformat = unsafe { av_probe_input_format(&probe_data, is_opened) };

        let ps = &mut format_context;
        let url = CString::from(c"");
        let url = url.as_ptr();
        let fmt = null();
        let options = null_mut();
        let error_code = unsafe { avformat_open_input(ps, url, fmt, options) };
        if error_code != 0 {
            eprintln!("Could not open input: {}", averror_to_string(error_code));
            return Err(error_code);
        }

        let error_code = unsafe { avformat_find_stream_info(format_context, null_mut()) };
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
                buffer,
                context,
            },
            unsafe { Input::wrap(format_context) },
        ))
    }
}

impl<T: BufRead + Seek> Drop for BufferInput<T> {
    fn drop(&mut self) {
        unsafe { av_free(self.buffer as *mut c_void) };
        unsafe { avio_context_free(&mut self.context) };
    }
}

type IoReadFn = unsafe extern "C" fn(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int;

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
            todo!("{error}")
        }
    };

    if len == 0 { AVERROR_EOF } else { len as c_int }
}

type IoSeekFn = unsafe extern "C" fn(opaque: *mut c_void, offset: i64, whence: c_int) -> i64;

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
    unsafe { av_strerror(error_code, buf.as_mut_ptr() as *mut i8, 128) };
    String::from_utf8_lossy(&buf).to_string()
}
