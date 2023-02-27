mod cpu;
mod display;
mod keyboard;
mod rom;

use cpu::Cpu;
use std::fs;

fn main() {
    let mut cpu = Cpu::new();

    cpu.execute_instruction(0x00E0);

    // let rom_path = "roms/invaders.ch8";
    // let rom_data = fs::read(rom_path).expect("Failed to read ROM file");
    // let mut memory = [0; 4096];
    // for (i, byte) in rom_data.iter().enumerate() {
    //     memory[0x200 + i] = *byte;
    // }

    // print!("ROM data: {:?}", rom_data);
}
