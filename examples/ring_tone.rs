
extern crate cult;

use std::{thread, time};

// Japanese tone: 400 Hz tone with amplitude modulation of 20 Hz.
// 1 sec tone with 2 sec pause
// smoothstep is used to avoid glitches

const SAMPLE_RATE: f32 = 44100_f32;
const STEP_LEN: f32 = 0.050_f32;

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = clamp((x - edge0)/(edge1 - edge0), 0_f32, 1_f32);
    x*x*(3_f32 - x*2_f32)
}

fn clamp(x: f32, low: f32, high: f32) -> f32 {
    if x < low { low }
    else if x > high { high }
    else { x }
}

fn main() {
    let ctx = cult::Context::new("Tone example", None).unwrap();

    println!("context open with {} backend", ctx.backend_id());

    let phase_pulse = std::f32::consts::PI * 2_f32 * 400_f32 / SAMPLE_RATE;
    let mod_phase_pulse = std::f32::consts::PI * 2_f32 * 20_f32 / SAMPLE_RATE;

    #[derive(Debug, Copy, Clone)]
    struct State {
        phase: f32,         // rad
        mod_phase: f32,     // rad
        tone_phase: f32,    // sec
    }
    let mut state = Box::new(State {
        phase: 0_f32, mod_phase: 0_f32, tone_phase: 0_f32
    });

    let cb: cult::DataCallback<f32> = Box::new(move |_: &[f32], outp: &mut [f32]| {
        let mut st = *state;
        for s in 0 .. outp.len() {

            let step = if st.tone_phase < 0_f32 {
                0_f32
            }
            else if st.tone_phase < STEP_LEN {
                smoothstep(0_f32, STEP_LEN, st.tone_phase)
            }
            else if st.tone_phase < 1_f32 {
                1.0_f32
            }
            else if st.tone_phase < 1_f32 + STEP_LEN {
                1_f32 - smoothstep(1_f32, 1_f32 + STEP_LEN, st.tone_phase)
            }
            else {
                0_f32
            };

            let sample = if step > 0_f32 {
                let modulation = 0.5_f32 * (1_f32 + st.mod_phase.sin());
                step * st.phase.sin() * modulation
            }
            else {
                0_f32
            };

            for c in 0 .. 1 {
                outp[s + c] = sample;
            }

            st.phase += phase_pulse;
            st.mod_phase += mod_phase_pulse;
            st.tone_phase += 1_f32 / SAMPLE_RATE;

            if st.phase > std::f32::consts::PI * 2_f32 {
                st.phase -= std::f32::consts::PI * 2_f32;
            }
            if st.mod_phase > std::f32::consts::PI * 2_f32 {
                st.mod_phase -= std::f32::consts::PI * 2_f32;
            }
            // overflow in the middle of the pause to avoid jump within step
            if st.tone_phase > 2_f32 {
                st.tone_phase -= 3_f32;
            }
        }
        (*state) = st;
        outp.len()
    });

    let params = cult::StreamParams::<f32>::new(SAMPLE_RATE as u32, 1, cult::ChannelLayout::Mono);
    let min_latency = ctx.min_latency(params).expect("could not retrieve minimum latency");

    let stm = cult::Stream::<f32>::new(
        &ctx, "Sine",
        None, None, None, Some(params),
        min_latency, cb, Some(Box::new(cult::print_state_change))
    ).expect("could not create audio stream");

    stm.start().unwrap();

    // 3 tones
    thread::sleep(time::Duration::from_millis(7000 + (1000 as f32*STEP_LEN) as u64));

    stm.stop().unwrap();
}
