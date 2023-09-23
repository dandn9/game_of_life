pub mod game_of_life;
// mod game_of_life_plugin;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// This exists for wasm only
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn main() {
    game_of_life::main();
}
