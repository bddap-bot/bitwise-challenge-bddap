use std::cell::RefCell;

use crate::{
    game::{Direction, Game, Runner},
    longsnake::Snake,
};

thread_local! {
    static GAME: RefCell<Runner<Snake>> = RefCell::new(Runner::default());
    static INPUT: RefCell<Vec<Direction>> = const { RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C" fn width() -> u32 {
    Snake::WIDTH as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn height() -> u32 {
    Snake::HEIGHT as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn direction(value: u32) {
    if value < 4 {
        INPUT.with_borrow_mut(|input| {
            input.push(Direction::from(value as u8));
        });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn frame(elapsed_ms: f64) -> *const u32 {
    GAME.with_borrow_mut(|game| {
        INPUT.with_borrow_mut(|input| {
            let buffer = game.frame(elapsed_ms, input).as_ptr();
            input.clear();
            buffer
        })
    })
}
