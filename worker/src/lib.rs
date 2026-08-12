#![cfg(target_arch = "wasm32")]

mod render;

use nightshade_api::offscreen::OffscreenConfig;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    nightshade_api::offscreen::run_offscreen(
        OffscreenConfig::default(),
        render::Board::default(),
        render::initialize,
        render::tick,
        render::apply_custom,
    );
}
