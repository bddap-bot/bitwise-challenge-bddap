# Longsnake

Eat green fruit, grow your tail, and avoid running into it. Use WASD or arrow keys.
The desktop game also accepts a gamepad d-pad or left stick. Escape closes the
desktop window; after a collision the game restarts automatically.

## Desktop

Install stable Rust. Linux also needs pkg-config, the X11/Wayland development
libraries used by minifb, and libudev development files for gilrs.

```sh
cargo run --release --bin longsnake
```

With Nix, enter `nix-shell` first.

## Browser

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked --release --lib --no-default-features --target wasm32-unknown-unknown
cp "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/bitwise_challenge_bddap.wasm" web/longsnake.wasm
python3 -m http.server --directory web 8000
```

Open <http://localhost:8000>. Both targets use the same state, direction queue,
frame step, and pixel buffer. The browser presents that buffer on a canvas at
60 simulation steps per second. Desktop dependencies are optional and excluded
from the wasm build.

## Checks and deployment

```sh
cargo fmt --check
cargo test --locked
cargo test --locked --no-default-features
cargo build --locked --release --bin longsnake
cargo build --locked --release --lib --no-default-features --target wasm32-unknown-unknown
```

Choose GitHub Actions as the repository's Pages source. The Pages workflow builds
and tests both targets, then deploys on pushes to main or the demo branch.
Pull requests build and test without deploying.
