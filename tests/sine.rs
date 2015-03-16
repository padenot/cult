#![feature(core)]
#![feature(std_misc)]

extern crate cult;
use cult::*;
use std::num::Float;
use std::sync::{StaticMutex, MUTEX_INIT, StaticCondvar, CONDVAR_INIT};

static CVAR: StaticCondvar = CONDVAR_INIT;
static M: StaticMutex = MUTEX_INIT;

#[test]
fn sine() {
  let ctx: std::rc::Rc<CubebContext> = std::rc::Rc::new(CubebContext::new("rust-cubeb"));
  let mut astream = AudioStream::new(ctx.clone());
  let g = M.lock().unwrap();
  let mut phase: Box<f32> = Box::new(0.0);

  let cb: DataCallback = Box::new(move |buffer: &mut [f32]| {
    let w = std::f32::consts::PI * 2.0 * 440. / (44100 as f32);
    for i in 0 .. buffer.len() {
      for j in range(0, 1) {
        buffer[i + j] = (*phase).sin();
      }
      (*phase) += w;
    }
    assert!(buffer.len() != 0);
    CVAR.notify_one();
    buffer.len() as i32
  });

  astream.init(44100, 1, CUBEB_SAMPLE_FLOAT32NE, cb, "rust-cubeb-stream0");

  astream.start();

  let g = CVAR.wait(g).unwrap();
  drop(g);

  astream.stop();
}

