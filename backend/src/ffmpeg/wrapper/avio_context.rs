use std::ptr::NonNull;

use ffmpeg_next::{
    error::ENOMEM,
    ffi::{AVERROR, AVIOContext, avio_alloc_context, avio_context_free},
};
use libc::{c_int, c_uchar, c_void};

pub type IoReadFn =
    unsafe extern "C" fn(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int;
pub type IoWriteFn =
    unsafe extern "C" fn(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int;
pub type IoSeekFn = unsafe extern "C" fn(opaque: *mut c_void, offset: i64, whence: c_int) -> i64;

pub struct AVIOContextInitParameter {
    pub buffer: *mut c_uchar,
    pub buffer_size: c_int,
    pub write_flag: c_int,
    pub opaque: *mut c_void,
    pub read_packet: Option<IoReadFn>,
    pub write_packet: Option<IoWriteFn>,
    pub seek: Option<IoSeekFn>,
}

pub struct AVIOContextOwned(NonNull<AVIOContext>);

impl AVIOContextOwned {
    pub fn new(param: AVIOContextInitParameter) -> Result<Self, c_int> {
        // Documentation: https://www.ffmpeg.org/doxygen/8.0/avio_8h.html#a50c588d3c44707784f3afde39e1c181c
        let context: *mut AVIOContext = unsafe {
            avio_alloc_context(
                param.buffer,
                param.buffer_size,
                param.write_flag,
                param.opaque,
                param.read_packet,
                param.write_packet,
                param.seek,
            )
        };

        match NonNull::new(context) {
            Some(context) => Ok(AVIOContextOwned(context)),
            None => Err(AVERROR(ENOMEM)),
        }
    }

    pub fn as_ptr(&self) -> *mut AVIOContext {
        self.0.as_ptr()
    }
}

impl Drop for AVIOContextOwned {
    fn drop(&mut self) {
        unsafe { avio_context_free(&mut self.0.as_ptr()) };
    }
}
