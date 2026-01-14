pub const W: usize = 64;
pub const H: usize = 32;
pub const MEM_SIZE: usize = 4096;

pub struct CPU {
    pub memory: [u8; MEM_SIZE],
    pub v: [u8; 16], // registers V0 to VF
    pub i: u16,
    pub pc: u16,
    pub stack: [u16; 16],
    pub sp: u8, // stack pointer

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub gfx: [u8; W * H], // 0/1 pixels
    pub keys: [bool; 16], // keypad state

    pub draw_flag: bool,            // set when DRW/CLS happens
    waiting_for_key: Option<usize>, // if Some(x), store next key press into Vx
}

impl CPU {
    pub fn new() -> Self {
        let c = Self {
            memory: [0; MEM_SIZE],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            gfx: [0; W * H],
            keys: [false; 16],
            draw_flag: false,
            waiting_for_key: None,
        };

        c
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = 0x200;
        let end = rom.len();
        self.memory[start..end].copy_from_slice(rom);
        self.pc = start as u16;
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        if key < 16 {
            self.keys[key] = pressed;
            if let Some(x) = self.waiting_for_key {
                self.v[x] = key as u8;
            }
        }
    }

    pub fn tick_timers_60hz(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn step(&mut self) {
        if self.waiting_for_key.is_some() {
            return;
        }

        let pc = self.pc as usize;
        // let opcode = ((self.memory[pc] as u16) << 8) | (self.memory[pc + 1] as u16);
        let opcode = u16::from_be_bytes([self.memory[pc], self.memory[pc + 1]]);

        self.pc = self.pc.wrapping_add(2);

        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;

        self.draw_flag = false;

        match opcode & 0xF000 {
            0x000 => match opcode {
                0x00E0 => {
                    // CLS
                    self.gfx = [0; W * H];
                    self.draw_flag = true;
                }
                0x00EE => {
                    self.sp = self.sp.wrapping_sub(1);
                    self.pc = self.stack[self.sp as usize];
                }
                _ => {
                    // 0nnn - SYS addr
                    // Jump to a machine code routine at nnn.
                    //
                    // This instruction is only used on the old computers
                    // on which Chip-8 was originally implemented.
                    // It is ignored by modern interpreters.
                }
            },
            0x1000 => {
                self.pc = nnn;
            }
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp = self.sp.wrapping_add(1);
                self.pc = nnn;
            }
            0x3000 => {
                if self.v[x] == kk {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            0x4000 => {
                if self.v[x] != kk {
                    self.pc = self.pc.wrapping_add(2);
                }
            }
            _ => {}
        }
    }
}
