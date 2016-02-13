extern crate sdl2;
extern crate time;
extern crate rand;

mod c8;

use std::env;
use std::fs::File;

fn main() {
    let program_path = env::args().nth(1).unwrap();
    println!("ROM path: {}", program_path);

    let rom_file = File::open(program_path).unwrap();
    let mut chip8_emu = c8::Chip8::new();

    chip8_emu.store_program_data(rom_file);

    chip8_emu.init_display();

    chip8_emu.run();
}
