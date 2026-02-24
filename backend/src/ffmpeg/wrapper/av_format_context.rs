use std::{
    mem::forget,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use ffmpeg_next::{
    error::ENOMEM,
    ffi::{AVERROR, AVFormatContext, avformat_alloc_context, avformat_free_context},
};
use libc::c_int;

pub struct AVFormatContextOwned(NonNull<AVFormatContext>);

impl AVFormatContextOwned {
    pub fn new() -> Result<Self, c_int> {
        match NonNull::new(unsafe { avformat_alloc_context() }) {
            Some(format_context) => Ok(AVFormatContextOwned(format_context)),
            None => Err(AVERROR(ENOMEM)),
        }
    }

    pub fn into_ptr(self) -> *mut AVFormatContext {
        let ptr = self.0.as_ptr();
        forget(self);
        ptr
    }
}

impl Drop for AVFormatContextOwned {
    fn drop(&mut self) {
        unsafe { avformat_free_context(self.0.as_ptr()) };
    }
}

impl Deref for AVFormatContextOwned {
    type Target = AVFormatContext;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for AVFormatContextOwned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut() }
    }
}

impl AsMut<AVFormatContext> for AVFormatContextOwned {
    fn as_mut(&mut self) -> &mut AVFormatContext {
        unsafe { self.0.as_mut() }
    }
}
