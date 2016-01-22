use std::fmt;

const RAM_SIZE: usize = 4096;

#[derive(Debug, Default)]
struct Registers {
	ram: Ram,
	reg_gp: [u8; 16],
	reg_i: u16,

	reg_delay: u8,
	reg_sound: u8,

	reg_pc: u16,
	reg_sp: u8,

	stack: [u16; 16]
}

struct Ram {
	ram: Vec<u8>,
}

impl fmt::Debug for Ram {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "TODO: Implement debug for RAM")
	}
}

impl Default for Ram {
	fn default() -> Ram {
		Ram {
			ram: vec![0; RAM_SIZE],
		}
	}
}

#[derive(Debug)]
struct Chip8 {
	reg: Registers
}

impl Chip8 {
	fn new() -> Chip8 {
		Chip8 {
			reg: Registers::default(),
		}
	}
}

fn main() {
	let chip8_emu = Chip8::new();

	println!("{:#?}", chip8_emu);
}