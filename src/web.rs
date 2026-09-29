use std::cell::RefCell;

use crate::{
    game::{Direction, Game, Runner},
    longsnake::Snake,
};

thread_local! {
    static GAME: RefCell<Runner<Snake>> = RefCell::new(Runner::default());
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
    GAME.with_borrow_mut(|game| game.push(Direction::from(value as u8)));
}

#[unsafe(no_mangle)]
pub extern "C" fn step() -> *const u32 {
    GAME.with_borrow_mut(|game| game.step().as_ptr())
}
