extern crate cult;
use cult::*;
use std::sync::{Arc, Mutex, Condvar};

#[test]
fn sine() {
  let ctx: std::rc::Rc<CubebContext> =
    std::rc::Rc::new(CubebContext::new("rust-cubeb"));
  let mut astream = AudioStream::new(ctx.clone());
  let p1 = Arc::new((Mutex::new(false), Condvar::new()));
  let p2 = p1.clone();


  let &(ref m1, ref cv1) = &*p1;
  let g = m1.lock().unwrap();
  let mut phase: Box<f32> = Box::new(0.0);

  let cb: DataCallback = Box::new(move |buffer: &mut [f32]| {
    let w = std::f32::consts::PI * 2.0 * 440. / (44100 as f32);
    let &(ref m2, ref cv2) = &*p2;
    for i in 0 .. buffer.len() {
      for j in 0 .. 1 {
        buffer[i + j] = (*phase).sin();
      }
      (*phase) += w;
    }
    assert!(buffer.len() != 0);
    cv2.notify_one();
    buffer.len() as i32
  });

  astream.init(44100, 1, CUBEB_SAMPLE_FLOAT32NE, cb, "rust-cubeb-stream0");

  astream.start();

  cv1.wait(g).unwrap();

  astream.stop();
}

