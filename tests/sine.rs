#![feature(core)]
#![feature(std_misc)]

extern crate cult;
extern crate libc;
use cult::*;
use std::num::Float;
use std::sync::{StaticMutex, MUTEX_INIT, StaticCondvar, CONDVAR_INIT};
use libc::c_void;
use std::io;
use std::io::prelude::*;
use std::mem::{transmute};

static CVAR: StaticCondvar = CONDVAR_INIT;
static M: StaticMutex = MUTEX_INIT;

#[test]
fn sine() {
  let ctx: std::rc::Rc<CubebContext> = std::rc::Rc::new(CubebContext::new("rust-cubeb"));
  let mut astream = AudioStream::new(ctx.clone());
  let g = M.lock().unwrap();
  let phase: f32 = 0.;

  astream.init(44100, 1, CUBEB_SAMPLE_FLOAT32NE, Box::new(|buffer: &mut [f32]| {
    let w = std::f32::consts::PI * 2.0 * 440. / (44100 as f32);
    for i in 0 .. buffer.len() {
      for j in range(0, 1) {
        buffer[i + j] = phase.sin();
      }
      phase += w;
    }
    assert!(buffer.len() != 0);
    CVAR.notify_one();
    buffer.len() as i32
  }) as DataCallback, "rust-cubeb-stream0");

  astream.start();

  let stdin = io::stdin();
  for line in stdin.lock().lines() {
    println!("{}", line.unwrap());
  }

  let g = CVAR.wait(g).unwrap();
  drop(g);

  astream.stop();
}

