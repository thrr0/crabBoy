use crabboy_core::gameboy::{Buttons, GameBoy};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct GameBoyWasm {
    inner: GameBoy,
}
#[wasm_bindgen]
impl GameBoyWasm {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameBoyWasm {
        let inner = GameBoy::new();
        GameBoyWasm { inner }
    }
    pub fn load_rom(&mut self, rom: &[u8]) {
        self.inner.load_rom(rom.to_vec());
    }

    pub fn load_save(&mut self, save: &[u8]) {
        self.inner.load_save(save.to_vec());
    }

    pub fn step(&mut self) -> bool {
        self.inner.step()
    }

    pub fn framebuffer(&mut self) -> Vec<u8> {
        self.inner.framebuffer().to_vec()
    }

    pub fn drain_audio(&mut self) -> Vec<f32> {
        self.inner.drain_audio()
    }

    pub fn set_button(&mut self, button: u8, is_pressed: bool) {
        let b = match button {
            1 => Buttons::Up,
            2 => Buttons::Down,
            3 => Buttons::Left,
            4 => Buttons::Right,
            5 => Buttons::A,
            6 => Buttons::B,
            7 => Buttons::Select,
            8 => Buttons::Start,
            _ => unreachable!(),
        };

        self.inner.set_button(b, is_pressed);
    }

    pub fn is_save_dirty(&self) -> bool {
        self.inner.is_save_dirty()
    }

    pub fn get_save_data(&mut self) -> Vec<u8> {
        self.inner.get_save_data()
    }

    pub fn clear_save_dirty(&mut self) {
        self.inner.clear_save_dirty();
    }
}
