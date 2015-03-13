extern crate libc;
use libc::{c_void, c_uint, c_int};
use std::ptr;
use std::mem::{transmute};
use std::num::Float;
use std::rc;
use std::ffi::CString;
use std::io;
use std::io::prelude::*;

pub type cubeb_ptr = *mut c_void;
pub type cubeb_stream_ptr = *mut c_void;
pub type cubeb_data_callback = extern "C" fn(cubeb_stream_ptr,
                                             *mut libc::c_void,
                                             *mut libc::c_void,
                                             libc::c_long) -> libc::c_long;
pub type cubeb_state_callback = extern "C" fn(cubeb_stream_ptr,
                                              *mut libc::c_void,
                                              enum_cubeb_state);

pub type enum_cubeb_sample_format = c_uint;
pub const CUBEB_SAMPLE_S16LE: u32 = 0_u32;
pub const CUBEB_SAMPLE_S16BE: u32 = 1_u32;
pub const CUBEB_SAMPLE_FLOAT32LE: u32 = 2_u32;
pub const CUBEB_SAMPLE_FLOAT32BE: u32 = 3_u32;
pub const CUBEB_SAMPLE_S16NE: u32 = CUBEB_SAMPLE_S16LE;
pub const CUBEB_SAMPLE_FLOAT32NE: u32 = CUBEB_SAMPLE_FLOAT32LE;

pub type enum_cubeb_state = c_uint;
pub static CUBEB_STATE_STARTED: u32 = 0_u32; /**< Stream started. */
pub static CUBEB_STATE_STOPPED: u32 = 1_u32; /**< Stream stopped. */
pub static CUBEB_STATE_DRAINED: u32 = 2_u32; /**< Stream drained. */
pub static CUBEB_STATE_ERROR: u32 = 3_u32;   /**< Stream disabled due to error. */

pub type enum_cubeb_errors = c_int;
/* Success. */
pub static CUBEB_OK: i32 = 0_i32;
/* Unclassified error. */
pub static CUBEB_ERROR: i32 = -1_i32;
/* Unsupported cubeb_stream_params requested. */
pub static CUBEB_ERROR_INVALID_FORMAT: i32 = -2_i32;
/* Invalid parameter specified. */
pub static CUBEB_ERROR_INVALID_PARAMETER: i32 = -3_i32;


pub struct cubeb_stream_params {
  format: enum_cubeb_sample_format,
  rate: libc::uint32_t,
  channels: libc::uint32_t,
}

#[link(name = "cubeb")]
extern {
  fn cubeb_init(context: &mut cubeb_ptr, context_name: *const u8) -> libc::c_int;
  fn cubeb_get_backend_id(context: cubeb_ptr) -> *mut libc::c_char;
  fn cubeb_get_max_channel_count(context: cubeb_ptr,
                                 max_channels: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_get_min_latency(context: cubeb_ptr,
                           params: cubeb_stream_params,
                           latency_ms: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_get_preferred_sample_rate(context: cubeb_ptr,
                                     rate: *mut libc::uint32_t) -> libc::c_int;
  fn cubeb_destroy(context: cubeb_ptr);
  fn cubeb_stream_init(context: cubeb_ptr,
                       stream: *mut cubeb_stream_ptr,
                       stream_name: *const u8,
                       stream_params: cubeb_stream_params,
                       latency: libc::c_uint,
                       data_callback: cubeb_data_callback,
                       state_callback: cubeb_state_callback,
                       user_ptr: *mut AudioStream) -> libc::c_int;

  fn cubeb_stream_destroy(stream: cubeb_stream_ptr);
  fn cubeb_stream_start(stream: cubeb_stream_ptr) -> libc::c_int;
  fn cubeb_stream_stop(stream: cubeb_stream_ptr) -> libc::c_int;
  fn cubeb_stream_get_position(stream: cubeb_stream_ptr,
                               position: *mut libc::uint64_t) -> libc::c_int;
  fn cubeb_stream_get_latency(stream: cubeb_stream_ptr,
                              latency: *mut libc::uint32_t) -> libc::c_int;
}


pub struct AudioStream {
  rate: u32,
  format: enum_cubeb_sample_format,
  channels: u32,
  stream: cubeb_stream_ptr,
  phase: f32,
  ctx: std::rc::Rc<CubebContext>
}

impl AudioStream {
  pub fn new(ctx: std::rc::Rc<CubebContext>) -> AudioStream
  {
    return AudioStream {
      rate: 0,
      format: CUBEB_SAMPLE_FLOAT32NE,
      channels: 0,
      stream: ptr::null_mut(),
      phase: 0.0,
      ctx: ctx.clone()
    };
  }
  pub fn init(&mut self,
          rate: u32,
          channels: u32,
          format: enum_cubeb_sample_format,
          name: &str)
  {
    let mut rv = false;
    let cubeb_format = cubeb_stream_params {
       format: format,
       rate: rate,
       channels: channels
    };

    unsafe {
      self.rate = rate;
      self.format = format;
      self.channels = channels;
      self.phase = 0.;

      let cstr = CString::new(name).unwrap();

      rv = cubeb_stream_init(self.ctx.get(),
                             transmute::<&mut cubeb_stream_ptr,*mut cubeb_stream_ptr>(&mut self.stream),
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
  fn refill(&mut self, buffer: &mut [f32]) {
    let W = std::f32::consts::PI * 2.0 * 440. / (self.rate as f32);
    for i in range(0, buffer.len()) {
      for j in range(0, self.channels as usize) {
        buffer[i + j] = self.phase.sin();
      }
      self.phase += W;
    }
    println!("Clock: {}", self.clock());
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
    let mut rv = false;
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
    let mut rv = false;
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

extern fn refill_glue(stm: cubeb_stream_ptr,
                      user: *mut c_void,
                      buffer: *mut c_void,
                      nframes: libc::c_long) -> libc::c_long
{
  let fbuf: &mut[f32];
  let stream: *mut AudioStream;
  unsafe {
    stream = transmute(user as *mut AudioStream);
    fbuf = transmute((buffer as *mut f32, nframes * (*stream).channels as i64));
    (*stream).refill(fbuf);
  }

  nframes
}

extern fn state(stm: cubeb_stream_ptr,
                user: *mut c_void,
                state: enum_cubeb_state)
{
  unsafe {
    println!("state: {}", state);
  }
}


pub struct CubebContext {
  ctx: cubeb_ptr
}

impl CubebContext
{
  pub fn new(name: &str) -> CubebContext {
    let mut cubeb: cubeb_ptr = ptr::null_mut();
    let mut rv = false;
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
  fn get(&self) -> cubeb_ptr {
    self.ctx
  }
  pub fn backend_id(&self) ->  &'static str {
    // let backend_id : str;
    // unsafe {
    // backend_id = std::slice::from_raw_parts(cubeb_get_backend_id(self.get()), libc::strlen(self.get()))
    // }
    // backend_id
    ""
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
  pub fn min_latency(&self, params: cubeb_stream_params) -> u32 {
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

fn main() {
  let ctx: std::rc::Rc<CubebContext> = std::rc::Rc::new(CubebContext::new("rust-cubeb"));
  let mut astream = AudioStream::new(ctx.clone());

  astream.init(44100, 1, CUBEB_SAMPLE_FLOAT32NE, "rust-cubeb-stream0");
  astream.start();

      let stdin = io::stdin();
    for line in stdin.lock().lines() {
        println!("{}", line.unwrap());
    }

  astream.stop();
}

