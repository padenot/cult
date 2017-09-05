extern crate cult;

use std::sync::{Arc, Mutex, Condvar};

#[test]
fn sine() {
  let ctx = cult::Context::new("rust-cubeb", None).unwrap();
  let p1 = Arc::new((Mutex::new(false), Condvar::new()));
  let p2 = p1.clone();

  let &(ref m1, ref cv1) = &*p1;
  let g = m1.lock().unwrap();
  let mut phase: Box<f32> = Box::new(0.0);

  let cb: cult::DataCallback<f32> = Box::new(move |_: &[f32], obuf: &mut [f32]| {
    let w = std::f32::consts::PI * 2_f32 * 440_f32 / 44100_f32;
    let &(ref _m2, ref cv2) = &*p2;
    for i in 0 .. obuf.len() {
      for j in 0 .. 1 {
        obuf[i + j] = (*phase).sin();
      }
      (*phase) += w;
    }
    assert!(obuf.len() != 0);
    cv2.notify_one();
    obuf.len()
  });

  let params = cult::StreamParams::<f32>::new(44100, 1, cult::ChannelLayout::Mono);
  let min_latency = ctx.min_latency(params).expect("could not retrieve minimum latency");

  let astream = cult::Stream::<f32>::new(
      &ctx, "rust-cubeb-stream0",
      None, None, None, Some(params),
      min_latency, cb, Some(Box::new(cult::print_state_change))
  ).expect("could not create audio stream");

  astream.start().unwrap();

  cv1.wait(g).unwrap();

  astream.stop().unwrap();
}

