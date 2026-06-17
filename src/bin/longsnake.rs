// Thin native entry point. The game itself lives in the library
// (`bitwise_challenge_bddap::longsnake`) so wasm-bindgen can export it from the
// cdylib; this binary just runs it on the native (minifb) backend.
use bitwise_challenge_bddap::game::Game;
use bitwise_challenge_bddap::longsnake::Snake;

fn main() {
    Snake::run();
}
