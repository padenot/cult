
#![allow(non_camel_case_types)]

extern crate libc;
#[macro_use]
extern crate bitflags;
// #[macro_use]
// #[cfg(feature = "plugins")]
// extern crate heapsize;

use libc::{c_int, c_void, c_long};
use std::ffi::{CStr, CString};
use std::mem::transmute;
use std::ptr;
use std::slice;
use std::result;
use std::marker::PhantomData;
use std::boxed::Box;

pub mod ffi;
use ffi::*;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Undefined           = -1,
    InvalidFormat       = -2,
    InvalidParameter    = -3,
    NotSupported        = -4,
    DeviceUnavailable   = -5,
}

impl From<c_int> for Error {
    fn from (code: c_int) -> Self {
        debug_assert!(code < 0);
        match code {
            CUBEB_ERROR_INVALID_FORMAT      => Error::InvalidFormat,
            CUBEB_ERROR_INVALID_PARAMETER   => Error::InvalidParameter,
            CUBEB_ERROR_NOT_SUPPORTED       => Error::NotSupported,
            CUBEB_ERROR_DEVICE_UNAVAILABLE  => Error::DeviceUnavailable,
            _                               => Error::Undefined,
        }
    }
}

pub type Result<T> = result::Result<T, Error>;


pub type DataCallback<T> = Box<FnMut(&[T], &mut [T]) -> usize>;
pub type StateCallback = Box<FnMut(State)>;
pub type DeviceChangedCallback = Box<FnMut()>;


#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum SampleFormat {
    Signed16LE  = 0,
    Signed16BE  = 1,
    Float32LE   = 2,
    Float32BE   = 3,
}

#[cfg(target_endian = "little")]
pub fn native_signed16() -> SampleFormat {
    SampleFormat::Signed16LE
}

#[cfg(target_endian = "big")]
pub fn native_signed16() -> SampleFormat {
    SampleFormat::Signed16BE
}

#[cfg(target_endian = "little")]
pub fn native_float32() -> SampleFormat {
    SampleFormat::Float32LE
}

#[cfg(target_endian = "big")]
pub fn native_float32() -> SampleFormat {
    SampleFormat::Float32BE
}

impl Into<cubeb_sample_format> for SampleFormat
{
    fn into(self) -> cubeb_sample_format {
        self as cubeb_sample_format
    }
}

pub trait Sample
{
    fn format() -> SampleFormat;
    fn data_cb_ffi() -> cubeb_data_callback;
}

impl Sample for i16 {
    fn format() -> SampleFormat {
        native_signed16()
    }
    fn data_cb_ffi() -> cubeb_data_callback {
        data_callback_i16
    }
}

impl Sample for f32 {
    fn format() -> SampleFormat {
        native_float32()
    }
    fn data_cb_ffi() -> cubeb_data_callback {
        data_callback_f32
    }
}


#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum LogLevel {
    Disabled    = 0,
    Normal      = 1,
    Verbose     = 2,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum ChannelLayout {
    Undefined       = 0,
    DualMono        = 1,
    DualMono_LFE    = 2,
    Mono            = 3,
    Mono_LFE        = 4,
    Stereo          = 5,
    Stereo_LFE      = 6,
    F3              = 7,
    F3_LFE          = 8,
    F2_1            = 9,
    F2_1_LFE        = 10,
    F3_1            = 11,
    F3_1_LFE        = 12,
    F2_2            = 13,
    F2_2_LFE        = 14,
    F3_2            = 15,
    F3_2_LFE        = 16,
    F3_R3_LFE       = 17,
    F3_4_LFE        = 18,
    Max             = 19,
}

impl Into<cubeb_channel_layout> for ChannelLayout {
    fn into(self) -> cubeb_channel_layout {
        self as cubeb_channel_layout
    }
}

impl From<cubeb_channel_layout> for ChannelLayout {
    fn from(layout: cubeb_channel_layout) -> ChannelLayout {
        unsafe { transmute( layout as u8 ) }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum State {
    Started = 0,
    Stopped = 1,
    Drained = 2,
    Error   = 3,
}

impl From<cubeb_state> for State {
    fn from(state: cubeb_state) -> State {
        unsafe { transmute( state as u8 ) }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DeviceType {
    Unknown = 0,
    Input   = 1,
    Output  = 2,
}

impl Into<cubeb_device_type> for DeviceType
{
    fn into(self) -> cubeb_device_type {
        self as cubeb_device_type
    }
}

impl From<cubeb_device_type> for DeviceType
{
    fn from(dt: cubeb_device_type) -> Self {
        unsafe { transmute( dt as u8 ) }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DeviceState {
    Disabled,
    Unplugged,
    Enabled,
}

impl Into<cubeb_device_state> for DeviceState
{
    fn into(self) -> cubeb_device_state {
        self as cubeb_device_state
    }
}

impl From<cubeb_device_state> for DeviceState
{
    fn from(dt: cubeb_device_state) -> Self {
        unsafe { transmute( dt as u8 ) }
    }
}

bitflags!{
    pub struct DeviceFmt : u16 {
        const DEVICE_FMT_S16LE  = 0x0010;
        const DEVICE_FMT_S16BE  = 0x0020;
        const DEVICE_FMT_F32LE  = 0x1000;
        const DEVICE_FMT_F32BE  = 0x2000;

        const DEVICE_FMT_S16_MASK = DEVICE_FMT_S16LE.bits |
                                    DEVICE_FMT_S16BE.bits;
        const DEVICE_FMT_F32_MASK = DEVICE_FMT_F32LE.bits |
                                    DEVICE_FMT_F32BE.bits;
        const DEVICE_FMT_ALL = DEVICE_FMT_S16_MASK.bits |
                               DEVICE_FMT_F32_MASK.bits;
    }
}

#[cfg(target_endian = "little")]
pub const DEVICE_FMT_S16NE: DeviceFmt = DEVICE_FMT_S16LE;
#[cfg(target_endian = "little")]
pub const DEVICE_FMT_F32NE: DeviceFmt = DEVICE_FMT_F32LE;
#[cfg(target_endian = "big")]
pub const DEVICE_FMT_S16NE: DeviceFmt = DEVICE_FMT_S16BE;
#[cfg(target_endian = "big")]
pub const DEVICE_FMT_F32NE: DeviceFmt = DEVICE_FMT_F32BE;


bitflags!{
    pub struct DevicePref : u8 {
        const DEVICE_PREF_NONE            = 0x00;
        const DEVICE_PREF_MULTIMEDIA      = 0x01;
        const DEVICE_PREF_VOICE           = 0x02;
        const DEVICE_PREF_NOTIFICATION    = 0x04;
        const DEVICE_PREF_ALL             = 0x0F;
    }
}


#[derive(Debug)]
pub struct Context {
    native: *const cubeb,
}

impl Context {
    pub fn new(context_name: &str, backend_name: Option<&str>) -> Result<Context> {
        let mut ctx = ptr::null();
        let context_name = CString::new(context_name).unwrap();
        let backend_name = backend_name.map(|s| CString::new(s).unwrap());
        let res = unsafe {
            cubeb_init(
                &mut ctx as *mut *const cubeb,
                context_name.as_ptr(),
                backend_name.map_or(ptr::null(), |s| s.as_ptr())
            )
        };
        match res {
            CUBEB_OK => {
                Ok( Context { native: ctx } )
            },
            _ => { Err( Error::from(res) ) }
        }
    }

    pub fn backend_id(&self) -> &str {
        let bid = unsafe {
            CStr::from_ptr(cubeb_get_backend_id(self.native))
        };
        bid.to_str().expect("Cubeb::backend_id is invalid UTF-8")
    }

    pub fn enumerate_devices(&self, devtype: DeviceType)
            -> Result<DeviceCollection> {
        let mut col = cubeb_device_collection {
            device: ptr::null(),
            count: 0,
        };
        let res = unsafe {
            cubeb_enumerate_devices(self.native, devtype.into(), &mut col)
        };
        match res {
            CUBEB_OK => {
                Ok(DeviceCollection {
                    native: col,
                    // ctx: Weak::upgrade(&self.weak_me.borrow()).unwrap(),
                    ctx: self,
                })
            }
            _ => { Err( Error::from(res) ) }
        }
    }

    pub fn max_channel_count(&self) -> Result<u32> {
        let mut count = 0;
        let res = unsafe {
            cubeb_get_max_channel_count(self.native, &mut count as *mut u32)
        };
        match res {
            CUBEB_OK => { Ok(count) },
            _ => { Err( Error::from(res) ) }
        }
    }

    pub fn min_latency<T: Sample>(&self, params: StreamParams<T>) -> Result<u32> {
        let mut latency = 0;
        let params = params.into();
        let res = unsafe {
            cubeb_get_min_latency(self.native,
                        &params as *const _,
                        &mut latency as *mut _)
        };
        match res {
            CUBEB_OK => { Ok(latency) },
            _ => { Err( Error::from(res) ) }
        }
    }

    pub fn preferred_sample_rate(&self) -> Result<u32> {
        let mut rate = 0;
        let res = unsafe {
            cubeb_get_preferred_sample_rate(self.native, &mut rate)
        };
        match res {
            CUBEB_OK => { Ok(rate) },
            _ => { Err( Error::from(res) ) }
        }
    }

    pub fn preferred_channel_layout(&self) -> Result<ChannelLayout> {
        let mut layout = 0;
        let res = unsafe {
            cubeb_get_preferred_channel_layout(self.native, &mut layout)
        };
        match res {
            CUBEB_OK => { Ok(ChannelLayout::from(layout)) },
            _ => { Err( Error::from(res) ) }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { cubeb_destroy(self.native); }
    }
}


pub enum DevId {}


#[derive(Debug)]
pub struct DeviceInfo<'a> {
    native: *const cubeb_device_info,
    phantom: PhantomData<&'a cubeb_device_info>,
}

impl<'a> DeviceInfo<'a>
{
    pub fn devid(&self) -> &'a DevId {
        unsafe { transmute( (*self.native).devid ) }
    }

    pub fn device_id(&self) -> &str {
        let slice = unsafe {
            CStr::from_ptr( (*self.native).device_id )
        };
        slice.to_str().expect("UTF-8 error")
    }

    pub fn friendly_name(&self) -> &str {
        let slice = unsafe {
            CStr::from_ptr( (*self.native).friendly_name )
        };
        slice.to_str().expect("UTF-8 error")
    }

    pub fn group_id(&self) -> &str {
        let slice = unsafe {
            CStr::from_ptr( (*self.native).group_id )
        };
        slice.to_str().expect("UTF-8 error")
    }

    pub fn vendor_name(&self) -> &str {
        let slice = unsafe {
            CStr::from_ptr( (*self.native).vendor_name )
        };
        slice.to_str().expect("UTF-8 error")
    }


    pub fn device_type(&self) -> DeviceType {
        DeviceType::from( unsafe { (*self.native).device_type } )
    }

    pub fn state(&self) -> DeviceState {
        DeviceState::from( unsafe { (*self.native).state } )
    }

    pub fn preferred(&self) -> DevicePref {
        DevicePref {
            bits: unsafe { (*self.native).preferred as u8 }
        }
    }


    pub fn format(&self) -> DeviceFmt {
        DeviceFmt {
            bits: unsafe { (*self.native).format as u16 }
        }
    }

    pub fn default_format(&self) -> DeviceFmt {
        DeviceFmt {
            bits: unsafe { (*self.native).default_format as u16 }
        }
    }

    pub fn max_channels(&self) -> u32 {
        unsafe { (*self.native).max_channels }
    }

    pub fn default_rate(&self) -> u32 {
        unsafe { (*self.native).default_rate }
    }

    pub fn max_rate(&self) -> u32 {
        unsafe { (*self.native).max_rate }
    }

    pub fn min_rate(&self) -> u32 {
        unsafe { (*self.native).min_rate }
    }


    pub fn latency_lo(&self) -> u32 {
        unsafe { (*self.native).latency_lo }
    }

    pub fn latency_hi(&self) -> u32 {
        unsafe { (*self.native).latency_hi }
    }
}

#[derive(Debug)]
pub struct DeviceCollection<'a> {
    ctx: &'a Context,
    native: cubeb_device_collection,
}

impl<'a> Iterator for DeviceCollection<'a> {
    type Item = DeviceInfo<'a>;
    fn next (&mut self) -> Option<Self::Item> {
        match self.native.count {
            0 => None,
            _ => {
                self.native.count -= 1;
                unsafe {
                    let res = DeviceInfo {
                        native: self.native.device,
                        phantom: PhantomData,
                    };
                    self.native.device = self.native.device.offset(1);
                    Some (res)
                }
            }
        }
    }
}

impl<'a> Drop for DeviceCollection<'a> {
    fn drop (&mut self) {
        unsafe {
            cubeb_device_collection_destroy(self.ctx.native, &mut self.native);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct StreamParams<T: Sample> {
    rate: u32,
    channels: u32,
    layout: ChannelLayout,
    phantom: PhantomData<T>,
}

impl<T: Sample> StreamParams<T> {
    pub fn new (rate: u32, channels: u32, layout: ChannelLayout) -> StreamParams<T> {
        StreamParams {
            rate: rate, channels: channels, layout: layout, phantom: PhantomData
        }
    }
}

impl<T: Sample> Into<cubeb_stream_params> for StreamParams<T> {
    fn into(self) -> cubeb_stream_params {
        cubeb_stream_params {
            format: T::format().into(),
            rate: self.rate,
            channels: self.channels,
            layout: self.layout.into(),
        }
    }
}


pub struct Stream<T: Sample> {
    native: *const cubeb_stream,
    data: Box<StreamData<T>>,
}

struct StreamData<T: Sample> {
    data_cb: DataCallback<T>,
    state_cb: Option<StateCallback>,
    in_channels: u32,
    out_channels: u32,
}

impl<T: Sample> Stream<T> {
    pub fn new(ctx: &Context, stream_name: &str,
               in_device: Option<&DevId>, in_params: Option<StreamParams<T>>,
               out_device: Option<&DevId>, out_params: Option<StreamParams<T>>,
               latency_frames: u32, data_cb: DataCallback<T>,
               state_cb: Option<StateCallback>) -> Result<Stream<T>> {
        let mut stm = ptr::null();

        let nat_state_cb: Option<cubeb_state_callback> = match state_cb.as_ref() {
            Some(_) => Some(state_callback_cb),
            None => Some(state_callback_noop),
        };
        let mut data = Box::new(StreamData {
            data_cb: data_cb,
            state_cb: state_cb,
            in_channels: in_params.as_ref().map_or(0, |p| p.channels),
            out_channels: out_params.as_ref().map_or(0, |p| p.channels),
        });

        let stream_name = CString::new(stream_name).unwrap();
        let in_params: Option<cubeb_stream_params> = in_params.map(|p| p.into());
        let out_params: Option<cubeb_stream_params> = out_params.map(|p| p.into());

        let res = unsafe {
            cubeb_stream_init(ctx.native, &mut stm, stream_name.as_ptr(),
                transmute(in_device), transmute(in_params.as_ref()),
                transmute(out_device), transmute(out_params.as_ref()),
                latency_frames, Some(T::data_cb_ffi()), nat_state_cb,
                &mut *data as *mut StreamData<T> as *mut c_void
            )
        };
        match res {
            CUBEB_OK => Ok(Stream {
                native: stm,
                data: data,
            }),
            _ => Err( Error::from(res) )
        }
    }

    pub fn start(&self) -> Result<()> {
        let res = unsafe {
            cubeb_stream_start(self.native)
        };
        match res {
            CUBEB_OK => Ok(()),
            _ => Err(Error::from(res)),
        }
    }

    pub fn stop(&self) -> Result<()> {
        let res = unsafe {
            cubeb_stream_stop(self.native)
        };
        match res {
            CUBEB_OK => Ok(()),
            _ => Err(Error::from(res)),
        }
    }

    pub fn reset_default_device(&self) -> Result<()> {
        let res = unsafe {
            cubeb_stream_reset_default_device(self.native)
        };
        match res {
            CUBEB_OK => Ok(()),
            _ => Err(Error::from(res)),
        }
    }

    pub fn position(&self) -> Result<u64> {
        let mut val = 0;
        let res = unsafe {
            cubeb_stream_get_position(self.native, &mut val)
        };
        match res {
            CUBEB_OK => Ok(val),
            _ => Err(Error::from(res)),
        }
    }

    pub fn latency(&self) -> Result<u32> {
        let mut val = 0;
        let res = unsafe {
            cubeb_stream_get_latency(self.native, &mut val)
        };
        match res {
            CUBEB_OK => Ok(val),
            _ => Err(Error::from(res)),
        }
    }

    pub fn set_volume(&self, volume: f32) -> Result<()> {
        let res = unsafe {
            cubeb_stream_set_volume(self.native, volume)
        };
        match res {
            CUBEB_OK => Ok(()),
            _ => Err(Error::from(res)),
        }
    }

    pub fn set_panning(&self, panning: f32) -> Result<()> {
        let res = unsafe {
            cubeb_stream_set_panning(self.native, panning)
        };
        match res {
            CUBEB_OK => Ok(()),
            _ => Err(Error::from(res)),
        }
    }
}

impl<T: Sample> Drop for Stream<T> {
    fn drop(&mut self) {
        unsafe { cubeb_stream_destroy(self.native); }
    }
}


pub fn print_state_change(state: State) {
    match state {
        State::Started => println!("stream started"),
        State::Stopped => println!("stream stopped"),
        State::Drained => println!("stream drained"),
        State::Error => println!("stream error"),
    }

}


extern fn data_callback_i16(_stm: *const cubeb_stream,
                            user: *mut c_void,
                            in_buf: *const c_void,
                            out_buf: *mut c_void,
                            nframes: c_long) -> c_long {
    unsafe {
        let data: &mut StreamData<i16> = transmute(user);
        let ibuf: &[i16] = slice::from_raw_parts(
            in_buf as *const i16, nframes as usize * data.in_channels as usize
        );
        let obuf: &mut [i16] = slice::from_raw_parts_mut(
            out_buf as *mut i16, nframes as usize * data.out_channels as usize
        );

        (data.data_cb)(ibuf, obuf) as c_long
    }
}

extern fn data_callback_f32(_stm: *const cubeb_stream,
                            user: *mut c_void,
                            in_buf: *const c_void,
                            out_buf: *mut c_void,
                            nframes: c_long) -> c_long {
    unsafe {
        let data: &mut StreamData<f32> = transmute(user);
        let ibuf: &[f32] = slice::from_raw_parts(
            in_buf as *const f32, nframes as usize * data.in_channels as usize
        );
        let obuf: &mut [f32] = slice::from_raw_parts_mut(
            out_buf as *mut f32, nframes as usize * data.out_channels as usize
        );

        (data.data_cb)(ibuf, obuf) as c_long
    }
}

extern fn state_callback_noop(_stm: *const cubeb_stream,
                               _user: *mut c_void,
                               _state: cubeb_state)
{}

extern fn state_callback_cb(_stm: *const cubeb_stream,
                             user: *mut c_void,
                             state: cubeb_state) {
    unsafe {
        let data: &mut StreamData<f32> = transmute(user);
        (*data.state_cb.as_mut().unwrap())(State::from(state));
    }
}


// #[cfg(feature = "plugins")]
// known_heap_size!(0, AudioStream);
