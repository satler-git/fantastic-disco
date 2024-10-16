#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::Level;
use embassy_time::{Duration, Timer};
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};
use microbit_bsp::speaker::PwmSpeaker;
use embassy_nrf::pwm::SimplePwm;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();

    let mut display = board.display;
    let btn_a = board.btn_a;
    let btn_b = board.btn_b;

    display.set_brightness(display::Brightness::new(128));
    defmt::info!("Application started, press buttons!");
    // let mut speacker = PwmSpeaker::new(SimplePwm::new_1ch(board.pwm0, board.speaker));
    loop {
        match (btn_a.get_level(), btn_b.get_level()) {
            _ => (),
        }
        Timer::after_millis(100).await;
    }
}
