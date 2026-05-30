struct APU {
    active: bool,
    divider: u64, // cpu works at 4Mhz and sample rate is 44.1Khz (4MHz / 44.1KHz = ~95)
    l_master: u8,
    r_master: u8,
    buffer: Vec<f32>,
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
                volume: 0, // 1, 2, 4
                length_timer: 0,
            },
            channel_4: NoiseChannel {
                active: false,
                timer: 0,
                lfsr: 0x7FFF,
                short_mode: false, //whether lfsr uses 7 or 15 bits
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
}

struct SquareChannel {
    active: bool,
    timer: u16,
    duty_pos: u8,
    length_timer: u8,
    sweep: Option<SweepState>,
    envelope: EnvelopeState,
}

struct WaveChannel {
    active: bool,
    timer: u16,
    sample_pos: u8,
    volume: u8, // 1, 2, 4
    length_timer: u8,
}

struct NoiseChannel {
    active: bool,
    timer: u16,
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
