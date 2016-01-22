mod c8;

use std::env;
use std::fs::File;

fn main() {
    let program_path = env::args().nth(1).unwrap();
    println!("ROM path: {}", program_path);

    let rom_file = File::open(program_path).unwrap();
    let mut chip8_emu = c8::Chip8::new();

    chip8_emu.store_program_data(rom_file);
    println!("{:#?}", chip8_emu);

    chip8_emu.run();
}