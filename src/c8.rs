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

impl Registers {
    fn new() -> Registers {
        let mut reg = Registers::default();
        reg.reg_pc = (ROM_ADDR as u16);
        reg
    }

    fn write_register(&mut self, target_reg: u8, data_value: u8) {
        self.reg_gp[target_reg as usize] = data_value;
        println!("Loaded value {:#x} into register V{:x}", data_value, target_reg)
    }
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
            reg: Registers::new(),
            mem: Memory::default(),
            keys: Keypad::default()
        }
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.read_word();
            self.process_instruction(instruction);
        }
    }

    pub fn store_program_data(&mut self, rom: File) {
        self.mem.store_program_data(rom);
    }

    fn read_word(&mut self) -> u16 {
        let instruction_high_order = (self.mem.mem[self.reg.reg_pc as usize] as u16) << 8;
        let instruction_low_order = self.mem.mem[(self.reg.reg_pc + 2) as usize] as u16;

        let instruction = instruction_high_order | instruction_low_order;
        println!("Instruction: {:#x}", instruction);

        self.reg.reg_pc += 4;
        instruction
    }

    fn process_instruction(&mut self, instruction: u16) {
        let op_type: u8 = ((instruction >> 12) & 0xff) as u8;

        match op_type {
            0x6 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let data_value = (instruction & 0x00ff) as u8;
                self.reg.write_register(target_reg, data_value);
            },
            _ => {
                println!("Chip8 status at end time: {:#?}", self);
                panic!("Unsupported op type: {:#x}", op_type)
            }
        }
    }
}