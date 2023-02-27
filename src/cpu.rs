const OPCODE_SIZE: usize = 2;
const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
const CHIP8_RAM: usize = 4096;

enum ProgramCounterChange {
    Inc,
    Jump(usize),
}

pub struct Cpu {
    vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    vram_changed: bool,
    ram: [u8; CHIP8_RAM],
    i: usize,
    v: [u8; 16],
    stack: [usize; 16],
    pc: usize, // CPU state
    sp: usize,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc: 0x200,
            vram: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            vram_changed: false,
            ram: [0; CHIP8_RAM],
            i: 0,
            stack: [0; 16],
            v: [0; 16],
            sp: 0,
        }
    }

    pub fn parse_opcode(&self, opcode: u16) -> (u8, u8, u8, u8) {
        (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        )
    }

    pub fn execute_instruction(&mut self, opcode: u16) {
        let opcode = 0x00E0;
        let (n1, n2, n3, n4) = self.parse_opcode(opcode);

        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = n2;
        let y = n3;
        let n = n4;

        let pc_change: ProgramCounterChange = match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.ox_00e0(),
            (0x0, 0x0, 0xE, 0xE) => self.ox_00ee(),
            (0x1, _, _, _) => self.ox_1nnn(nnn),
            (0x2, _, _, _) => self.ox_2nnn(nnn),
            _ => ProgramCounterChange::Inc,
        };

        match pc_change {
            ProgramCounterChange::Inc => self.pc += OPCODE_SIZE,
            ProgramCounterChange::Jump(addr) => self.pc = addr,
        };
    }

    fn ox_00e0(&mut self) -> ProgramCounterChange {
        // Clear the display
        self.vram = [[0; CHIP8_WIDTH]; CHIP8_HEIGHT];
        self.vram_changed = true;
        ProgramCounterChange::Inc
    }

    fn ox_00ee(&mut self) -> ProgramCounterChange {
        //TODO: check if it works
        let addr = self.stack[self.sp];
        self.sp -= 1;
        ProgramCounterChange::Jump(addr)
    }

    fn ox_1nnn(&mut self, nnn: usize) -> ProgramCounterChange {
        ProgramCounterChange::Jump(nnn)
    }

    fn ox_2nnn(&mut self, nnn: usize) -> ProgramCounterChange {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        ProgramCounterChange::Jump(nnn)
    }
}
