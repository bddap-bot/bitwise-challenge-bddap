// Copied from https://github.com/zesterer/bitwise-examples/blob/b8479bfd485356b3ac399f984e713939ba6a6573/src/lib.rs
// and modified to support input keypress events instead of just is_key_down

use gilrs::{Axis, Button, EventType, Gilrs};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::{marker::PhantomData, time::Duration};

/// A device-agnostic directional press. Keyboard arrows, gamepad d-pad, and the
/// left stick all funnel through this so game logic never sees the input device.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

// WASD as well as arrows: a non-Steam shortcut's default Steam Input template
// often sends the stick/d-pad as keyboard keys rather than a gamepad, so
// covering both keeps the Deck responsive whatever template is active.
fn key_dir(key: Key) -> Option<Dir> {
    match key {
        Key::Up | Key::W => Some(Dir::Up),
        Key::Down | Key::S => Some(Dir::Down),
        Key::Left | Key::A => Some(Dir::Left),
        Key::Right | Key::D => Some(Dir::Right),
        _ => None,
    }
}

fn button_dir(button: Button) -> Option<Dir> {
    match button {
        Button::DPadUp => Some(Dir::Up),
        Button::DPadDown => Some(Dir::Down),
        Button::DPadLeft => Some(Dir::Left),
        Button::DPadRight => Some(Dir::Right),
        _ => None,
    }
}

/// Reads gamepads and turns them into edge-triggered directional presses, so a
/// held d-pad or stick fires once per push — matching how keypress events behave.
struct Pads {
    // `None` if no gamepad backend is available; the game stays keyboard-only.
    gilrs: Option<Gilrs>,
    // The stick's last cardinal zone, so we only emit on entering a new one.
    stick_dir: Option<Dir>,
}

impl Pads {
    fn new() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
            stick_dir: None,
        }
    }

    /// Directions newly pressed this frame: d-pad edges from any pad, plus the
    /// first pad's left stick.
    fn poll(&mut self) -> Vec<Dir> {
        let Some(gilrs) = self.gilrs.as_mut() else {
            return Vec::new();
        };

        let mut dirs = Vec::new();
        // Draining events both yields d-pad edges and refreshes the axis cache
        // that the stick read below relies on.
        while let Some(event) = gilrs.next_event() {
            if let EventType::ButtonPressed(button, _) = event.event {
                dirs.extend(button_dir(button));
            }
        }

        let stick = stick_dir(gilrs);
        if stick != self.stick_dir {
            dirs.extend(stick);
            self.stick_dir = stick;
        }
        dirs
    }
}

/// The dominant cardinal direction the first gamepad's left stick is pushed, or
/// `None` if it's inside the deadzone. One stick is enough for a one-player game,
/// and reading a single pad keeps the edge-detection zone unambiguous. gilrs
/// normalizes up and right to positive.
fn stick_dir(gilrs: &Gilrs) -> Option<Dir> {
    const DEADZONE: f32 = 0.5;
    let (_id, pad) = gilrs.gamepads().next()?;
    let x = pad.value(Axis::LeftStickX);
    let y = pad.value(Axis::LeftStickY);
    if x.abs().max(y.abs()) < DEADZONE {
        return None;
    }
    Some(if x.abs() >= y.abs() {
        if x > 0.0 { Dir::Right } else { Dir::Left }
    } else if y > 0.0 {
        Dir::Up
    } else {
        Dir::Down
    })
}

pub trait Game: Sized + 'static {
    const NAME: &'static str;
    const WIDTH: usize;
    const HEIGHT: usize;

    fn init() -> u64;

    fn tick(prev: u64, input: &Input<'_, Self>, output: &mut Output<'_, Self>) -> u64;

    fn run() -> ! {
        let mut win = Window::new(
            Self::NAME,
            Self::WIDTH,
            Self::HEIGHT,
            WindowOptions::default(),
        )
        .unwrap();

        win.limit_update_rate(Some(Duration::from_micros(16600)));

        let mut state = Self::init();
        let mut pads = Pads::new();

        let mut tick = 0;
        while win.is_open() && !win.is_key_down(Key::Escape) {
            let mut buf = vec![0; Self::WIDTH * Self::HEIGHT];

            let input = Input {
                win: &win,
                tick,
                pad_dirs: pads.poll(),
                phantom: PhantomData,
            };
            let mut output = Output::new();

            state = Self::tick(state, &input, &mut output);

            output.write_to(&mut buf);

            win.update_with_buffer(&buf, Self::WIDTH, Self::HEIGHT)
                .unwrap();

            tick += 1;
        }

        std::process::exit(0)
    }
}

pub struct Input<'a, G: Game> {
    win: &'a Window,
    tick: u64,
    pad_dirs: Vec<Dir>,
    phantom: PhantomData<&'static mut G>,
}

impl<'a, G: Game> Input<'a, G> {
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Directional presses this frame, merged across keyboard arrows and any
    /// gamepad. Edge-triggered: one event per push, not per frame held.
    pub fn get_directions_pressed(&self) -> impl Iterator<Item = Dir> + '_ {
        self.win
            .get_keys_pressed(KeyRepeat::No)
            .into_iter()
            .filter_map(key_dir)
            .chain(self.pad_dirs.iter().copied())
    }
}

pub struct Output<'a, G: Game> {
    shapes: Vec<Shape>,
    phantom: PhantomData<&'a mut G>,
}

enum Shape {
    Rect {
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        color: [u8; 3],
    },
}

impl<'a, G: Game> Output<'a, G> {
    pub fn rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: [u8; 3]) {
        self.shapes.push(Shape::Rect { x, y, w, h, color });
    }

    fn new() -> Self {
        Self {
            shapes: Vec::new(),
            phantom: PhantomData,
        }
    }

    fn write_to(self, buf: &mut Vec<u32>) {
        for shape in self.shapes {
            match shape {
                Shape::Rect { x, y, w, h, color } => {
                    for j in 0..h {
                        for i in 0..w {
                            let pos = [x + i as i32, y + j as i32];
                            if pos[0] > 0
                                && pos[0] < G::WIDTH as i32
                                && pos[1] > 0
                                && pos[1] < G::HEIGHT as i32
                            {
                                buf[pos[1] as usize * G::WIDTH + pos[0] as usize] =
                                    u32::from_le_bytes([color[0], color[1], color[2], 255]);
                            }
                        }
                    }
                }
            }
        }
    }
}
