
extern crate cult;

use std::{thread, time};

// 5 seconds of play through

const SAMPLE_RATE: f32 = 44100_f32;


fn main() {
    let ctx = cult::Context::new("Playthrough example", None).unwrap();

    println!("context open with {} backend", ctx.backend_id());

    let cb: cult::DataCallback<f32> = Box::new(move |ib: &[f32], ob: &mut [f32]| {
        for s in 0 .. ob.len() {
            ob[s] = ib[s];
        }
        ob.len()
    });

    let params = cult::StreamParams::<f32>::new(SAMPLE_RATE as u32, 1, cult::ChannelLayout::Mono);
    let min_latency = ctx.min_latency(params).expect("could not retrieve minimum latency");

    let stm = cult::Stream::<f32>::new(
        &ctx, "Playthrough",
        None, Some(params), None, Some(params),
        min_latency, cb, Some(Box::new(cult::print_state_change))
    ).expect("could not create audio stream");

    stm.start().unwrap();

    thread::sleep(time::Duration::from_millis(5000));

    stm.stop().unwrap();
}
