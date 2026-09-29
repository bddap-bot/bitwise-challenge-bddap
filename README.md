# Longsnake

A snake game whose entire state fits in one `u64`. Eat the green fruit to grow
and don't run into your tail; the board wraps at its edges, and a crash restarts
the game.

Steer with WASD or the arrow keys. The desktop build also takes a gamepad d-pad
or left stick; Escape closes it.

## Desktop

```sh
cargo run --release
```

On Linux this needs pkg-config and the libudev and libxkbcommon development
files; `nix-shell` provides them.

## Browser

```sh
rustup target add wasm32-unknown-unknown
cargo build --release --lib --target wasm32-unknown-unknown
cp "${CARGO_TARGET_DIR:-target}/wasm32-unknown-unknown/release/bitwise_challenge_bddap.wasm" web/longsnake.wasm
python3 -m http.server --directory web 8000
```

Then open <http://localhost:8000>.

## Deployment

The Pages workflow checks formatting, runs the tests and builds the browser
version for every pull request and every push to `main`. Pushes to `main` also
publish `web/` to GitHub Pages; set the repository's Pages source to GitHub
Actions.
