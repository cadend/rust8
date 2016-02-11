use std::fmt;
use std::fs::File;
use std::io::Read;

use sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::EventPump;

use time::PreciseTime;

const MEM_SIZE: usize = 4096;
const ROM_ADDR: usize = 0x200;
const FRAMES_PER_SECOND: i64 = 60;
const SKIP_TICKS: i64 = 1000 / FRAMES_PER_SECOND;

#[derive(Debug, Default)]
struct Registers {
    reg_gp: [u8; 16],
    reg_i: u16,

    reg_delay: u8,
    reg_sound: u8,

    reg_pc: u16,
    reg_sp: u8,

    stack: [u16; 16],
}

impl Registers {
    fn new() -> Registers {
        let mut reg = Registers::default();
        reg.reg_pc = ROM_ADDR as u16;
        reg
    }

    fn write_register(&mut self, target_reg: u8, data_value: u8) {
        self.reg_gp[target_reg as usize] = data_value;
    }

    fn write_register_i(&mut self, data_value: u16) {
        self.reg_i = data_value;
    }

    fn write_delay_timer(&mut self, data_value: u8) {
        self.reg_delay = data_value;
    }

    fn read_register(&self, target_reg: u8) -> u8 {
        self.reg_gp[target_reg as usize]
    }

    fn read_register_i(&self) -> u16 {
        self.reg_i
    }

    fn read_delay_timer(&self) -> u8 {
        self.reg_delay
    }

    fn read_pc(&self) -> u16 {
        self.reg_pc
    }

    fn jump_to_address(&mut self, addr: u16, jump_type: JumpType) {
        match jump_type {
            JumpType::SUBROUTINE => {
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_sp += 1;
            }
            JumpType::NORMAL => {}
        }
        self.reg_pc = addr;
    }

    fn return_from_subroutine(&mut self) {
        self.reg_pc = self.stack[(self.reg_sp - 1) as usize];
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
                }
                Err(e) => panic!("Some error {:?} occurred while storing program data.", e),
            }
        }
    }

    fn load_fonts(&mut self) {
        let font_file = File::open("./font.bin").unwrap();
        let mut mem_addr = 0x0;
        for byte in font_file.bytes() {
            match byte {
                Ok(b) => {
                    self.mem[mem_addr] = b;
                    mem_addr += 1;
                }
                Err(e) => panic!("Some error {:?} occurred while loading font data.", e),
            }
        }
    }

    fn read_byte(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write_byte(&mut self, address: u16, new_byte: u8) {
        self.mem[address as usize] = new_byte;
    }

    fn _display_pong_rom(&self) {
        let mut addr = ROM_ADDR;
        for _ in 1..100 {
            println!("{:#x}", self.mem[addr]);
            addr += 1;
        }
    }

    fn _display_font_data(&self) {
        let mut addr = 0x0;
        for _ in 0..80 {
            println!("{:#x}", self.mem[addr]);
            addr += 1;
        }
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TODO implement mem debug")
    }
}

impl Default for Memory {
    fn default() -> Memory {
        Memory { mem: [0u8; MEM_SIZE] }
    }
}

pub struct Chip8<'a> {
    reg: Registers,
    mem: Memory,
    keys: Keypad,
    sdl_event_pump: EventPump,
    window: Renderer<'a>,
}

impl<'a> fmt::Debug for Chip8<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}{:#?}{:#?}", self.reg, self.mem, self.keys)
    }
}

impl<'a> Chip8<'a> {
    pub fn new() -> Chip8<'a> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let new_window = video_subsystem.window("rust-sdl2", 800, 600)
                                        .position_centered()
                                        .opengl()
                                        .build()
                                        .unwrap();

        let renderer = new_window.renderer().build().unwrap();

        Chip8 {
            reg: Registers::new(),
            mem: Memory::default(),
            keys: Keypad::default(),
            sdl_event_pump: sdl_context.event_pump().unwrap(),
            window: renderer,
        }
    }

    pub fn init_display(&mut self) {
        self.mem.load_fonts();

        self.window.set_draw_color(Color::RGB(255, 0, 0));
        self.window.clear();
        self.window.present();
    }

    pub fn run(&mut self) {

        let mut start_time = PreciseTime::now();
        let mut diff;

        'running: loop {
            let end_time = PreciseTime::now();
            diff = start_time.to(end_time).num_milliseconds();
            if diff >= SKIP_TICKS {
                start_time = end_time;
                for event in self.sdl_event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), .. } => {
                            break 'running
                        }
                        _ => {}
                    }
                }

                let delay_timer_value = self.reg.read_delay_timer();
                if delay_timer_value > 0 {
                    self.reg.write_delay_timer(delay_timer_value - 1);
                }

                let instruction = self.read_word();
                self.process_instruction(instruction);
            } else {
                continue;
            }
        }
    }

    pub fn store_program_data(&mut self, rom: File) {
        self.mem.store_program_data(rom);
    }

    pub fn _debug_pong_rom(&self) {
        self.mem._display_pong_rom();
    }

    pub fn _debug_font_data(&self) {
        self.mem._display_font_data();
    }

    fn read_word(&mut self) -> u16 {
        let instruction_high_order = (self.mem.mem[self.reg.reg_pc as usize] as u16) << 8;
        let instruction_low_order = self.mem.mem[(self.reg.reg_pc + 1) as usize] as u16;

        let instruction = instruction_high_order | instruction_low_order;

        self.reg.reg_pc += 2;
        instruction
    }

    fn process_instruction(&mut self, instruction: u16) {
        let op_type: u8 = ((instruction >> 12) & 0xff) as u8;

        match op_type {
            0x0 => {
                // we will ignore the 0nnn opcode used for jumping to machine code routines
                let operation = instruction & 0x00ff;
                if operation == 0xe0 {
                    println!("PC: {}    |    Opcode: {:#x}      |    cls",
                             self.reg.read_pc() - 2,
                             instruction);
                    println!("clear display");
                } else if operation == 0xee {
                    println!("PC: {}    |    Opcode: {:#x}      |    ret",
                             self.reg.read_pc() - 2,
                             instruction);
                    self.reg.return_from_subroutine();
                }
            }
            0x1 => {
                let jump_addr = instruction & 0x0fff;
                println!("PC: {}    |    Opcode: {:#x}    |    jmp {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         jump_addr);
                self.reg.jump_to_address(jump_addr, JumpType::NORMAL);
            }
            0x2 => {
                let subroutine_addr = instruction & 0x0fff;
                println!("PC: {}    |    Opcode: {:#x}    |    call {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         subroutine_addr);
                self.reg.jump_to_address(subroutine_addr, JumpType::SUBROUTINE);
            }
            0x6 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let data_value = (instruction & 0x00ff) as u8;
                println!("PC: {}    |    Opcode: {:#x}    |    ld V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         data_value);
                self.reg.write_register(target_reg, data_value);
            }
            0x7 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let immediate_value = (instruction & 0x00ff) as u8;
                let reg_value = self.reg.read_register(target_reg);
                let data_value = immediate_value.wrapping_add(reg_value);
                println!("PC: {}    |    Opcode: {:#x}    |    add V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         immediate_value);
                self.reg.write_register(target_reg, data_value);
            }
            0x8 => {
                let reg_one = ((instruction >> 8) & 0x0f) as u8;
                let reg_two = ((instruction >> 4) & 0x0f) as u8;
                let operation = (instruction & 0x000f) as u8;
                match operation {
                    0 => {
                        let data_value = self.reg.read_register(reg_two);
                        println!("PC: {}    |    Opcode: {:#x}    |    ld V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, data_value);
                    }
                    1 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        let data_value = reg_one_value | reg_two_value;
                        println!("PC: {}    |    Opcode: {:#x}    |    or V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, data_value);
                    }
                    2 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        let data_value = reg_one_value & reg_two_value;
                        println!("PC: {}    |    Opcode: {:#x}    |    and V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, data_value);
                    }
                    3 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);
                        if reg_two_value > reg_one_value {
                            self.reg.write_register(0x0f, 0x01);
                        }
                        let data_value = reg_two_value - reg_one_value;
                        println!("PC: {}    |    Opcode: {:#x}    |    xor V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, data_value);
                    }
                    4 => {}
                    5 => {}
                    6 => {}
                    7 => {}
                    0xe => {}
                    _ => panic!("Unrecognized opcode: {:#x}", instruction),
                }
            }
            0xa => {
                let data_value = instruction & 0x0fff;
                println!("PC: {}    |    Opcode: {:#x}    |    ld i {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         data_value);
                self.reg.write_register_i(data_value);
            }
            0xb => {
                let initial_addr = instruction & 0x0fff;
                let offset = self.reg.read_register(0) as u16;
                println!("PC: {}    |    Opcode: {:#x}    |    jp V0 {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         initial_addr + offset);
                self.reg.jump_to_address(initial_addr + offset, JumpType::NORMAL);
            }
            0xd => {
                println!("PC: {}    |    Opcode: {:#x}    |    drw",
                         self.reg.read_pc() - 2,
                         instruction);
            }
            0xe => {
                // TODO: input checks
                println!("PC: {}    |    Opcode: {:#x}    |    input",
                         self.reg.read_pc() - 2,
                         instruction);
            }
            0xf => {
                let operation = (instruction & 0x00FF) as u8;
                let register_index = ((instruction & 0x0F00) >> 8) as u8;

                match operation {
                    0x07 => {
                        println!("PC: {}    |    Opcode: {:#x}    |    ld V{} DT",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let reg_value = self.reg.read_delay_timer();
                        self.reg.write_register(register_index, reg_value);
                    }
                    0x15 => {
                        println!("PC: {}    |    Opcode: {:#x}    |    ld DT V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let reg_value = self.reg.read_register(register_index);
                        self.reg.write_delay_timer(reg_value);
                    }
                    0x29 => {
                        println!("PC: {}    |    Opcode: {:#x}    |    ld F V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);

                        let reg_value = self.reg.read_register(register_index);
                        match reg_value {
                            0 => {
                                self.reg.write_register_i(0x0);
                            }
                            1 => {
                                self.reg.write_register_i(0x5);
                            }
                            2 => {
                                self.reg.write_register_i(0xa);
                            }
                            3 => {
                                self.reg.write_register_i(0xf);
                            }
                            4 => {
                                self.reg.write_register_i(0x14);
                            }
                            5 => {
                                self.reg.write_register_i(0x19);
                            }
                            6 => {
                                self.reg.write_register_i(0x1e);
                            }
                            7 => {
                                self.reg.write_register_i(0x23);
                            }
                            8 => {
                                self.reg.write_register_i(0x28);
                            }
                            9 => {
                                self.reg.write_register_i(0x2d);
                            }
                            0xa => {
                                self.reg.write_register_i(0x32);
                            }
                            0xb => {
                                self.reg.write_register_i(0x37);
                            }
                            0xc => {
                                self.reg.write_register_i(0x3c);
                            }
                            0xd => {
                                self.reg.write_register_i(0x41);
                            }
                            0xe => {
                                self.reg.write_register_i(0x46);
                            }
                            0xf => {
                                self.reg.write_register_i(0x4b);
                            }
                            _ => {
                                panic!("Should never hit this statement, all cases covered.");
                            }
                        }
                    }
                    0x33 => {
                        let mut reg_value = self.reg.read_register(register_index);
                        let ones_digit: u8 = reg_value % 10;
                        reg_value = reg_value / 10;
                        let tens_digit: u8 = reg_value % 10;
                        reg_value = reg_value / 10;
                        let hundreds_digit: u8 = reg_value % 10;

                        println!("PC: {}    |    Opcode: {:#x}    |    ld B V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);

                        self.mem.write_byte(self.reg.read_register_i(), hundreds_digit);
                        self.mem.write_byte(self.reg.read_register_i() + 1, tens_digit);
                        self.mem.write_byte(self.reg.read_register_i() + 2, ones_digit);
                    }
                    0x65 => {
                        println!("PC: {}    |    Opcode: {:#x}    |    ld V{} [I]",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let mem_addr = self.reg.read_register_i();
                        for n in 0..(register_index + 1) {
                            let byte = self.mem.read_byte(mem_addr + (n as u16));
                            self.reg.write_register(n as u8, byte);
                        }
                    }
                    _ => {
                        println!("Chip8 status at end time: {:#?}", self);
                        println!("*************Unrecognized opcode!*************");
                        panic!("PC: {}    |    Opcode: {:#x}    |    various",
                               self.reg.read_pc() - 2,
                               instruction);
                    }
                }
            }
            _ => {
                println!("Chip8 status at end time: {:#?}", self);
                panic!("Unsupported op type: {:#x}", op_type)
            }
        }
    }
}

enum JumpType {
    NORMAL,
    SUBROUTINE,
}
