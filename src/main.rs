use crate::gameboy::GameBoy;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use minifb::{Key, Window, WindowOptions};
use ringbuf::traits::*;
use std::time::{Duration, Instant};

mod apu;
mod cpu;
mod gameboy;
mod hardware;
mod memory;
mod ppu;

// const COLORS: [u32; 4] = [0x9BBC0F, 0x8BAC0F, 0x306230, 0x0F380F]; // DMG original const
// const COLORS: [u32; 4] = [0xFFFFFF, 0xAAAAAA, 0x555555, 0x000000]; // Black & white
const COLORS: [u32; 4] = [0xC4CFA1, 0x8B956D, 0x4D533C, 0x1F1F1F]; // GB Pocket
// const COLORS: [u32; 4] = [0xE8F8E0, 0xB0E018, 0x509000, 0x202850]; // GB Light
// const COLORS: [u32; 4] = [0xFFFFFF, 0x666666, 0x333333, 0x000000]; // High contrast

fn main() {
    //CORE
    let mut gameboy = GameBoy::new();

    //FRONTEND
    //minifb
    let mut window = Window::new("CrabBoy", 160 * 3, 144 * 3, WindowOptions::default()).unwrap();

    //cpal
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().unwrap();
    let ring_buffer = ringbuf::HeapRb::<f32>::new(44100 * 2 / 10);
    let (mut prod, mut cons) = ring_buffer.split();

    let stream = device
        .build_output_stream(
            &config.into(),
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

    let path = "roms/";

    let filename;
    //TEST ROMS
    // filename = "cpu_instrs.gb";
    // filename = "individual/01-special.gb";
    // filename = "individual/02-interrupts.gb";
    // filename = "individual/03-op sp,hl.gb";
    // filename = "individual/05-op rp.gb";
    // filename = "individual/06-ld r,r.gb";
    // filename = "individual/07-jr,jp,call,ret,rst.gb";
    // filename = "individual/08-misc instrs.gb";
    // filename = "individual/09-op r,r.gb";
    // filename = "individual/10-bit ops.gb";
    // filename = "individual/11-op a,(hl).gb";
    // filename = "dmg-acid2.gb";
    //
    //GAMES
    // filename = "games/dr mario.gb";
    // filename = "games/super mario land.gb";
    filename = "games/zelda.gb";
    // filename = "games/wario land.gb";
    // filename = "games/donkey kong 3.gb";
    // filename = "games/metroid 2.gb";
    // filename = "games/pokemon yellow.gb";
    // filename = "games/kirby.gb";
    // filename = "games/mk.gb";
    // filename = "games/st2.gb";
    // filename = "games/contra.gb";

    let full_path = format!("{}{}", path, filename);

    gameboy.load_rom(full_path);

    let frame_duration = Duration::from_nanos(16_666_667);
    // let frame_duration = Duration::from_nanos(8_333_332);
    // let frame_duration = Duration::from_nanos(512_222_223);
    //
    let mut last_frame = Instant::now();

    loop {
        gameboy.set_button(gameboy::Buttons::Up, window.is_key_down(Key::W));
        gameboy.set_button(gameboy::Buttons::Down, window.is_key_down(Key::S));
        gameboy.set_button(gameboy::Buttons::Left, window.is_key_down(Key::A));
        gameboy.set_button(gameboy::Buttons::Right, window.is_key_down(Key::D));
        gameboy.set_button(gameboy::Buttons::A, window.is_key_down(Key::Period));
        gameboy.set_button(gameboy::Buttons::B, window.is_key_down(Key::Comma));
        gameboy.set_button(gameboy::Buttons::Select, window.is_key_down(Key::Backspace));
        gameboy.set_button(gameboy::Buttons::Start, window.is_key_down(Key::Enter));

        if gameboy.step() {
            let video_buffer: Vec<u32> = gameboy
                .framebuffer()
                .iter()
                .map(|&c| COLORS[c as usize])
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
