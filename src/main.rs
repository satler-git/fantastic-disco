#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::Level;
use embassy_time::{Duration, Timer};
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};
use microbit_bsp::speaker::PwmSpeaker;
use embassy_nrf::pwm::SimplePwm;
use microbit::board::Board;
use microbit::hal::temp::Temp;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();
    let board_not_embassy = Board::take().unwrap();

    let mut display = board.display;
    let btn_a = board.btn_a;
    let btn_b = board.btn_b;

    display.set_brightness(display::Brightness::new(128));
    defmt::info!("Application started, press buttons!");
    // let mut speacker = PwmSpeaker::new(SimplePwm::new_1ch(board.pwm0, board.speaker));
    let mut tmp_sen = Temp::new(board_not_embassy.TEMP);
    loop {
        match (btn_a.get_level(), btn_b.get_level()) {
            (Level::High, Level::Low) | (Level::Low, Level::High) => {
                let temp: f32 = tmp_sen.measure().to_num();
                let mut buf = [0u8; 10];
                let s = format_no_std::show(
                    &mut buf,
                    format_args!("{} C", temp as i32),
                ).unwrap();
                display.scroll(&s).await;
            }
            _ => (),
        }
        Timer::after_millis(100).await;
    }
}
