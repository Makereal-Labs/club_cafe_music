use std::ptr::NonNull;

use ffmpeg_next::{
    error::ENOMEM,
    ffi::{AVERROR, av_free, av_malloc},
};
use libc::{c_int, c_void};

pub struct AVBufferOwned(NonNull<c_void>);

impl AVBufferOwned {
    pub fn new(buffer_size: usize) -> Result<Self, c_int> {
        match NonNull::new(unsafe { av_malloc(buffer_size) }) {
            Some(buffer) => Ok(AVBufferOwned(buffer)),
            None => Err(AVERROR(ENOMEM)),
        }
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

impl Drop for AVBufferOwned {
    fn drop(&mut self) {
        unsafe { av_free(self.0.as_ptr()) };
    }
}
