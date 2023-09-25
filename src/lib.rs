#[cfg(target_arch = "wasm32")]
mod game_of_life;
#[cfg(target_arch = "wasm32")]
mod ui;
// mod game_of_life_ui;
// mod game_of_life_plugin;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// This exists for wasm only
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main() {
    game_of_life::init();
}
