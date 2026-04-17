use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const VF: usize = 0xF;
const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const SKIP: u16 = 2;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut emu = Emu {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };
        emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        emu
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[u8] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = start + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn fetch(&mut self) -> u16 {
        let high = self.ram[self.pc as usize] as u16;
        let low = self.ram[(self.pc + 1) as usize] as u16;

        let opcode = (high << 8) | low;
        self.pc += SKIP;

        opcode
    }

    fn execute(&mut self, op: u16) {
        // fetch already increments pc, so we don't need to worry about that here

        let nibbles = (
            (op & 0xF000) >> 12,
            (op & 0x0F00) >> 8,
            (op & 0x00F0) >> 4,
            op & 0x000F,
        );
        let nnn = op & 0x0FFF;
        let nn = (op & 0x00FF) as u8;
        let n = (op & 0x000F) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;

        match nibbles {
            (0, 0, 0, 0) => return,
            (0, 0, 0xE, 0) => {
                self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop();
            }
            (1, _, _, _) => {
                self.pc = nnn;
            }
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = nnn;
            }
            (3, _, _, _) => {
                if self.v_reg[x] == nn {
                    self.pc += SKIP;
                }
            }
            (4, _, _, _) => {
                if self.v_reg[x] != nn {
                    self.pc += SKIP;
                }
            }
            (5, _, _, 0) => {
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += SKIP;
                }
            }
            (6, _, _, _) => {
                self.v_reg[x] = nn;
            }
            (7, _, _, _) => {
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            (8, _, _, 0) => {
                self.v_reg[x] = self.v_reg[y];
            }
            (8, _, _, 1) => {
                self.v_reg[x] |= self.v_reg[y];
            }
            (8, _, _, 2) => {
                self.v_reg[x] &= self.v_reg[y];
            }
            (8, _, _, 3) => {
                self.v_reg[x] ^= self.v_reg[y];
            }
            (8, _, _, 4) => {
                let (vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let vf = if carry { 1 } else { 0 };
                self.v_reg[x] = vx;
                self.v_reg[VF] = vf;
            }
            (8, _, _, 5) => {
                let (vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = vx;
                self.v_reg[VF] = vf;
            }

            (8, _, _, 6) => {
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[VF] = lsb;
            }
            (8, _, _, 7) => {
                let (vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let vf = if borrow { 0 } else { 1 };
                self.v_reg[x] = vx;
                self.v_reg[VF] = vf;
            }
            (8, _, _, 0xE) => {
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[VF] = msb;
            }

            (9, _, _, 0) => {
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += SKIP;
                }
            }
            (0xA, _, _, _) => {
                self.i_reg = nnn;
            }
            (0xB, _, _, _) => {
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[x] as u16;
                let y_coord = self.v_reg[y] as u16;
                let mut flipped = 0;

                for y_line in 0..(n as u16) {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Get our pixel's index for our 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= 1;
                        }
                    }
                }

                self.v_reg[VF] = flipped;
            }

            (0xE, _, 9, 0xE) => {
                let key = self.keys[self.v_reg[x] as usize];
                if key {
                    self.pc += SKIP;
                }
            }
            (0xE, _, 0xA, 1) => {
                let key = self.keys[self.v_reg[x] as usize];
                if !key {
                    self.pc += SKIP;
                }
            }
            (0xF, _, 0, 7) => {
                self.v_reg[x] = self.dt;
            }
            (0xF, _, 0, 0xA) => {
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo opcode
                    self.pc -= SKIP;
                }
            }
            (0xF, _, 1, 5) => {
                self.dt = self.v_reg[x];
            }
            (0xF, _, 1, 8) => {
                self.st = self.v_reg[x];
            }
            (0xF, _, 1, 0xE) => {
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            (0xF, _, 2, 9) => {
                self.i_reg = (self.v_reg[x] as u16) * 5; // font sprite take 5 bytes each
            }
            (0xF, _, 3, 3) => {
                let vx = self.v_reg[x] as f32;
                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;
                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF, _, 5, 5) => {
                for idx in 0..=x {
                    self.ram[(self.i_reg as usize) + idx] = self.v_reg[idx];
                }
            }
            (0xF, _, 6, 5) => {
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[(self.i_reg as usize) + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }
    }
}
