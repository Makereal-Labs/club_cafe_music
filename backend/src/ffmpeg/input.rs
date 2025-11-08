use std::{
    cmp::min,
    ffi::{CString, c_int, c_void},
    ptr::{null, null_mut},
};

use ffmpeg_next::{
    error::ENOMEM,
    ffi::{
        AVERROR, AVERROR_EOF, AVFMT_FLAG_CUSTOM_IO, AVIOContext, AVProbeData, AVSEEK_SIZE,
        SEEK_CUR, SEEK_END, SEEK_SET, av_dump_format, av_free, av_malloc, av_probe_input_format,
        av_strerror, avformat_alloc_context, avformat_find_stream_info, avformat_open_input,
        avio_alloc_context, avio_context_free,
    },
    format::context::Input,
};

pub struct BufferInput {
    _data: Box<DataBuffer>,
    buffer: *mut u8,
    context: *mut AVIOContext,
}

unsafe impl Send for BufferInput {}

struct DataBuffer {
    cursor: i64,
    data: Box<[u8]>,
}

type OpaquePointer = *mut DataBuffer;

impl BufferInput {
    pub fn new(data: Box<[u8]>) -> Result<(Self, Input), i32> {
        let mut data = Box::new(DataBuffer { data, cursor: 0 });
        let buffer_size = 4096;
        let buffer = unsafe { av_malloc(buffer_size) } as *mut u8;
        if buffer.is_null() {
            return Err(AVERROR(ENOMEM));
        }
        let write_flag = 0;
        let opaque = data.as_mut() as OpaquePointer;
        let read_packet = Some(io_read as IoReadFn);
        let write_packet = None;
        let seek = Some(io_seek as IoSeekFn);
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
        let probe_data = AVProbeData {
            filename: filename.as_ptr(),
            buf: data.data.as_mut_ptr(),
            buf_size: 256,
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

        unsafe { av_dump_format(format_context, 0, url, 0) };

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

impl Drop for BufferInput {
    fn drop(&mut self) {
        unsafe { av_free(self.buffer as *mut c_void) };
        unsafe { avio_context_free(&mut self.context) };
    }
}

type IoReadFn = unsafe extern "C" fn(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int;

unsafe extern "C" fn io_read(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int {
    let data = unsafe { (opaque as OpaquePointer).as_mut() }.expect("Caught null ptr");
    let buffer = unsafe { std::slice::from_raw_parts_mut(buf, buf_size as usize) };
    if data.data.len() <= data.cursor as usize {
        return AVERROR_EOF;
    }
    let read_len = min(buf_size as usize, data.data.len() - data.cursor as usize);
    buffer[..read_len]
        .copy_from_slice(&data.data[data.cursor as usize..(data.cursor as usize + read_len)]);
    data.cursor += read_len as i64;
    read_len as c_int
}

type IoSeekFn = unsafe extern "C" fn(opaque: *mut c_void, offset: i64, whence: c_int) -> i64;

unsafe extern "C" fn io_seek(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
    let data = unsafe { (opaque as OpaquePointer).as_mut() }.expect("Caught null ptr");

    match whence {
        AVSEEK_SIZE => {
            return data.data.len() as i64;
        }
        SEEK_SET => {
            data.cursor = offset;
        }
        SEEK_CUR => {
            data.cursor += offset;
        }
        SEEK_END => {
            data.cursor = data.data.len() as i64 + offset;
        }
        _ => unreachable!(),
    }
    if data.cursor < 0 || data.cursor >= data.data.len() as i64 {
        return -1;
    }
    data.cursor
}

pub fn averror_to_string(error_code: c_int) -> String {
    let mut buf = [0u8; 128];
    unsafe { av_strerror(error_code, buf.as_mut_ptr() as *mut i8, 128) };
    String::from_utf8_lossy(&buf).to_string()
}
