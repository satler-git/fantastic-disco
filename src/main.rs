#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::Level;
use embassy_time::{Duration, Delay};
use embedded_hal_async::delay::DelayNs;
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();
    let mut delay = Delay;

    let mut display = board.display;
    let btn_a = board.btn_a;
    let btn_b = board.btn_b;

    display.set_brightness(display::Brightness::MAX);
    display.scroll("Hello, World!").await;
    defmt::info!("Application started, press buttons!");
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
                display
                    .display(display::fonts::ARROW_RIGHT, Duration::from_secs(1))
                    .await;
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
        delay.delay_ms(50).await;
    }
}
