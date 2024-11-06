#![no_std]
#![no_main]

use core::default;
use core::ops::{Range, RangeInclusive};

use heapless::Vec;

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

const DEFAULT_INTERVAL: u32 = 50;

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

#[derive(defmt::Format, PartialEq, Eq)]
struct State {
    pub times: Vec<Block, 100>, // 4 * 5 * 5 // lines * rows * pages
    pub mode: Mode,
    /// FrameCount, オーバーフローしたらリセットしていい。
    /// 時間を計測するために使わないこと
    pub counter: u64,
}

type ScrollPoint = u8;

#[derive(defmt::Format, PartialEq, Eq)]
enum Mode {
    ModeSelect(u8),
    Timer {
        from: Instant,
        duration: Duration,
        reverse: bool,
    },
    Viewer(ScrollPoint),
}

impl State {
    async fn event_loop(self, board: Microbit) {
        let mut board = board;
        let btn_a = board.btn_a;
        let btn_b = board.btn_b;
        let mut bottun_state = (Level::Low, Level::Low);
        let mut state = self;
        let mut interval = DEFAULT_INTERVAL;
        loop {
            let now_bottun_state = (btn_a.get_level(), btn_b.get_level());
            if now_bottun_state.0 != bottun_state.0 {
                bottun_state.0 = now_bottun_state.0;
            } else {
                bottun_state.0 = Level::High;
            }
            if now_bottun_state.1 != bottun_state.1 {
                bottun_state.1 = now_bottun_state.1;
            } else {
                bottun_state.1 = Level::High;
            }
            (state, interval) = state.new_state(bottun_state, interval);
            board
                .display
                .display(state.render(), Duration::from_millis(50))
                .await;
            Timer::after_millis(interval as u64).await;
            // defmt::debug!("{:?}", state);
            // defmt::debug!("{}", now_bottun_state);
        }
    }

    fn render(&self) -> Frame<5, 5> {
        match self.mode {
            Mode::ModeSelect(_) => self.render_select(),
            Mode::Viewer(_) => self.render_viewer(),
            Mode::Timer {
                from: _,
                duration: _,
                reverse: _,
            } => self.mode.render_timer(),
        }
    }

    fn render_select(&self) -> Frame<5, 5> {
        if let Mode::ModeSelect(s) = self.mode {
            if s == 1 {
                Frame::new([
                    Bitmap::new(0b11000, 5),
                    Bitmap::new(0b11000, 5),
                    Bitmap::new(0b11000, 5),
                    Bitmap::new(0b11000, 5),
                    Bitmap::new(0b11000, 5),
                ])
            } else if s == 2 {
                Frame::new([
                    Bitmap::new(0b00011, 5),
                    Bitmap::new(0b00011, 5),
                    Bitmap::new(0b00011, 5),
                    Bitmap::new(0b00011, 5),
                    Bitmap::new(0b00011, 5),
                ])
            } else {
                Frame::new([
                    Bitmap::new(0b00100, 5),
                    Bitmap::new(0b00100, 5),
                    Bitmap::new(0b00100, 5),
                    Bitmap::new(0b00100, 5),
                    Bitmap::new(0b00100, 5),
                ])
            }
        } else {
            Frame::empty()
        }
    }

    /// ブロッキングしても良い
    /// -> (new_state, final_sleep_mills)
    fn new_state(self, input: (Level, Level), prev_sleep_mills: u32) -> (Self, u32) {
        let def = |s: State| -> (State, u32) {
            (
                State {
                    times: s.times,
                    mode: s.mode,
                    counter: s.counter + 1,
                },
                prev_sleep_mills,
            )
        };
        let mut s = self;
        match s.mode {
            Mode::ModeSelect(sp) => {
                let n = {
                    if input.0 == Level::Low {
                        if sp == 1 {
                            Mode::Timer {
                                from: Instant::now(),
                                duration: Duration::from_secs(60 * 25),
                                reverse: false,
                            }
                        } else if sp == 2 {
                            Mode::ModeSelect(0)
                        } else {
                            Mode::ModeSelect(1)
                        }
                    } else if input.1 == Level::Low {
                        if sp == 1 {
                            Mode::ModeSelect(0)
                        } else if sp == 2 {
                            Mode::Viewer(0)
                        } else {
                            Mode::ModeSelect(2)
                        }
                    } else {
                        Mode::ModeSelect(sp)
                    }
                };
                (
                    State {
                        times: s.times,
                        mode: n,
                        counter: s.counter + 1,
                    },
                    prev_sleep_mills,
                )
            }
            Mode::Timer {
                from,
                duration,
                reverse,
            } => {
                if (Instant::now().as_millis() - from.as_millis()) >= duration.as_millis() {
                    // defmt::debug!(
                    //     "from: {}, duration: {}",
                    //     (Instant::now().as_millis() - from.as_millis()),
                    //     duration.as_millis()
                    // );
                    if reverse {
                        s.times.push(Block {
                            count: 1,
                            kind: BlockKind::Rest,
                        });
                        (
                            State {
                                times: s.times,
                                mode: Mode::ModeSelect(0), // 中間
                                counter: s.counter + 1,
                            },
                            prev_sleep_mills,
                        )
                    } else {
                        s.times.push(Block {
                            count: 5,
                            kind: BlockKind::Other(0),
                        });
                        (
                            State {
                                times: s.times,
                                mode: Mode::Timer {
                                    from: Instant::now(),
                                    duration: Duration::from_secs(60 * 5),
                                    reverse: true,
                                },
                                counter: s.counter + 1,
                            },
                            prev_sleep_mills,
                        )
                    }
                } else {
                    def(s)
                }
            }
            Mode::Viewer(sp) => {
                let new_mode = {
                    if input == (Level::High, Level::High) {
                        Mode::ModeSelect(0)
                    } else if input.1 == Level::High {
                        Mode::Viewer((sp + 1).min(4))
                    } else if input.0 == Level::High {
                        Mode::Viewer(((sp as i16) - 1).max(0) as u8)
                    } else {
                        Mode::Viewer(sp)
                    }
                };
                (
                    State {
                        times: s.times,
                        mode: new_mode,
                        counter: s.counter + 1,
                    },
                    prev_sleep_mills,
                )
            }
        }
    }

    fn render_viewer(&self) -> Frame<5, 5> {
        if let Mode::Viewer(sp) = self.mode {
            if sp >= 5 {
                // page 5以上にアクセスしようとしている。
                defmt::error!(
                    "ScrollPoint is bigger than 4, in this time, show you p4(p0 is first page)."
                );
            }
            let sp = sp.min(4) as usize;
            let mut f = Frame::new([
                Bitmap::new(0b00000, 5),
                Bitmap::new(0b11111, 5),
                Bitmap::new(0b11111, 5),
                Bitmap::new(0b11111, 5),
                Bitmap::new(0b11111, 5),
            ]);
            f.set(sp, 0); // ScrollBar
            let mut diff = 0; // 一番最初のブロックがどれだけ入るのか。
            let display_blocks = {
                let mut ue_shita = (0, 0);
                let mut times = 0;
                let time_border = 20 * sp;
                for (ii, bi) in self.times.iter().enumerate() {
                    times += bi.count as usize;
                    if time_border <= times && ue_shita == (0, 0) {
                        ue_shita.0 = ii;
                        diff = times - time_border;
                    }
                    if time_border + 20 <= times {
                        ue_shita.1 = ii;
                        break;
                    }
                }
                &self.times[ue_shita.0..=ue_shita.1]
            };
            let off = (self.counter % display_blocks.len() as u64) as usize;
            let off_mae = display_blocks[1..off].iter().map(|x| x.count).sum::<u8>() + diff as u8;
            let off_at = off_mae + display_blocks[off].count;
            let mut times = 0;
            for y in 0..5 {
                for x in 0..5 {
                    times += 1;
                    if off_mae <= times && off_at >= times {
                        f.set(x, y);
                    }
                }
            }
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
                let mut kyori =
                    (Instant::now().as_millis() - from.as_millis()) / (duration.as_millis() / 9);
                kyori = kyori.min(9);
                if *reverse {
                    kyori = 9 - kyori;
                }
                kyori
            };
            defmt::info!("kyori: {}", kyori);
            defmt::info!("jikan: {}", (Instant::now().as_millis() - from.as_millis()));
            for x in 0..5 {
                for y in 0..5 {
                    if x + y <= (kyori as i32 - 1).max(0) {
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

/// 切り捨て
#[derive(defmt::Format, PartialEq, Eq)]
struct Block {
    /// 5分 * count
    count: u8,
    pub kind: BlockKind,
}

#[derive(defmt::Format, PartialEq, Eq)]
enum BlockKind {
    Rest,
    // typeを記録できるように
    Other(u8),
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let board = Microbit::default();
    let board_not_embassy = Board::take().unwrap();

    defmt::info!("Application started, press buttons!");
    let state = State {
        times: Vec::new(),
        mode: Mode::ModeSelect(0),
        counter: 0,
    };
    state.event_loop(board).await;
}
