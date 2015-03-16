extern crate libc;
use libc::{c_void, c_uint, c_int};
use std::ptr;
use std::mem::{transmute};
use std::ffi::CString;
use std::ffi::CStr;

pub type CubebPtr = *mut c_void;
pub type CubebStreamPtr = *mut c_void;
pub type CubebDataCallback = extern "C" fn(CubebStreamPtr,
                                             *mut libc::c_void,
                                             *mut libc::c_void,
                                             libc::c_long) -> libc::c_long;
pub type CubebStateCallback = extern "C" fn(CubebStreamPtr,
                                              *mut libc::c_void,
                                              CubebState);

pub type CubebSampleFormat = c_uint;
pub const CUBEB_SAMPLE_S16LE: u32 = 0_u32;
pub const CUBEB_SAMPLE_S16BE: u32 = 1_u32;
pub const CUBEB_SAMPLE_FLOAT32LE: u32 = 2_u32;
pub const CUBEB_SAMPLE_FLOAT32BE: u32 = 3_u32;
pub const CUBEB_SAMPLE_S16NE: u32 = CUBEB_SAMPLE_S16LE;
pub const CUBEB_SAMPLE_FLOAT32NE: u32 = CUBEB_SAMPLE_FLOAT32LE;

pub type CubebState = c_uint;
pub static CUBEB_STATE_STARTED: u32 = 0_u32; /**< Stream started. */
pub static CUBEB_STATE_STOPPED: u32 = 1_u32; /**< Stream stopped. */
pub static CUBEB_STATE_DRAINED: u32 = 2_u32; /**< Stream drained. */
pub static CUBEB_STATE_ERROR: u32 = 3_u32;   /**< Stream disabled due to error. */

pub type CubebErrors = c_int;
/* Success. */
pub static CUBEB_OK: i32 = 0_i32;
/* Unclassified error. */
pub static CUBEB_ERROR: i32 = -1_i32;
/* Unsupported CubebStreamParams requested. */
pub static CUBEB_ERROR_INVALID_FORMAT: i32 = -2_i32;
/* Invalid parameter specified. */
pub static CUBEB_ERROR_INVALID_PARAMETER: i32 = -3_i32;


#[repr(C)]
pub struct CubebStreamParams {
  format: CubebSampleFormat,
  rate: libc::uint32_t,
  channels: libc::uint32_t,
}

#[link(name = "cubeb")]
extern {
  fn cubeb_init(context: &mut CubebPtr, context_name: *const u8) -> libc::c_int;
  fn cubeb_get_backend_id(context: CubebPtr) -> *const libc::c_char;
  fn cubeb_get_max_channel_count(context: CubebPtr,
                                 max_channels: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_get_min_latency(context: CubebPtr,
                           params: CubebStreamParams,
                           latency_ms: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_get_preferred_sample_rate(context: CubebPtr,
                                     rate: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_destroy(context: CubebPtr);
  fn cubeb_stream_init(context: CubebPtr,
                       stream: *mut CubebStreamPtr,
                       stream_name: *const u8,
                       stream_params: CubebStreamParams,
                       latency: libc::c_uint,
                       data_callback: CubebDataCallback,
                       state_callback: CubebStateCallback,
                       user_ptr: *const AudioStream) -> libc::c_int;

  fn cubeb_stream_destroy(stream: CubebStreamPtr);
  fn cubeb_stream_start(stream: CubebStreamPtr) -> libc::c_int;
  fn cubeb_stream_stop(stream: CubebStreamPtr) -> libc::c_int;
  fn cubeb_stream_get_position(stream: CubebStreamPtr,
                               position: *mut libc::uint64_t) -> libc::c_int;
  fn cubeb_stream_get_latency(stream: CubebStreamPtr,
                              latency: *mut libc::uint32_t) -> libc::c_int;
}

pub type DataCallback = Box<FnMut(&mut [f32]) -> i32>;

fn noopcallback(buffer: &mut [f32]) -> i32
{
  buffer.len() as i32
}

pub struct AudioStream {
  rate: u32,
  format: CubebSampleFormat,
  channels: u32,
  stream: CubebStreamPtr,
  ctx: std::rc::Rc<CubebContext>,
  callback: DataCallback
}

impl AudioStream {
  pub fn new(ctx: std::rc::Rc<CubebContext>) -> AudioStream
  {
    return AudioStream {
      rate: 0,
      format: CUBEB_SAMPLE_FLOAT32NE,
      channels: 0,
      stream: ptr::null_mut(),
      ctx: ctx.clone(),
      callback: Box::new(|_| 0)
    };
  }
  pub fn init(&mut self,
                 rate: u32,
                 channels: u32,
                 format: CubebSampleFormat,
                 callback: DataCallback,
                 name: &str)
  {
    let mut rv;
    let cubeb_format = CubebStreamParams {
       format: format,
       rate: rate,
       channels: channels
    };

    unsafe {
      self.rate = rate;
      self.format = format;
      self.channels = channels;
      self.callback = callback;

      let cstr = CString::new(name).unwrap();

      rv = cubeb_stream_init(self.ctx.get(),
                             transmute::<&mut CubebStreamPtr,*mut CubebStreamPtr>(&mut self.stream),
                             cstr.as_bytes_with_nul().as_ptr(),
                             cubeb_format,
                             40,
                             refill_glue,
                             state,
                             self) == 0;
      if !rv {
        println!("Error.");
      }
    }
  }
  pub fn start(&self) {
    unsafe {
      cubeb_stream_start(self.stream);
    }
  }
  pub fn stop(&self) {
    unsafe {
      cubeb_stream_stop(self.stream);
    }
  }
  pub fn clock(&self) -> u64 {
    let mut pos: libc::uint64_t = 0;
    let mut rv;
    unsafe {
      rv = cubeb_stream_get_position(self.stream, &mut pos) == CUBEB_OK;
    }
    if !rv {
      println!("clock() failed.");
      return 0
    }
    pos
  }
  pub fn latency(&self) -> u32 {
    let mut lat: libc::uint32_t = 0;
    let mut rv;
    unsafe {
      rv = cubeb_stream_get_latency(self.stream, &mut lat) == CUBEB_OK;
    }
    if !rv {
      println!("latency() failed.");
      return 0
    }
    lat
  }
}

impl Drop for AudioStream {
  fn drop(&mut self) {
    unsafe {
      println!("stream destroy.");
      cubeb_stream_destroy(self.stream);
    }
  }
}

extern fn refill_glue(stm: CubebStreamPtr,
                      user: *mut c_void,
                      buffer: *mut c_void,
                      nframes: libc::c_long) -> libc::c_long
{
  let fbuf: &mut[f32];
  let stream: *mut AudioStream;
  unsafe {
    stream = transmute(user as *mut AudioStream);
    fbuf = transmute((buffer as *mut f32, nframes * (*stream).channels as i64));
    ((*stream).callback)(fbuf);
  }

  nframes
}

extern fn state(stm: CubebStreamPtr,
                user: *mut c_void,
                state: CubebState)
{
  println!("state: {}", state);
}


pub struct CubebContext {
  ctx: CubebPtr
}

impl CubebContext
{
  pub fn new(name: &str) -> CubebContext {
    let mut cubeb: CubebPtr = ptr::null_mut();
    let mut rv;
    let cstr = CString::new(name).unwrap();
    unsafe {
      rv = cubeb_init(transmute(&mut cubeb), cstr.as_bytes_with_nul().as_ptr()) == 0;
    }
    if !rv {
      println!("cubeb_init failed.");
    }
    CubebContext {
      ctx: cubeb
    }
  }
  fn get(&self) -> CubebPtr {
    self.ctx
  }
  pub fn backend_id(&self) ->  &'static str {
    let chars : *const i8;
    unsafe {
      chars = cubeb_get_backend_id(self.get());
      let slice = CStr::from_ptr(chars).to_bytes();
      return std::str::from_utf8(slice).unwrap();
    }
  }
  pub fn max_channel_count(&self) -> u32 {
    let mut max_channel_count: u32 = 0;
    unsafe {
      if cubeb_get_max_channel_count(self.get(), &mut max_channel_count) != CUBEB_OK {
        println!("failed.");
      }
    }
    max_channel_count
  }
  pub fn min_latency(&self, params: CubebStreamParams) -> u32 {
    let mut min_latency: u32 = 0;
    unsafe {
      if cubeb_get_min_latency(self.get(), params, &mut min_latency) !=
        CUBEB_OK {
        println!("failed.");
      }
    }
    min_latency
  }
  pub fn preferred_sample_rate(&self) -> u32 {
    let mut sr : u32 = 0;
    unsafe {
      if cubeb_get_preferred_sample_rate(self.get(), &mut sr) != CUBEB_OK {
        println!("failed.");
      }
    }
    sr
  }
}

impl Drop for CubebContext {
  fn drop(&mut self) {
    unsafe {
      println!("Cubeb_destroy.");
      cubeb_destroy(self.ctx);
    }
  }
}

