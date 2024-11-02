#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level};
use embassy_nrf::pwm::SimplePwm;
use embassy_time::{Duration, Timer};
use microbit::board::Board;
use microbit::hal::temp::Temp;
use microbit_bsp::speaker::PwmSpeaker;
use microbit_bsp::*;
use {defmt_rtt as _, panic_probe as _};

/// `low_pin` の0はbtn_a, 1はbtn_bに対応していてそのピンがlowになるまでブロックする
async fn block_for_high<'a>(
    btn_a: &mut embassy_nrf::gpio::Input<'a, AnyPin>,
    btn_b: &mut embassy_nrf::gpio::Input<'a, AnyPin>,
    low_pin: (bool, bool),
) {
    loop {
        if (btn_a.is_high() || !low_pin.0) && (btn_b.is_high() || !low_pin.1) {
            break;
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();
    let board_not_embassy = Board::take().unwrap();

    let mut display = board.display;
    let mut btn_a = board.btn_a;
    let mut btn_b = board.btn_b;

    display.set_brightness(display::Brightness::new(128));
    defmt::info!("Application started, press buttons!");
    // let mut speacker = PwmSpeaker::new(SimplePwm::new_1ch(board.pwm0, board.speaker));
    let tmp_sen = Temp::new(board_not_embassy.TEMP);
    loop {
        defmt::info!("{} {}", btn_a.get_level(), btn_b.get_level());
        match (btn_a.get_level(), btn_b.get_level()) {
            (Level::High, Level::Low) => {
                block_for_high(&mut btn_a, &mut btn_b, (false, true)).await
            }
            (Level::Low, Level::High) => {
                block_for_high(&mut btn_a, &mut btn_b, (true, false)).await
            }
            // (Level::High, Level::Low) | (Level::Low, Level::High) => {
            //     let temp: f32 = tmp_sen.measure().to_num();
            //     let mut buf = [0u8; 10];
            //     let s = format_no_std::show(&mut buf, format_args!("{} C", temp as i32)).unwrap();
            //     display.scroll(&s).await;
            // }
            _ => (),
        }
        Timer::after_millis(100).await;
    }
}
