use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crabboy_core::gameboy::{Buttons, GameBoy};
use minifb::{Key, Window, WindowOptions};
use ringbuf::traits::*;
use std::time::{Duration, Instant};

use crate::config::Config;

mod config;
// const COLORS: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F]; // DMG original const
// const COLORS: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000]; // Black & white
// const COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F]; // GB Pocket
// const COLORS: [u32; 4] = [0xE8F8E0, 0xB0E018, 0x509000, 0x202850]; // GB Light
// const COLORS: [u32; 4] = [0xFFFFFF, 0x666666, 0x333333, 0x000000]; // High contrast

fn main() {
    //CORE
    let mut gameboy = GameBoy::new();

    let config = Config::load("config.toml");

    //FRONTEND
    //minifb
    let mut window = Window::new(
        "CrabBoy",
        160 * config.scale as usize,
        144 * config.scale as usize,
        WindowOptions::default(),
    )
    .unwrap();

    //cpal
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let output_config = device.default_output_config().unwrap();
    let ring_buffer = ringbuf::HeapRb::<f32>::new(44100 * 2 / 10);
    let (mut prod, mut cons) = ring_buffer.split();

    let stream = device
        .build_output_stream(
            &output_config.into(),
            move |data: &mut [f32], _| {
                for sample in data.iter_mut() {
                    *sample = cons.try_pop().unwrap_or(0.0);
                }
            },
            |err| eprintln!("audio error: {}", err),
            None,
        )
        .unwrap();

    stream.play().unwrap();

    let path = rfd::FileDialog::new()
        .add_filter("GameBoy ROM", &["gb"])
        .pick_file();

    if let Some(path) = path {
        gameboy.load_rom(path.to_string_lossy().to_string());
    } else {
        return;
    }

    let frame_duration = Duration::from_nanos(16_666_667);
    // let frame_duration = Duration::from_nanos(8_333_332);
    // let frame_duration = Duration::from_nanos(512_222_223);
    //
    let mut last_frame = Instant::now();

    loop {
        gameboy.set_button(Buttons::Up, window.is_key_down(Key::W));
        gameboy.set_button(Buttons::Down, window.is_key_down(Key::S));
        gameboy.set_button(Buttons::Left, window.is_key_down(Key::A));
        gameboy.set_button(Buttons::Right, window.is_key_down(Key::D));
        gameboy.set_button(Buttons::A, window.is_key_down(Key::Period));
        gameboy.set_button(Buttons::B, window.is_key_down(Key::Comma));
        gameboy.set_button(Buttons::Select, window.is_key_down(Key::Backspace));
        gameboy.set_button(Buttons::Start, window.is_key_down(Key::Enter));

        if gameboy.step() {
            let video_buffer: Vec<u32> = gameboy
                .framebuffer()
                .iter()
                .map(|&c| config.palette[c as usize])
                .collect();

            window.update_with_buffer(&video_buffer, 160, 144).unwrap();

            let audio_buffer = gameboy.drain_audio();

            for sample in audio_buffer {
                let _ = prod.try_push(sample);
            }

            // frame timing
            let elapsed = last_frame.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
            last_frame = Instant::now();
        }
    }
}
