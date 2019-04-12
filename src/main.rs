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

#[allow(unused_variables)]
fn main() -> Result<(), Error> {
    let audioengine = audioengine::EngineController::start();

    let sample_rate = audioengine.sample_rate;
    let time_per_sample = 1.0 / sample_rate;

    let mut time = 0.0;

    let mut current_key = None;
    let synth = move |action: Option<i32>| {
        time += time_per_sample;
        if action != current_key {
            current_key = action;

            println!("{:?}", action);
        }
        if let Some(action) = action {
            let frequency = 440_f64 * 2_f64.powf((action as f64) / 12_f64);
            return (time * frequency * 2_f64 * 3.1415_f64).sin();
        } else {
            return 0_f64;
        }
    };

    audioengine.set_processor_function(Box::new(synth));

    let mut window = Ui::new(
        "Synthesizer",
        [1280.0, 800.0],
        audioengine,
        None,
        None,
        None,
    );

    window.show();

    Ok(())
}

#[derive(Debug)]
enum Error {}
