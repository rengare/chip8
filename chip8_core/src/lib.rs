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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_and_tick(emu: &mut Emu, opcodes: &[u16]) {
        let bytes: Vec<u8> = opcodes
            .iter()
            .flat_map(|&op| [(op >> 8) as u8, op as u8])
            .collect();
        emu.load(&bytes);
        for _ in opcodes {
            emu.tick();
        }
    }

    // --- 0x00E0: Clear screen ---
    #[test]
    fn test_clear_screen() {
        let mut emu = Emu::new();
        // Dirty the screen
        emu.screen[0] = 1;
        emu.screen[100] = 1;
        load_and_tick(&mut emu, &[0x00E0]);
        assert!(emu.screen.iter().all(|&p| p == 0));
    }

    // --- 0x00EE / 0x2NNN: call and return ---
    #[test]
    fn test_call_and_return() {
        let mut emu = Emu::new();
        // CALL 0x300 then RET
        // Place RET at 0x300
        emu.ram[0x300] = 0x00;
        emu.ram[0x301] = 0xEE;
        load_and_tick(&mut emu, &[0x2300]); // CALL 0x300 → pc=0x300, stack has 0x202
        assert_eq!(emu.pc, 0x300);
        emu.tick(); // RET → pc=0x202
        assert_eq!(emu.pc, 0x202);
        assert_eq!(emu.sp, 0);
    }

    // --- 0x1NNN: jump ---
    #[test]
    fn test_jump() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x1300]);
        assert_eq!(emu.pc, 0x300);
    }

    // --- 0x3XNN: skip if VX == NN ---
    #[test]
    fn test_skip_if_vx_eq_nn_true() {
        let mut emu = Emu::new();
        // Set V1=0x42, then 3142 should skip
        load_and_tick(&mut emu, &[0x6142, 0x3142]);
        // After load+tick of 6142: pc=0x202; after 3142 with skip: pc=0x206
        assert_eq!(emu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_skip_if_vx_eq_nn_false() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6100, 0x3142]);
        assert_eq!(emu.pc, START_ADDR + 4);
    }

    // --- 0x4XNN: skip if VX != NN ---
    #[test]
    fn test_skip_if_vx_ne_nn() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6100, 0x4142]);
        assert_eq!(emu.pc, START_ADDR + 6);
    }

    // --- 0x5XY0: skip if VX == VY ---
    #[test]
    fn test_skip_if_vx_eq_vy() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6142, 0x6242, 0x5120]);
        assert_eq!(emu.pc, START_ADDR + 8);
    }

    // --- 0x6XNN: set VX = NN ---
    #[test]
    fn test_set_vx() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A42]);
        assert_eq!(emu.v_reg[0xA], 0x42);
    }

    // --- 0x7XNN: VX += NN (wrapping) ---
    #[test]
    fn test_add_vx_nn() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A10, 0x7A05]);
        assert_eq!(emu.v_reg[0xA], 0x15);
    }

    #[test]
    fn test_add_vx_nn_wrapping() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6AFF, 0x7A01]);
        assert_eq!(emu.v_reg[0xA], 0x00);
    }

    // --- 0x8XY0-7,E arithmetic/logic ---
    #[test]
    fn test_8xy0_assign() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A42, 0x8BA0]);
        assert_eq!(emu.v_reg[0xB], 0x42);
    }

    #[test]
    fn test_8xy1_or() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A0F, 0x6BF0, 0x8AB1]);
        assert_eq!(emu.v_reg[0xA], 0xFF);
    }

    #[test]
    fn test_8xy2_and() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6AFF, 0x6B0F, 0x8AB2]);
        assert_eq!(emu.v_reg[0xA], 0x0F);
    }

    #[test]
    fn test_8xy3_xor() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6AFF, 0x6BF0, 0x8AB3]);
        assert_eq!(emu.v_reg[0xA], 0x0F);
    }

    #[test]
    fn test_8xy4_add_no_carry() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A01, 0x6B02, 0x8AB4]);
        assert_eq!(emu.v_reg[0xA], 3);
        assert_eq!(emu.v_reg[VF], 0);
    }

    #[test]
    fn test_8xy4_add_carry() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6AFF, 0x6B01, 0x8AB4]);
        assert_eq!(emu.v_reg[0xA], 0);
        assert_eq!(emu.v_reg[VF], 1);
    }

    #[test]
    fn test_8xy5_sub_no_borrow() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A05, 0x6B03, 0x8AB5]);
        assert_eq!(emu.v_reg[0xA], 2);
        assert_eq!(emu.v_reg[VF], 1);
    }

    #[test]
    fn test_8xy5_sub_borrow() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A01, 0x6B02, 0x8AB5]);
        assert_eq!(emu.v_reg[0xA], 0xFF);
        assert_eq!(emu.v_reg[VF], 0);
    }

    #[test]
    fn test_8xy6_shr() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A06, 0x8A06]);
        assert_eq!(emu.v_reg[0xA], 3);
        assert_eq!(emu.v_reg[VF], 0);
    }

    #[test]
    fn test_8xy6_shr_lsb() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A07, 0x8A06]);
        assert_eq!(emu.v_reg[0xA], 3);
        assert_eq!(emu.v_reg[VF], 1);
    }

    #[test]
    fn test_8xye_shl() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A04, 0x8A0E]);
        assert_eq!(emu.v_reg[0xA], 8);
        assert_eq!(emu.v_reg[VF], 0);
    }

    #[test]
    fn test_8xye_shl_msb() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A80, 0x8A0E]);
        assert_eq!(emu.v_reg[0xA], 0);
        assert_eq!(emu.v_reg[VF], 1);
    }

    #[test]
    fn test_8xy7_subn_no_borrow() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A03, 0x6B05, 0x8AB7]);
        assert_eq!(emu.v_reg[0xA], 2);
        assert_eq!(emu.v_reg[VF], 1);
    }

    // --- 0x9XY0: skip if VX != VY ---
    #[test]
    fn test_9xy0_skip_if_ne() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A01, 0x6B02, 0x9AB0]);
        assert_eq!(emu.pc, START_ADDR + 8);
    }

    // --- 0xANNN: set I ---
    #[test]
    fn test_set_i() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0xA123]);
        assert_eq!(emu.i_reg, 0x123);
    }

    // --- 0xBNNN: jump V0 + NNN ---
    #[test]
    fn test_jump_v0() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6002, 0xB300]);
        assert_eq!(emu.pc, 0x302);
    }

    // --- 0xFX07/15: delay timer ---
    #[test]
    fn test_delay_timer_set_get() {
        let mut emu = Emu::new();
        // Set V1=5, store to DT (F115), then load DT into V2 (F207)
        load_and_tick(&mut emu, &[0x6105, 0xF115, 0xF207]);
        assert_eq!(emu.v_reg[2], 5);
    }

    // --- tick_timers ---
    #[test]
    fn test_tick_timers_decrement() {
        let mut emu = Emu::new();
        emu.dt = 3;
        emu.st = 2;
        emu.tick_timers();
        assert_eq!(emu.dt, 2);
        assert_eq!(emu.st, 1);
        emu.tick_timers();
        assert_eq!(emu.dt, 1);
        assert_eq!(emu.st, 0);
        emu.tick_timers();
        assert_eq!(emu.dt, 0);
        assert_eq!(emu.st, 0); // should not underflow
    }

    // --- 0xFX18: sound timer ---
    #[test]
    fn test_sound_timer_set() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A07, 0xFA18]);
        assert_eq!(emu.st, 7);
    }

    // --- 0xFX1E: I += VX ---
    #[test]
    fn test_i_add_vx() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0xA100, 0x6A05, 0xFA1E]);
        assert_eq!(emu.i_reg, 0x105);
    }

    // --- 0xFX29: font sprite ---
    #[test]
    fn test_font_sprite() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0x6A03, 0xFA29]); // digit 3 → addr 15
        assert_eq!(emu.i_reg, 15);
    }

    // --- 0xFX33: BCD ---
    #[test]
    fn test_bcd() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0xA300, 0x6A96, 0xFA33]); // V_A = 150
        assert_eq!(emu.ram[0x300], 1);
        assert_eq!(emu.ram[0x301], 5);
        assert_eq!(emu.ram[0x302], 0);
    }

    // --- 0xFX55 / 0xFX65: store/load registers ---
    #[test]
    fn test_store_load_registers() {
        let mut emu = Emu::new();
        // Set V0=1, V1=2, V2=3, store regs 0..2, then clear and reload
        let program: Vec<u16> = vec![
            0x6001, 0x6102, 0x6203, // set V0-V2
            0xA400, // I = 0x400
            0xF255, // store V0-V2 at I
            0x6000, 0x6100, 0x6200, // clear V0-V2
            0xA400, // I = 0x400
            0xF265, // load V0-V2 from I
        ];
        load_and_tick(&mut emu, &program);
        assert_eq!(emu.v_reg[0], 1);
        assert_eq!(emu.v_reg[1], 2);
        assert_eq!(emu.v_reg[2], 3);
    }

    // --- 0xEX9E / 0xEXA1: key skip ---
    #[test]
    fn test_skip_if_key_pressed() {
        let mut emu = Emu::new();
        emu.keypress(5, true);
        load_and_tick(&mut emu, &[0x6A05, 0xEA9E]);
        assert_eq!(emu.pc, START_ADDR + 6);
    }

    #[test]
    fn test_skip_if_key_not_pressed() {
        let mut emu = Emu::new();
        // key 5 is NOT pressed
        load_and_tick(&mut emu, &[0x6A05, 0xEAA1]);
        assert_eq!(emu.pc, START_ADDR + 6);
    }

    // --- 0xFX0A: wait for key ---
    #[test]
    fn test_wait_for_key_no_press() {
        let mut emu = Emu::new();
        load_and_tick(&mut emu, &[0xF00A]);
        // Should have re-decremented pc back to same opcode
        assert_eq!(emu.pc, START_ADDR);
    }

    #[test]
    fn test_wait_for_key_pressed() {
        let mut emu = Emu::new();
        emu.keypress(3, true);
        load_and_tick(&mut emu, &[0xFA0A]);
        assert_eq!(emu.v_reg[0xA], 3);
        assert_eq!(emu.pc, START_ADDR + 2);
    }

    // --- 0xDXYN: draw sprite ---
    #[test]
    fn test_draw_sprite_no_collision() {
        let mut emu = Emu::new();
        // Put a simple 1-row sprite (0xFF) at RAM[0x300]
        emu.ram[0x300] = 0xFF;
        // I=0x300, V0=0, V1=0, draw 1 row
        let program: Vec<u16> = vec![0xA300, 0x6000, 0x6100, 0xD011];
        load_and_tick(&mut emu, &program);
        // 8 pixels at row 0 should be set
        for i in 0..8 {
            assert_eq!(emu.screen[i], 1, "pixel {i} should be set");
        }
        assert_eq!(emu.v_reg[VF], 0, "no collision expected");
    }

    #[test]
    fn test_draw_sprite_collision() {
        let mut emu = Emu::new();
        emu.ram[0x300] = 0xFF;
        let program: Vec<u16> = vec![0xA300, 0x6000, 0x6100, 0xD011, 0xD011];
        load_and_tick(&mut emu, &program);
        // Drawing twice should clear pixels and set VF=1
        for i in 0..8 {
            assert_eq!(emu.screen[i], 0, "pixel {i} should be cleared");
        }
        assert_eq!(emu.v_reg[VF], 1, "collision expected");
    }

    // --- reset ---
    #[test]
    fn test_reset() {
        let mut emu = Emu::new();
        emu.v_reg[0] = 42;
        emu.pc = 0x500;
        emu.reset();
        assert_eq!(emu.pc, START_ADDR);
        assert_eq!(emu.v_reg[0], 0);
    }

    // --- get_display ---
    #[test]
    fn test_get_display_length() {
        let emu = Emu::new();
        assert_eq!(emu.get_display().len(), SCREEN_WIDTH * SCREEN_HEIGHT);
    }
}
