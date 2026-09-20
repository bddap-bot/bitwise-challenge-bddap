pub mod cheeky_encoding;
pub mod game;
pub mod longsnake;

#[cfg(all(feature = "desktop", not(target_arch = "wasm32")))]
mod desktop;

#[cfg(target_arch = "wasm32")]
mod web;
