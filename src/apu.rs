use crate::memory::MemoryBus;

pub struct APU {
    active: bool,
    divider: u64, // cpu works at 4Mhz and sample rate is 44.1Khz (4MHz / 44.1KHz = ~95)
    l_master: u8,
    r_master: u8,
    pub buffer: Vec<f32>,
    channel_1: SquareChannel,
    channel_2: SquareChannel,
    channel_3: WaveChannel,
    channel_4: NoiseChannel,
    channel_routing: ChannelRouting,
}

impl APU {
    pub fn new() -> APU {
        APU {
            active: false,
            divider: 0,
            l_master: 0,
            r_master: 0,
            buffer: Vec::new(),
            channel_1: SquareChannel {
                active: false,
                timer: 0,
                duty: 0,
                frequency: 0,
                duty_pos: 0,
                length_timer: 0,
                sweep: Some(SweepState {
                    sweep_timer: 0,
                    sweep_direction: false,
                    sweep_shift: 0,
                    sweep_frec: 0,
                }),
                envelope: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                },
            },
            channel_2: SquareChannel {
                active: false,
                timer: 0,
                duty: 0,
                frequency: 0,
                duty_pos: 0,
                length_timer: 0,
                sweep: None,
                envelope: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                },
            },
            channel_3: WaveChannel {
                active: false,
                timer: 0,
                sample_pos: 0,
                volume: 0,
                length_timer: 0,
            },
            channel_4: NoiseChannel {
                active: false,
                timer: 0,
                lfsr: 0x7FFF,
                short_mode: false,
                length_timer: 0,
                env: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                },
            },
            channel_routing: ChannelRouting {
                channel_1: Panning {
                    left: false,
                    right: false,
                },
                channel_2: Panning {
                    left: false,
                    right: false,
                },
                channel_3: Panning {
                    left: false,
                    right: false,
                },
                channel_4: Panning {
                    left: false,
                    right: false,
                },
            },
        }
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus, cycles: u32) {
        let nr52 = memory_bus.read(0xFF26); // audio master control

        //bit 7: audio on/off
        if nr52 & 0x80 == 0 {
            return;
        }

        self.update_channel_registers(memory_bus);

        self.update_timers(cycles);

        if self.divider >= 95 {
            self.generate_sample();
            self.divider -= 95;
        }
    }

    fn update_timers(&mut self, cycles: u32) {
        self.divider += cycles as u64;

        if self.channel_1.active {
            self.channel_1.timer = self.channel_1.timer.wrapping_sub(cycles);
        }
        if self.channel_2.active {
            self.channel_2.tick(cycles);
        }
        if self.channel_3.active {
            self.channel_3.timer = self.channel_3.timer.wrapping_sub(cycles);
        }
        if self.channel_4.active {
            self.channel_4.timer = self.channel_4.timer.wrapping_sub(cycles);
        }
    }

    fn update_channel_registers(&mut self, memory_bus: &mut MemoryBus) {
        self.update_master_volume(memory_bus);

        self.update_channel_2(memory_bus);
        self.update_channel_routing(memory_bus);
    }

    fn update_master_volume(&mut self, memory_bus: &MemoryBus) {
        let nr50 = memory_bus.read(0xFF24);

        self.l_master = (nr50 & 0b01110000) >> 4;
        self.r_master = nr50 & 0b00000111;
    }

    fn update_channel_2(&mut self, memory_bus: &mut MemoryBus) {
        //          7 6         5 4 3 2 1 0
        //nr21      wave duty   initial length timer
        let nr21 = memory_bus.read(0xFF16);
        //nr23 = low 8 bits of channel 2 frequency
        let nr23 = memory_bus.read(0xFF18);
        //          7           6            5 4 3     2 1 0
        //nr24  trigger     Length enable   -------     high 3 bits of ch2 freq
        let nr24 = memory_bus.read(0xFF19);

        self.channel_2.frequency = (nr24 as u16 & 0b00000111) << 8 | nr23 as u16;

        let wave_duty = (nr21 >> 6) & 0x03;
        // Value        Duty cycle
        //  0b00          12.5%
        //  0b01          25%
        //  0b10          50%
        //  0b11          75%
        self.channel_2.duty = match wave_duty {
            0b00 => 0b00000001,
            0b01 => 0b10000001,
            0b10 => 0b10000111,
            0b11 => 0b01111111,
            _ => unreachable!(),
        };

        if nr24 & 0b10000000 != 0 {
            self.trigger_channel_2(memory_bus);
            //game only triggers channel once
            memory_bus.write(0xFF19, nr24 & !(1 << 7));
            // eprintln!("channel 2 env vol: {}", self.channel_2.envelope.volume);
        }
    }

    fn trigger_channel_2(&mut self, memory_bus: &MemoryBus) {
        //          7 6 5 4          3          2 1 0
        //nr22    initial volume    env dir    sweep race
        let nr22 = memory_bus.read(0xFF17);
        // eprintln!("nr22 = {}", nr22);

        self.channel_2.active = true;

        self.channel_2.envelope.volume = (nr22 >> 4) & 0b00001111;

        self.channel_2.envelope.env_direction = nr22 & 0b00001000 != 0;

        self.channel_2.envelope.env_timer = nr22 & 0b00000111;

        self.channel_2.timer = (2048 - self.channel_2.frequency as u32) * 4;

        self.channel_2.duty_pos = 0;
    }

    fn update_channel_routing(&mut self, memory_bus: &MemoryBus) {
        // NR51(0xFF25) controls which channels are active on each speaker
        // bit 7: ch4 -> left channel
        // bit 6: ch3 -> left channel
        // bit 5: ch2 -> left channel
        // bit 4: ch1 -> left channel
        //
        // bit 3: ch4 -> right channel
        // bit 2: ch3 -> right channel
        // bit 1: ch2 -> right channel
        // bit 0: ch1 -> right channel
        let nr51 = memory_bus.read(0xFF25);

        self.channel_routing.channel_4.left = nr51 & (1 << 7) != 0;
        self.channel_routing.channel_3.left = nr51 & (1 << 6) != 0;
        self.channel_routing.channel_2.left = nr51 & (1 << 5) != 0;
        self.channel_routing.channel_1.left = nr51 & (1 << 4) != 0;
        self.channel_routing.channel_4.right = nr51 & (1 << 3) != 0;
        self.channel_routing.channel_3.right = nr51 & (1 << 2) != 0;
        self.channel_routing.channel_2.right = nr51 & (1 << 1) != 0;
        self.channel_routing.channel_1.right = nr51 & (1 << 0) != 0;
    }

    fn generate_sample(&mut self) {
        // eprintln!("ch2 active: {}", self.channel_2.active);
        // eprintln!("ch2 frequency: {}", self.channel_2.frequency);
        let ch2_output: u8 = if self.channel_2.active {
            self.channel_2.output()
        } else {
            0
        };

        let ch2_left: u16 = if self.channel_routing.channel_2.left {
            ch2_output as u16 * (self.l_master as u16 + 1)
        } else {
            0
        };
        let ch2_right: u16 = if self.channel_routing.channel_2.right {
            ch2_output as u16 * (self.r_master as u16 + 1)
        } else {
            0
        };

        let (l_sample, r_sample) = (ch2_left as f32 / 120.0, ch2_right as f32 / 120.0);

        // eprintln!("l_sample: {}", l_sample);
        // eprintln!("r_sample: {}", r_sample);
        self.buffer.push(l_sample);
        self.buffer.push(r_sample);
    }
}

struct SquareChannel {
    active: bool,
    timer: u32,
    frequency: u16,
    duty: u8,
    duty_pos: u8,
    length_timer: u8,
    sweep: Option<SweepState>,
    envelope: EnvelopeState,
}

impl SquareChannel {
    fn tick(&mut self, cycles: u32) {
        self.timer = self.timer.saturating_sub(cycles);

        if self.timer == 0 {
            self.duty_pos = (self.duty_pos + 1) % 8; //wraps at 8

            self.timer += (2048 - self.frequency as u32) * 4;
        }
    }

    fn output(&self) -> u8 {
        // eprintln!("env volume: {}", self.envelope.volume);
        // eprintln!("ch2 duty: {}", self.duty);
        // eprintln!("ch2 duty_pos: {}", self.duty_pos);
        if (self.duty >> self.duty_pos) & 1 != 0 {
            self.envelope.volume
        } else {
            0
        }
    }
}

struct WaveChannel {
    active: bool,
    timer: u32,
    sample_pos: u8,
    volume: u8, // 1, 2, 4
    length_timer: u8,
}

struct NoiseChannel {
    active: bool,
    timer: u32,
    lfsr: u16,
    short_mode: bool, //whether lfsr uses 7 or 15 bits
    length_timer: u8,
    env: EnvelopeState,
}

struct EnvelopeState {
    volume: u8,
    env_timer: u8,
    env_direction: bool, // 1= up; 0 =down
}
struct SweepState {
    sweep_timer: u16,
    sweep_direction: bool,
    sweep_shift: u8,
    sweep_frec: u16,
}

struct ChannelRouting {
    channel_1: Panning,
    channel_2: Panning,
    channel_3: Panning,
    channel_4: Panning,
}

struct Panning {
    left: bool,
    right: bool,
}
