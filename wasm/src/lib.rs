use chip8_core::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct EmuWasm {
    chip8: Emu,
}

#[wasm_bindgen]
impl EmuWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> EmuWasm {
        EmuWasm { chip8: Emu::new() }
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8 = Emu::new();
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, data: &[u8]) {
        self.chip8.load(data);
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) {
        self.chip8.tick_timers();
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.chip8.keypress(idx, pressed);
    }

    /// Returns the display as a flat Uint8Array of SCREEN_WIDTH * SCREEN_HEIGHT bytes (0 or 1).
    #[wasm_bindgen]
    pub fn get_display(&self) -> Vec<u8> {
        self.chip8.get_display().to_vec()
    }

    #[wasm_bindgen]
    pub fn screen_width() -> usize {
        SCREEN_WIDTH
    }

    #[wasm_bindgen]
    pub fn screen_height() -> usize {
        SCREEN_HEIGHT
    }
}
