#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::Level;
use embassy_time::{Duration, Timer};
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};
use  microbit_bsp::speaker::Note;
use microbit_bsp::speaker::Pitch;
use microbit_bsp::speaker::NamedPitch;
use microbit_bsp::speaker::PwmSpeaker;
use embassy_nrf::pwm::SimplePwm;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();

    let mut display = board.display;
    let btn_a = board.btn_a;
    let btn_b = board.btn_b;

    display.set_brightness(display::Brightness::MAX);
    display.scroll("Hello, World!").await;
    defmt::info!("Application started, press buttons!");
    let sound = Note(Pitch::Named(NamedPitch::A4), 1000);
    let mut speacker = PwmSpeaker::new(SimplePwm::new_1ch(board.pwm0, board.speaker));
    loop {
        match (btn_a.get_level(), btn_b.get_level()) {
            (Level::Low, Level:: High) => {
                defmt::info!("A pressed");
                display
                    .display(display::fonts::ARROW_LEFT, Duration::from_secs(1))
                    .await;
            }
            (Level::High, Level::Low) => {
                defmt::info!("B pressed");
                let playing = speacker.play(&sound);
                display
                    .display(display::fonts::ARROW_RIGHT, Duration::from_secs(1))
                    .await;
                playing.await;
            }
            (Level::Low, Level::Low) => {
                defmt::info!("A and B pressed");
                display
                    .display(display::fonts::ARROW_RIGHT, Duration::from_secs(1))
                    .await;
                display.scroll("Hello, World!").await;
            }
            _ => {}
        }
        Timer::after_millis(100).await;
    }
}
