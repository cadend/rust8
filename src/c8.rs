use std::fmt;
use std::fs::File;
use std::io::Read;
use std::ptr;
use super::glium::{ DisplayBuild, Surface };
use super::glium::backend::glutin_backend::GlutinFacade;

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

    fn write_register_i(&mut self, data_value: u16) {
        self.reg_i = data_value;
        println!("Loaded value {:#x} into I register", data_value);
    }

    fn read_register(&self, target_reg: u8) -> u8 {
        self.reg_gp[target_reg as usize]
    }

    fn jump_to_address(&mut self, addr: u16, jump_type: JumpType) {
        match jump_type {
            JumpType::SUBROUTINE => {
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_sp += 1;
            }
            JumpType::NORMAL => {}
        }
        println!("Jumping to address {:#x}", addr);
        self.reg_pc = addr;
    }

    fn return_from_subroutine(&mut self) {
        self.reg_pc = self.stack[self.reg_sp as usize];
        self.reg_sp -= 1;
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
                    last_stored_addr += 1;
                },
                Err(e) => panic!("Some error {:?} occurred while storing program data.", e)
            }
        }
    }

    fn load_fonts(&mut self) {
        let font_file = File::open("./font.bin");
        let mem_addr = 0x0;
        for byte in font_file.bytes() {
            match byte {
                Ok(b) => {
                    self.mem[mem_addr] = b;
                    mem_addr += 1;
                },
                Err(e) => panic!("Some error {:?} occurred while loading font data.", e)
            }
        }
    }

    fn display_pong_rom(&self) {
        let mut addr = ROM_ADDR;
        for i in 1..100 {
            println!("{:#x}", self.mem[addr]);
            addr += 1;
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO implement debug output for memory")
    }
}

impl Default for Memory {
    fn default() -> Memory {
        Memory {
            mem: [0u8; MEM_SIZE]
        }
    }
}

pub struct Chip8 {
    reg: Registers,
    mem: Memory,
    keys: Keypad,
    display: GlutinFacade
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}{:#?}{:#?}", self.reg, self.mem, self.keys)
    }
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            reg: Registers::new(),
            mem: Memory::default(),
            keys: Keypad::default(),
            display: super::glium::glutin::WindowBuilder::new()
                .with_dimensions(64, 32)
                .with_title(String::from("rust8"))
                .build_glium()
                .unwrap()
        }   
    }

    pub fn init_display(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.finish().unwrap();

        self.mem.load_fonts();
    }

    pub fn run(&mut self) {
        loop {
            let instruction = self.read_word();
            self.process_instruction(instruction);

            for ev in self.display.poll_events() {
                match ev {
                    super::glium::glutin::Event::Closed => return,
                    _ => ()
                }
            }
        }
    }

    pub fn store_program_data(&mut self, rom: File) {
        self.mem.store_program_data(rom);
    }

    pub fn debug_pong_rom(&self) {
        self.mem.display_pong_rom();
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
            0x0 => {
                //we will ignore the 0nnn opcode used for jumping to machine code routines
                let operation = instruction & 0x00ff;
                if(operation == 0xe0) {
                    //TODO clear the display
                } else if (operation == 0xee) {
                    self.reg.return_from_subroutine();
                }
            },
            0x1 => {
                let jump_addr = instruction & 0x0fff;
                self.reg.jump_to_address(jump_addr, JumpType::NORMAL);
            },
            0x2 => {
                let subroutine_addr = instruction & 0x0fff;
                self.reg.jump_to_address(subroutine_addr, JumpType::SUBROUTINE);
            },
            0x6 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let data_value = (instruction & 0x00ff) as u8;
                self.reg.write_register(target_reg, data_value);
            },
            0x7 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let immediate_value = (instruction & 0x00ff) as u8;
                let reg_value = self.reg.read_register(target_reg);
                let data_value = immediate_value.wrapping_add(reg_value);
                println!("Addition");
                self.reg.write_register(target_reg, data_value);
            },
            0x8 => {
                let reg_one = ((instruction >> 8) & 0x0f) as u8;
                let reg_two = ((instruction >> 4) & 0x0f) as u8;
                let operation = (instruction & 0x000f) as u8;
                match operation {
                    0 => {
                        let data_value = self.reg.read_register(reg_two);
                        self.reg.write_register(reg_one, data_value);
                    },
                    1 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        let data_value = reg_one_value | reg_two_value;
                        self.reg.write_register(reg_one, data_value);
                    },
                    2 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        let data_value = reg_one_value & reg_two_value;
                        self.reg.write_register(reg_one, data_value);
                    },
                    3 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        if(reg_two_value > reg_one_value) {
                            self.reg.write_register(0x0f, 0x01);
                        }
                        let data_value = reg_two_value - reg_one_value;
                        self.reg.write_register(reg_one, data_value);
                    },
                    4 => {

                    },
                    5 => {

                    },
                    6 => {

                    },
                    7 => {

                    },
                    0xe => {

                    },
                    _ => panic!("Unrecognized opcode: {:#x}", instruction)
                }
            },
            0xa => {
                let data_value = instruction & 0x0fff;
                self.reg.write_register_i(data_value);
            },
            0xb => {
                let initial_addr = instruction & 0x0fff;
                let offset = self.reg.read_register(0) as u16;
                self.reg.jump_to_address(initial_addr + offset, JumpType::NORMAL);
            },
            0xd => {
                //TODO: display/sprites
                println!("Display operation");
            },
            0xe => {
                //TODO: input checks
                println!("Input checks");
            },
            _ => {
                println!("Chip8 status at end time: {:#?}", self);
                panic!("Unsupported op type: {:#x}", op_type)
            }
        }
    }
}

enum JumpType {
    NORMAL, SUBROUTINE
}