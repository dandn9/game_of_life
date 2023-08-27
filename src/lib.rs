pub mod game_of_life;
use wasm_bindgen::prelude::*;

// This exists for wasm only
#[wasm_bindgen]
pub fn main() {
    game_of_life::main();
}
