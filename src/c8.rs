use std::fmt;
use std::fs::File;
use std::io::Read;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::EventPump;

use rand;

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

    fn increment_pc(&mut self) {
        self.reg_pc += 2;
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
    keys: [bool; 16],
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

        let new_window = video_subsystem.window("Rust8", 640, 320)
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

        self.window.set_draw_color(Color::RGB(0, 0, 0));
        self.window.clear();
        self.window.present();
        self.window.set_draw_color(Color::RGB(255, 255, 255));
    }

    pub fn run(&mut self) {

        'running: loop {

            for event in self.sdl_event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), .. } => break 'running,
                    Event::KeyDown {keycode: Some(Keycode::Num1), ..} => {
                        self.keys.keys[1] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::Num2), ..} => {
                        self.keys.keys[2] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::Num3), ..} => {
                        self.keys.keys[3] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::Num4), ..} => {
                        self.keys.keys[12] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::Q), ..} => {
                        self.keys.keys[4] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::W), ..} => {
                        self.keys.keys[5] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::E), ..} => {
                        self.keys.keys[6] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::R), ..} => {
                        self.keys.keys[13] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::A), ..} => {
                        self.keys.keys[7] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::S), ..} => {
                        self.keys.keys[8] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::D), ..} => {
                        self.keys.keys[9] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::F), ..} => {
                        self.keys.keys[14] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::Z), ..} => {
                        self.keys.keys[10] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::X), ..} => {
                        self.keys.keys[0] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::C), ..} => {
                        self.keys.keys[11] = true;
                    }
                    Event::KeyDown {keycode: Some(Keycode::V), ..} => {
                        self.keys.keys[15] = true;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Num1), ..} => {
                        self.keys.keys[1] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Num2), ..} => {
                        self.keys.keys[2] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Num3), ..} => {
                        self.keys.keys[3] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Num4), ..} => {
                        self.keys.keys[12] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Q), ..} => {
                        self.keys.keys[4] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::W), ..} => {
                        self.keys.keys[5] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::E), ..} => {
                        self.keys.keys[6] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::R), ..} => {
                        self.keys.keys[13] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::A), ..} => {
                        self.keys.keys[7] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::S), ..} => {
                        self.keys.keys[8] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::D), ..} => {
                        self.keys.keys[9] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::F), ..} => {
                        self.keys.keys[14] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::Z), ..} => {
                        self.keys.keys[10] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::X), ..} => {
                        self.keys.keys[0] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::C), ..} => {
                        self.keys.keys[11] = false;
                    }
                    Event::KeyUp {keycode: Some(Keycode::V), ..} => {
                        self.keys.keys[15] = false;
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

        self.reg.increment_pc();
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
            0x3 => {
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;
                let comparison_byte = (instruction & 0x00ff) as u8;
                if self.reg.read_register(target_reg) == comparison_byte {
                    self.reg.increment_pc();
                }
                println!("PC: {}    |    Opcode: {:#x}    |    se V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         comparison_byte);
            }
            0x4 => {
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;
                let comparison_byte = (instruction & 0x00ff) as u8;
                if self.reg.read_register(target_reg) != comparison_byte {
                    self.reg.increment_pc();
                }
                println!("PC: {}    |    Opcode: {:#x}    |    se V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         comparison_byte);
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
            0xc => {
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;
                let combination_byte = (instruction & 0x00ff) as u8;
                let rand_num: u8 = rand::random();

                self.reg.write_register(target_reg, (combination_byte & rand_num));
                println!("PC: {}    |    Opcode: {:#x}    |    rnd V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         combination_byte);
            }
            0xd => {
                let reg_one = ((instruction & 0x0F00) >> 8) as u8;
                let reg_two = ((instruction & 0x00F0) >> 4) as u8;
                let num_bytes = (instruction & 0x000F) as u8;
                println!("PC: {}    |    Opcode: {:#x}    |    drw V{} V{} {}",
                         self.reg.read_pc() - 2,
                         instruction,
                         reg_one,
                         reg_two,
                         num_bytes);

                let sprite_x = self.reg.read_register(reg_one);
                let sprite_y = self.reg.read_register(reg_two);
                println!("Sprite X: {}  |  Sprite Y: {}", sprite_x, sprite_y);
                let mut bit_vec: Vec<u8> = Vec::new();
                let mut rect_vec: Vec<Rect> = Vec::new();
                for i in 0..num_bytes {
                    bit_vec.push(self.mem.read_byte(self.reg.read_register_i() + (i as u16)));
                }

                println!("Glyph:");
                for byte in bit_vec.clone() {
                    println!("{:#8b}", byte);
                }
                println!("");

                let mut index = 0;
                for byte in bit_vec {
                    for i in 0..8 {
                        if ((byte >> i) & 1) == 1 {
                            rect_vec.push(Rect::new_unwrap((((sprite_x as i32) * 10) +
                                                            ((7 - i) * 10)),
                                                           (((sprite_y as i32) * 10) +
                                                            (index * 10)),
                                                           10,
                                                           10));
                        }
                    }
                    index += 1;
                }

                // TODO switch to texture.with_lock so that the pixels can be XOR'd
                for r in rect_vec {
                    println!("Drawing 10*10 at {},{}", r.x(), r.y());
                    self.window.fill_rect(r);
                }

                self.window.present();

            }
            0xe => {
                let optype = (instruction & 0x00ff) as u8;
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;

                match optype {
                    0x9e => {
                        let key = self.reg.read_register(target_reg);
                        if self.keys.keys[key as usize] == true {
                            self.reg.increment_pc();
                        }
                        println!("PC: {}    |    Opcode: {:#x}    |    skp V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 target_reg);
                    }
                    0xa1 => {
                        let key = self.reg.read_register(target_reg);
                        if self.keys.keys[key as usize] == false {
                            self.reg.increment_pc();
                        }
                        println!("PC: {}    |    Opcode: {:#x}    |    skp V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 target_reg);
                    }
                    _ => panic!("Invalid instruction: {:#4x}", instruction),
                }
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
                panic!("Unsupported op type: {:#2x}", op_type);
            }
        }
    }
}

enum JumpType {
    NORMAL,
    SUBROUTINE,
}
