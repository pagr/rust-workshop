#[macro_use]
extern crate conrod;
extern crate conrod_derive;

extern crate audioengine;

mod event_loop;
mod types;
mod ui;

#[allow(unused_imports)]
use audioengine::types::KeyAction;

#[allow(unused_imports)]
use ui::Ui;

#[allow(unused_imports)]
use std::f64::consts::PI;

use std::sync::mpsc::channel;

#[derive(Copy, Clone)]
enum State{
    Attack,
    Decay,
    Sustain,
    Release
}

#[allow(unused_variables)]
fn main() -> Result<(), Error> {
    let audioengine = audioengine::EngineController::start();

    let sample_rate = audioengine.sample_rate;
    let time_per_sample = 1.0 / sample_rate;

    let mut time = 0.0;

    // Create channel for communication between UI and Audio thread
    let (sample_sender, sample_receiver) = channel::<Vec<f64>>();
    let mut buffer = [0_f64; 100];
    let mut buffer_index = 0;

    let mut current_key = None;
    let mut frequency: f64 = 440_f64;

    let mut gate = 0_f64;
    let mut value = 0_f64;
    let mut attack = 1000_f64;
    let mut decay = 1000_f64;
    let mut sustain = 0.5_f64;
    let mut release = 1000_f64;
    let mut state = State::Release;
    let synth = move |action: Option<i32>| {
        time += time_per_sample;
        if action != current_key {
            current_key = action;

            println!("{:?}", action);
        }
        let sample: f64;
        if let Some(action) = action {
            frequency = 440_f64 * 2_f64.powf((action as f64) / 12_f64);
            gate = 1_f64;
        } else {
            gate = 0_f64;
        }
        match state {
            State::Attack => {
                value += 1_f64 / attack;
                if value >= 1_f64 { state = State::Decay; }
                if gate < 0.5_f64 { state = State::Release; }
            },
            State::Decay => {
                value = value - (value - sustain)/decay;
                if gate < 0.5_f64 { state = State::Release; }
            },
            State::Sustain => (),
            State::Release => {
                value = value - value / release;
                if gate > 0.5_f64 { state = State::Attack; }
            },
        }
        value = value.min(1_f64).max(0_f64);
        sample = (time * frequency * 2_f64 * 3.1415_f64).sin().round() * value;

        buffer[buffer_index] = sample;
        if buffer_index == buffer.len() - 1 {
            let _ = sample_sender.send(buffer.to_vec());
            buffer_index = 0;
        } else {
           buffer_index = buffer_index + 1;
        }
        return sample
    };
    audioengine.set_processor_function(Box::new(synth));

    let mut window = Ui::new(
        "Synthesizer",
        [1280.0, 800.0],
        audioengine,
        None,
        None,
        Some(sample_receiver),
    );

    window.show();

    Ok(())
}

#[derive(Debug)]
enum Error {}
