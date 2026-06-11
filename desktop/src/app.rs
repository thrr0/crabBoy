use crate::config::Config;
use crabboy_core::gameboy::GameBoy;
use ringbuf::traits::Producer;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

pub struct App {
    pub gameboy: Option<GameBoy>,
    config: Config,
    show_config: bool,
    texture: Option<egui::TextureHandle>,
    prod: ringbuf::HeapProd<f32>,
    rom_path: Option<PathBuf>,
    recent_roms: Vec<String>,
    last_frame: Instant,
}

impl App {
    pub fn new(config: Config, prod: ringbuf::HeapProd<f32>) -> App {
        App {
            gameboy: None,
            config,
            show_config: true,
            texture: None,
            prod,
            rom_path: None,
            recent_roms: Vec::new(),
            last_frame: Instant::now(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let palette = self.config.palette;

        if let Some(gameboy) = &mut self.gameboy {
            let frame_duration = Duration::from_nanos(16_666_667);
            let elapsed = self.last_frame.elapsed();

            if elapsed > frame_duration {
                while !gameboy.step() {}
                self.last_frame = Instant::now();
            }

            ctx.request_repaint();

            let pixels: Vec<egui::Color32> = gameboy
                .framebuffer()
                .iter()
                .map(|&c| {
                    let color = palette[c as usize];
                    let r = ((color >> 16) & 0b11111111) as u8;
                    let g = ((color >> 8) & 0b11111111) as u8;
                    let b = (color & 0b11111111) as u8;
                    egui::Color32::from_rgb(r, g, b)
                })
                .collect();

            let image = egui::ColorImage {
                size: [160, 144],
                pixels,
            };

            self.texture = Some(ctx.load_texture("screen", image, egui::TextureOptions::NEAREST));

            let audio_buffer = gameboy.drain_audio();
            for sample in audio_buffer {
                let _ = self.prod.try_push(sample);
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("CrabBoy");
            if let Some(texture) = &self.texture {
                ui.image(egui::load::SizedTexture::new(texture.id(), [480.0, 432.0]));
            } else {
                if ui.button("Load ROM").clicked() {
                    self.rom_path = rfd::FileDialog::new()
                        .add_filter("GameBoy ROM", &["gb"])
                        .pick_file();

                    if let Some(path) = &self.rom_path {
                        let mut gb = GameBoy::new();
                        gb.load_rom(path.to_string_lossy().to_string());
                        self.gameboy = Some(gb);
                    }
                } else {
                }
            }
        });
    }
}
