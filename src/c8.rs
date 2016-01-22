use std::fmt;
use std::fs::File;
use std::io::Read;

const MEM_SIZE: usize = 4096;
const ROM_ADDR: usize = 0x200;

#[derive(Debug, Default)]
struct Registers {
	reg_gp: [u8; 16],
	reg_i: u16,

	reg_delay: u8,
	reg_sound: u8,

	reg_pc: u16,
	reg_sp: u8,

	stack: [u16; 16]
}

#[derive(Default, Debug)]
struct Keypad {
	keys: [u8; 16],
}

struct Memory {
	mem: [u8; MEM_SIZE],
}

impl Memory {
	fn store_program_data(&mut self, rom: File) {
		let mut last_stored_addr = ROM_ADDR;

		for byte in rom.bytes() {
			match byte {
				Ok(b) => {
					self.mem[last_stored_addr] = b;
					last_stored_addr += 2;
				},
				Err(e) => panic!("Some error {:?} occurred while storing program data.", e)
			}
		}
	}
}

impl fmt::Debug for Memory {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "First opcode: {:#x} {:#x}", self.mem[0x200], self.mem[0x202])
	}
}

impl Default for Memory {
	fn default() -> Memory {
		Memory {
			mem: [0u8; MEM_SIZE]
		}
	}
}

#[derive(Debug)]
pub struct Chip8 {
	reg: Registers,
	mem: Memory,
	keys: Keypad
}

impl Chip8 {
	pub fn new() -> Chip8 {
		Chip8 {
			reg: Registers::default(),
			mem: Memory::default(),
			keys: Keypad::default()
		}
	}

	pub fn store_program_data(&mut self, rom: File) {
		self.mem.store_program_data(rom);
	}
}