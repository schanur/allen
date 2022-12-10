use crate::{check_al_error, sys::*, AllenResult};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{ffi::c_void, mem::size_of};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Channels {
    Mono,
    Stereo,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BufferData<'a> {
    I8(&'a [i8]),
    I16(&'a [i16]),
}

impl BufferData<'_> {
    fn ptr(&self) -> *const c_void {
        match self {
            BufferData::I8(data) => data.as_ptr() as *const c_void,
            BufferData::I16(data) => data.as_ptr() as *const c_void,
        }
    }

    fn size(&self) -> usize {
        match self {
            BufferData::I8(data) => size_of::<i8>() * data.len(),
            BufferData::I16(data) => size_of::<i16>() * data.len(),
        }
    }
}

/// Buffer of audio data.
/// NOTE: Buffers are bound to a device.
/// To ensure safety, buffers are not allowed to be cloned. There can only be one instance per-handle.
pub struct Buffer {
    handle: u32,
}

impl Buffer {
    pub fn new() -> AllenResult<Self> {
        let mut handle = 0;
        unsafe { alGenBuffers(1, &mut handle) };

        check_al_error()?;

        Ok(Self { handle })
    }

    pub(crate) fn handle(&self) -> u32 {
        self.handle
    }

    pub fn data(&self, data: BufferData, channels: Channels, sample_rate: i32) -> AllenResult<()> {
        let format = match data {
            BufferData::I8(_) => match channels {
                Channels::Mono => AL_FORMAT_MONO8,
                Channels::Stereo => AL_FORMAT_MONO16,
            },
            BufferData::I16(_) => match channels {
                Channels::Mono => AL_FORMAT_MONO16,
                Channels::Stereo => AL_FORMAT_STEREO16,
            },
        };

        unsafe {
            alBufferData(
                self.handle,
                format,
                data.ptr(),
                data.size() as i32,
                sample_rate,
            )
        };

        check_al_error()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { alDeleteBuffers(1, &self.handle) }
        if let Err(err) = check_al_error() {
            println!("WARNING: Buffer drop failed! {}", err);
        }
    }
}
