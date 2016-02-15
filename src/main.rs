extern crate sdl2;
extern crate time;
extern crate rand;

mod cpu;

use std::env;
use std::fs::File;
use cpu::cpu::Chip8;

fn main() {
    let program_path = env::args().nth(1).unwrap();
    println!("ROM path: {}", program_path);

    let rom_file = File::open(program_path).unwrap();
    println!("Opened file");
    let mut chip8_emu = Chip8::new();

    chip8_emu.store_program_data(rom_file);

    chip8_emu.init_display();

    let debug_arg = env::args().nth(2);
    match debug_arg {
        Some(d) => {
            if &d == "-d" {
                chip8_emu._run_debug();
            } else {
                chip8_emu.run();
            }
        }
        None => {
            chip8_emu.run();
        }
    }
}
