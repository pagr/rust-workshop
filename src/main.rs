#[macro_use]
extern crate conrod;
extern crate conrod_derive;

extern crate audioengine;

mod event_loop;
mod types;
mod ui;

#[allow(unused_imports)]
use audioengine::types::{KeyAction, SignalBuffer};

use types::{GraphEvent, GraphEventType};

#[allow(unused_imports)]
use ui::Ui;

#[allow(unused_imports)]
use std::f64::consts::PI;

const C_FREQUENCY: f64 = 261.63;
const HALFSTEP_EXP: f64 = 1.059_463_094_36;

fn transform_key_action(action: Option<i32>) -> Option<f64> {
    action.map(|v| C_FREQUENCY * HALFSTEP_EXP.powi(v))
}

#[derive(Debug, Clone, Copy)]
enum ADSRState {
    Attack,
    Decay,
    Release
}

struct ADSR {
    state: ADSRState,
    value: f64,
    pub attack: f64,
    pub decay: f64,
    pub sustain: f64,
    pub release: f64
}

impl ADSR {
    pub fn new() -> Self {
        Self {
            state: ADSRState::Release,
            value: 0.0,
            attack: 10000.0,
            decay: 10000.0,
            sustain: 0.1,
            release: 10000.0
        }
    }

    pub fn process(&mut self, gate: f64) -> f64 {
        let trigger = gate >= 0.5;
        self.value = match self.state {
            ADSRState::Attack => (self.value + (gate / self.attack)).min(1.0),
            ADSRState::Decay => self.value - ((self.value - self.sustain) / self.decay),
            ADSRState::Release => self.value - (self.value / self.release)
        };
        self.state = match (self.state, trigger) {
            (_, false) => ADSRState::Release,
            (ADSRState::Attack, true) => if self.value > 0.99 { ADSRState::Decay } else { ADSRState::Attack },
            (ADSRState::Decay, true) => ADSRState::Decay,
            (ADSRState::Release, true) => ADSRState::Attack
        };
        self.value
    }
}

#[allow(unused_variables, unused_assignments)]
fn main() -> Result<(), Error> {
    let audioengine = audioengine::EngineController::start();

    let sample_rate = audioengine.sample_rate;
    let time_per_sample = 1.0 / sample_rate;

    let mut time = 0.0;
    let mut phase = 0.0;
    let mut freq = 440.0;
    let mut adsr = ADSR::new();

    let (sender, receiver) = std::sync::mpsc::channel::<GraphEvent>();
    let mut signal_buffer = SignalBuffer::new();

    let synth = move |action: Option<i32>| {
        time += time_per_sample;

        if let Some(new_freq) = transform_key_action(action) {
            freq = new_freq;
        }
        phase += freq * time_per_sample * 2.0 * PI;

        let mut phase_crossed_zero = false;
        if phase > PI {
            phase -= 2.0 * PI;
            phase_crossed_zero = true;
        }

        let gate = match action {
            Some(_) => 1.0,
            None => 0.0
        };

        let my_value = phase.sin() * adsr.process(gate);

        signal_buffer.push_back(my_value);

        if phase_crossed_zero {
            sender.send((GraphEventType::SignalGraph, signal_buffer.clone(), 4410)).expect("Unable to send graph data to UI.");
            signal_buffer.clear();
        }

        my_value
    };

    audioengine.set_processor_function(Box::new(synth));

    let mut window = Ui::new(
        "Synthesizer",
        [1280.0, 800.0],
        audioengine,
        None,
        None,
        Some(receiver),
    );

    window.show();

    Ok(())
}

#[derive(Debug)]
enum Error {}
