#![no_std]
#![no_main]

use core::ops::{Range, RangeInclusive};

use display::{Bitmap, Frame};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{AnyPin, Level};
use embassy_nrf::pwm::SimplePwm;
use embassy_time::{Duration, Instant, Timer};
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

struct State {
    pub times: [OneBlock; 256],
    pub mode: Mode,
    /// FrameCount, オーバーフローしたらリセットしていい。
    /// 時間を計測するために使わないこと
    pub counter: u64,
}

type ScrollPoint = u8;

enum Mode {
    ModeSelect,
    Timer {
        from: Instant,
        duration: Duration,
        reverse: bool,
    },
    Viewer(ScrollPoint),
}

impl State {
    fn render(&self) -> Frame<5, 5> {
        match self.mode {
            Mode::ModeSelect => todo!(),
            Mode::Viewer(_) => self.render_viewer(),
            Mode::Timer {
                from: _,
                duration: _,
                reverse: _,
            } => self.mode.render_timer(),
        }
    }

    fn render_viewer(&self) -> Frame<5, 5> {
        if let Mode::Viewer(sp) = self.mode {
            let mut f = Frame::empty();
            if sp >= 5 {
                defmt::error!(
                    "ScrollPoint is bigger than 4, in this time, show you p4(p0 is first page)."
                );
            }
            f.set(sp.min(4) as usize, 0); // ScrollBar
                                          // TODO:
            f
        } else {
            defmt::error!(
                "A non-Viewer Mode was passed to `render_viewer`. This is probably an error."
            );
            Frame::empty()
        }
    }
}

impl Mode {
    fn render_timer(&self) -> Frame<5, 5> {
        if let Mode::Timer {
            from,
            duration,
            reverse,
        } = self
        {
            let mut f = Frame::empty();
            let kyori = {
                let mut kyori = from.as_millis() / (duration.as_millis() / 9);
                kyori = kyori.min(9);
                if *reverse {
                    kyori = 9 - kyori;
                }
                kyori
            };
            defmt::info!("kyori: {}", kyori);
            for x in 0..5 {
                for y in 0..5 {
                    if x + y <= kyori - 1 {
                        f.set(x as usize, y as usize);
                    }
                }
            }
            f
        } else {
            defmt::error!(
                "A non-Timer Mode was passed to `render_timer`. This is probably an error."
            );
            Frame::empty()
        }
    }
}

struct OneBlock {
    /// 5分 * `count`
    /// 切り捨て
    pub count: u8,
    pub kind: BlockKind,
}

enum BlockKind {
    Rest,
    // typeを記録できるように
    Other(u8),
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();
    let board_not_embassy = Board::take().unwrap();

    let mut display = board.display;
    // display
    //     .display(
    //         Mode::render_timer(&Mode::Timer {
    //             from: Instant::from_secs(40),
    //             duration: Duration::from_secs(60),
    //             reverse: false,
    //         }),
    //         Duration::from_secs(10),
    //     )
    //     .await;
    let mut btn_a = board.btn_a;
    let mut btn_b = board.btn_b;

    defmt::info!("Application started, press buttons!");
    // let mut speacker = PwmSpeaker::new(SimplePwm::new_1ch(board.pwm0, board.speaker));
    let tmp_sen = Temp::new(board_not_embassy.TEMP);
    let mut state = false;
    loop {
        if state {
            let frame = Frame::new([
                Bitmap::new(0b11000, 5),
                Bitmap::new(0b11000, 5),
                Bitmap::new(0b11000, 5),
                Bitmap::new(0b11000, 5),
                Bitmap::new(0b11000, 5),
            ]);
            display.set_brightness(display::Brightness::new(4));
            display.display(frame, Duration::from_millis(2)).await;
            state = false;
        } else {
            let frame = Frame::new([
                Bitmap::new(0b00011, 5),
                Bitmap::new(0b00011, 5),
                Bitmap::new(0b00011, 5),
                Bitmap::new(0b00011, 5),
                Bitmap::new(0b00011, 5),
            ]);
            display.set_brightness(display::Brightness::new(255));
            display.display(frame, Duration::from_millis(2)).await;
            state = true;
        }
        // defmt::info!("{} {}", btn_a.get_level(), btn_b.get_level());
        // match (btn_a.get_level(), btn_b.get_level()) {
        //     (Level::High, Level::Low) => {
        //         block_for_high(&mut btn_a, &mut btn_b, (false, true)).await
        //     }
        //     (Level::Low, Level::High) => {
        //         block_for_high(&mut btn_a, &mut btn_b, (true, false)).await
        //     }
        //     // (Level::High, Level::Low) | (Level::Low, Level::High) => {
        //     //     let temp: f32 = tmp_sen.measure().to_num();
        //     //     let mut buf = [0u8; 10];
        //     //     let s = format_no_std::show(&mut buf, format_args!("{} C", temp as i32)).unwrap();
        //     //     display.scroll(&s).await;
        //     // }
        //     _ => (),
        // }
        // Timer::after_millis(100).await;
    }
}
