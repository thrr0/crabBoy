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
    div_apu: u8,
    last_div: u8,
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
                length_enable: false,
                sweep: Some(SweepState {
                    sweep_timer: 0,
                    sweep_direction: false,
                    sweep_shift: 0,
                    sweep_frec: 0,
                    sweep_pace: 0,
                }),
                envelope: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                    env_period: 0,
                },
            },
            channel_2: SquareChannel {
                active: false,
                timer: 0,
                duty: 0,
                frequency: 0,
                duty_pos: 0,
                length_timer: 0,
                length_enable: false,
                sweep: None,
                envelope: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                    env_period: 0,
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
                timer_period: 0,
                lfsr: 0x7FFF,
                short_mode: false,
                length_timer: 0,
                length_enable: false,
                envelope: EnvelopeState {
                    volume: 0,
                    env_timer: 0,
                    env_direction: false,
                    env_period: 0,
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
            div_apu: 0,
            last_div: 0,
        }
    }

    pub fn step(&mut self, memory_bus: &mut MemoryBus, cycles: u32) {
        let nr52 = memory_bus.read(0xFF26); // audio master control

        let mut ticked = false;
        //div-apu is increased every time DIV's bit goes from 0 to 1.
        if (self.last_div & 0b00010000) != 0 && (memory_bus.read(0xFF04) & 0b00010000) == 0 {
            self.div_apu = self.div_apu.wrapping_add(1) % 8;
            ticked = true;
        }

        self.last_div = memory_bus.read(0xFF04);

        //bit 7: audio on/off
        if nr52 & 0x80 == 0 {
            return;
        }

        if ticked {
            self.div_apu_events();
        }

        self.update_channel_registers(memory_bus);

        self.update_timers(cycles);

        if self.divider >= 95 {
            self.generate_sample();
            self.divider -= 95;
        }
    }

    fn div_apu_events(&mut self) {
        match self.div_apu {
            0 | 4 => {
                tick_length_timers(&mut self.channel_1);
                tick_length_timers(&mut self.channel_2);
            }
            2 | 6 => {
                tick_length_timers(&mut self.channel_1);
                tick_length_timers(&mut self.channel_2);
                tick_sweep(&mut self.channel_1);
            }
            7 => {
                tick_envelopes(&mut self.channel_1.envelope);
                tick_envelopes(&mut self.channel_2.envelope);
                tick_envelopes(&mut self.channel_4.envelope);
            }
            _ => {}
        }
    }

    fn update_timers(&mut self, cycles: u32) {
        self.divider += cycles as u64;

        if self.channel_1.active {
            self.channel_1.tick(cycles);
        }
        if self.channel_2.active {
            self.channel_2.tick(cycles);
        }
        if self.channel_3.active {
            self.channel_3.timer = self.channel_3.timer.wrapping_sub(cycles);
        }
        if self.channel_4.active {
            self.channel_4.tick(cycles);
        }
    }

    fn update_channel_registers(&mut self, memory_bus: &mut MemoryBus) {
        self.update_master_volume(memory_bus);

        self.update_channel_1(memory_bus);
        self.update_channel_2(memory_bus);
        self.update_channel_4(memory_bus);
        self.update_channel_routing(memory_bus);
    }

    fn update_master_volume(&mut self, memory_bus: &MemoryBus) {
        let nr50 = memory_bus.read(0xFF24);

        self.l_master = (nr50 & 0b01110000) >> 4;
        self.r_master = nr50 & 0b00000111;
    }

    fn update_channel_1(&mut self, memory_bus: &mut MemoryBus) {
        //          SWEEP
        //          7     6 5 4    3     2 1 0
        //nr10      ~      pace   dir    Individual setp
        let nr10 = memory_bus.read(0xFF10);
        //          LENGTH TIMER & DUTY CYCLE
        //          7 6         5 4 3 2 1 0
        //nr11      wave duty   initial length timer
        let nr11 = memory_bus.read(0xFF11);
        //          VOLUME & ENVELOPE
        //          7 6 5 4          3          2 1 0
        //nr12    initial volume    env dir    sweep race
        let nr12 = memory_bus.read(0xFF12);

        //nr13 = low 8 bits of channel 2 frequency
        let nr13 = memory_bus.read(0xFF13);
        //          7           6            5 4 3     2 1 0
        //nr14  trigger     Length enable   -------     high 3 bits of ch2 freq
        let nr14 = memory_bus.read(0xFF14);

        update_square_channel(&mut self.channel_1, nr11, nr13, nr14);

        if nr14 & 0b10000000 != 0 {
            self.trigger_channel_1(nr12);
            //game only triggers channel once
            memory_bus.write(0xFF14, nr14 & !(1 << 7));
            // eprintln!("channel 2 env vol: {}", self.channel_2.envelope.volume);
        }
        if let Some(sweep) = &mut self.channel_1.sweep {
            update_sweep(sweep, nr10);
        }
    }

    fn update_channel_2(&mut self, memory_bus: &mut MemoryBus) {
        //          7 6         5 4 3 2 1 0
        //nr21      wave duty   initial length timer
        let nr21 = memory_bus.read(0xFF16);
        //          7 6 5 4          3          2 1 0
        //nr22    initial volume    env dir    sweep race
        let nr22 = memory_bus.read(0xFF17);

        //nr23 = low 8 bits of channel 2 frequency
        let nr23 = memory_bus.read(0xFF18);
        //          7           6            5 4 3     2 1 0
        //nr24  trigger     Length enable   -------     high 3 bits of ch2 freq
        let nr24 = memory_bus.read(0xFF19);
        update_square_channel(&mut self.channel_2, nr21, nr23, nr24);

        if nr24 & 0b10000000 != 0 {
            self.trigger_channel_2(nr22);
            //game only triggers channel once
            memory_bus.write(0xFF19, nr24 & !(1 << 7));
            // eprintln!("channel 2 env vol: {}", self.channel_2.envelope.volume);
        }
    }

    fn update_channel_4(&mut self, memory_bus: &mut MemoryBus) {
        let nr41 = memory_bus.read(0xFF20);
        let nr42 = memory_bus.read(0xFF21);
        let nr43 = memory_bus.read(0xFF22);
        let nr44 = memory_bus.read(0xFF23);

        self.channel_4.length_timer = nr41 & 0b00111111;
        self.channel_4.length_enable = nr44 & 0b01000000 != 0;

        if nr44 & 0b10000000 != 0 {
            self.trigger_channel_4(nr42, nr43);
            memory_bus.write(0xFF23, nr44 & !(1 << 7));
        }
    }

    fn trigger_channel_1(&mut self, nr2: u8) {
        // eprintln!("nr22 = {}", nr22);

        let frequency = self.channel_1.frequency;
        if let Some(sweep) = &mut self.channel_1.sweep {
            sweep.sweep_frec = frequency;
        }
        trigger_square_channel(&mut self.channel_1, nr2);
    }

    fn trigger_channel_2(&mut self, nr2: u8) {
        // eprintln!("nr22 = {}", nr22);

        trigger_square_channel(&mut self.channel_2, nr2);

        // eprint!("ch2 env timer: {}", self.channel_2.envelope.env_timer);
        // eprint!("ch2 env period: {}", self.channel_2.envelope.env_period);
    }

    fn trigger_channel_4(&mut self, nr2: u8, nr3: u8) {
        let r = nr3 & 0b00000111;
        let s = (nr3 & 0b11110000) >> 4;
        let divisor: u32 = if r == 0 { 8 } else { r as u32 * 16 };
        self.channel_4.active = true;

        self.channel_4.envelope.volume = (nr2 >> 4) & 0b00001111;

        self.channel_4.envelope.env_direction = nr2 & 0b00001000 != 0;

        self.channel_4.envelope.env_timer = nr2 & 0b00000111;
        self.channel_4.envelope.env_period = nr2 & 0b00000111;
        self.channel_4.timer = divisor << s;
        self.channel_4.timer_period = divisor << s;

        self.channel_4.lfsr = 0x7FFF;
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
        let ch1_output: u8 = if self.channel_1.active {
            self.channel_1.output()
        } else {
            0
        };
        let ch2_output: u8 = if self.channel_2.active {
            self.channel_2.output()
        } else {
            0
        };

        let ch4_output: u8 = if self.channel_4.active {
            self.channel_4.output()
        } else {
            0
        };

        let ch1_left: u16 = if self.channel_routing.channel_1.left {
            ch1_output as u16 * (self.l_master as u16 + 1)
        } else {
            0
        };
        let ch1_right: u16 = if self.channel_routing.channel_1.right {
            ch1_output as u16 * (self.r_master as u16 + 1)
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

        let ch4_left: u16 = if self.channel_routing.channel_4.left {
            ch4_output as u16 * (self.l_master as u16 + 1)
        } else {
            0
        };
        let ch4_right: u16 = if self.channel_routing.channel_4.right {
            ch4_output as u16 * (self.r_master as u16 + 1)
        } else {
            0
        };

        let (l_sample, r_sample) = (
            (ch1_left as f32 + ch2_left as f32 + ch4_left as f32) / 360.0,
            (ch1_right as f32 + ch2_right as f32 + ch4_right as f32) / 360.0,
        );

        // eprintln!("l_sample: {}", l_sample);
        // eprintln!("r_sample: {}", r_sample);
        self.buffer.push(l_sample);
        self.buffer.push(r_sample);
    }
}

fn trigger_square_channel(channel: &mut SquareChannel, nr2: u8) {
    channel.active = true;

    channel.envelope.volume = (nr2 >> 4) & 0b00001111;

    channel.envelope.env_direction = nr2 & 0b00001000 != 0;

    channel.envelope.env_timer = nr2 & 0b00000111;
    channel.envelope.env_period = nr2 & 0b00000111;
    channel.timer = (2048 - channel.frequency as u32) * 4;

    channel.duty_pos = 0;
    // eprintln!("length_enable: {}", channel.length_enable);
}

fn update_square_channel(channel: &mut SquareChannel, nr1: u8, nr3: u8, nr4: u8) {
    channel.frequency = (nr4 as u16 & 0b00000111) << 8 | nr3 as u16;
    channel.length_enable = (nr4 & 0b01000000) != 0;

    let wave_duty = (nr1 >> 6) & 0b00000011;
    // Value        Duty cycle
    //  0b00          12.5%
    //  0b01          25%
    //  0b10          50%
    //  0b11          75%
    channel.duty = match wave_duty {
        0b00 => 0b00000001,
        0b01 => 0b10000001,
        0b10 => 0b10000111,
        0b11 => 0b01111111,
        _ => unreachable!(),
    };
}

fn update_sweep(sweep: &mut SweepState, nr10: u8) {
    sweep.sweep_timer = (nr10 as u16 & 0b01110000) >> 4;
    sweep.sweep_direction = (nr10 & 0b00001000) != 0;
    sweep.sweep_shift = nr10 & 0b00000111;
    sweep.sweep_pace = (nr10 & 0b01110000) >> 4;
}

fn tick_envelopes(envelope: &mut EnvelopeState) {
    if envelope.env_period == 0 {
        return;
    }
    if envelope.env_timer > 0 {
        envelope.env_timer = envelope.env_timer.wrapping_sub(1);
    }
    if envelope.env_timer == 0 {
        envelope.env_timer = envelope.env_period;

        envelope.volume = if envelope.env_direction {
            envelope.volume.saturating_add(1).min(15)
        } else {
            envelope.volume.saturating_sub(1)
        }
    }
}

fn tick_length_timers(channel: &mut SquareChannel) {
    if channel.length_enable {
        channel.length_timer = channel.length_timer.saturating_sub(1);

        if channel.length_timer == 0 {
            channel.active = false;
        }
    }
}

fn tick_sweep(channel_1: &mut SquareChannel) {
    let new_freq = {
        if let Some(sweep) = &mut channel_1.sweep {
            match sweep.sweep_timer {
                0 => sweep.sweep_timer = sweep.sweep_pace as u16,
                _ => sweep.sweep_timer = sweep.sweep_timer.saturating_sub(1),
            };

            if sweep.sweep_shift > 0 {
                let new_freq = if sweep.sweep_direction {
                    sweep.sweep_frec - (sweep.sweep_frec >> sweep.sweep_shift)
                } else {
                    sweep.sweep_frec + (sweep.sweep_frec >> sweep.sweep_shift)
                };

                if new_freq < 2047 {
                    sweep.sweep_frec = new_freq;
                }

                Some(new_freq)
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(freq) = new_freq {
        if freq > 2047 {
            channel_1.active = false;
        } else {
            channel_1.frequency = freq;
        }
    }
}

struct SquareChannel {
    active: bool,
    timer: u32,
    frequency: u16,
    duty: u8,
    duty_pos: u8,
    length_timer: u8,
    length_enable: bool,
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
    timer_period: u32,
    lfsr: u16,
    short_mode: bool, //whether lfsr uses 7 or 15 bits
    length_timer: u8,
    length_enable: bool,
    envelope: EnvelopeState,
}
impl NoiseChannel {
    fn tick(&mut self, cycles: u32) {
        self.timer = self.timer.saturating_sub(cycles);

        if self.timer == 0 {
            let cur_lfsr = self.lfsr;
            let xor = (cur_lfsr & 0b000000001) ^ (cur_lfsr & 0b00000010) >> 1;
            self.lfsr = cur_lfsr >> 1;
            self.lfsr = self.lfsr & !(1 << 14);
            self.lfsr = self.lfsr | (xor << 14);

            if self.short_mode {
                self.lfsr = self.lfsr & !(1 << 6);
                self.lfsr = self.lfsr | (xor << 6);
            }

            self.timer = self.timer_period;
        }
    }
    fn output(&mut self) -> u8 {
        if (self.lfsr & 0b00000001) == 0 {
            self.envelope.volume
        } else {
            0
        }
    }
}

struct EnvelopeState {
    volume: u8,
    env_timer: u8,
    env_period: u8,
    env_direction: bool,
}
struct SweepState {
    sweep_timer: u16,
    sweep_direction: bool,
    sweep_shift: u8,
    sweep_frec: u16,
    sweep_pace: u8,
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
