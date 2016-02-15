use super::register::Registers;
use super::keypad::Keypad;
use super::memory::Memory;

use std::fmt;
use std::fs::File;

use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Renderer;
use sdl2::EventPump;

use rand;

use time::PreciseTime;


const FRAMES_PER_SECOND: i64 = 4000;
const SKIP_TICKS: i64 = 1000 / FRAMES_PER_SECOND;

pub struct Chip8<'a> {
    reg: Registers,
    mem: Memory,
    keys: Keypad,
    sdl_event_pump: EventPump,
    window: Renderer<'a>,
    display: [[bool; 32]; 64],
    display_updated: bool,
    _next_step: bool,
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
            display: [[false; 32]; 64],
            display_updated: false,
            _next_step: false,
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
        let mut quit = false;
        let mut start_time = PreciseTime::now();
        let mut diff;

        'running: loop {
            let end_time = PreciseTime::now();
            diff = start_time.to(end_time).num_milliseconds();

            self.cpu_cycle();


            quit = self.handle_input();

            if quit == true {
                break 'running;
            }

            if diff >= SKIP_TICKS {
                start_time = end_time;
                let delay_timer_value = self.reg.read_delay_timer();
                if delay_timer_value > 0 {
                    self.reg.write_delay_timer(delay_timer_value - 1);
                }

                let sound_timer_value = self.reg.read_sound_timer();
                if sound_timer_value > 0 {
                    // TODO: actually output a beep or something
                    println!("BEEP!");
                    self.reg.write_sound_timer(sound_timer_value - 1);
                }
            }
            self.render();
        }
    }

    pub fn _run_debug(&mut self) {
        let mut quit = false;
        let mut start_time = PreciseTime::now();
        let mut diff;

        'running: loop {
            let end_time = PreciseTime::now();
            diff = start_time.to(end_time).num_milliseconds();

            while !self._next_step {
                quit = self.handle_input();
                if quit == true {
                    break 'running;
                }
            }
            self._next_step = false;

            self.cpu_cycle();

            if self.display_updated {
                self.render();
            }

            println!("{:?}", self);

            quit = self.handle_input();

            if quit == true {
                break 'running;
            }

            if diff >= SKIP_TICKS {
                start_time = end_time;


                let delay_timer_value = self.reg.read_delay_timer();
                if delay_timer_value > 0 {
                    self.reg.write_delay_timer(delay_timer_value - 1);
                }

                let sound_timer_value = self.reg.read_sound_timer();
                if sound_timer_value > 0 {
                    // TODO: actually output a beep or something
                    println!("BEEP!");
                    self.reg.write_sound_timer(sound_timer_value - 1);
                }
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

    fn cpu_cycle(&mut self) {
        let instruction = self.read_word();
        self.process_instruction(instruction);
    }

    fn render(&mut self) {
        let mut fg_rect_vec: Vec<Rect> = Vec::new();
        let mut bg_rect_vec: Vec<Rect> = Vec::new();

        for x in 0..64 {
            for y in 0..32 {
                // println!("Loading display byte at {},{}", x, y);
                let nibble = self.display[x][y];
                if nibble {
                    fg_rect_vec.push(Rect::new_unwrap((x * 10) as i32, (y * 10) as i32, 10, 10));
                } else {
                    bg_rect_vec.push(Rect::new_unwrap((x * 10) as i32, (y * 10) as i32, 10, 10));
                }
            }
        }

        self.window.set_draw_color(Color::RGB(0, 0, 0));

        for r in bg_rect_vec {
            self.window.fill_rect(r);
        }

        self.window.set_draw_color(Color::RGB(255, 255, 255));

        for r in fg_rect_vec {
            self.window.fill_rect(r);
        }

        self.window.present();
        self.display_updated = false;
    }

    fn handle_input(&mut self) -> bool {

        for event in self.sdl_event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown {keycode: Some(Keycode::Escape), .. } => {
                    return true
                }
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
                Event::KeyDown {keycode: Some(Keycode::K), ..} => {
                    self._next_step = true;
                }
                Event::KeyDown {keycode: Some(Keycode::M), ..} => {
                    self.mem._dump_mem_to_disk();
                }
                _ => {}
            }
        }

        false
    }

    fn read_word(&mut self) -> u16 {
        let instruction_high_order = (self.mem.read_byte(self.reg.read_pc()) as u16) << 8;
        let instruction_low_order = self.mem.read_byte(self.reg.read_pc() + 1) as u16;

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
                    println!("PC: {:#x}    |    Opcode: {:#x}      |    cls",
                             self.reg.read_pc() - 2,
                             instruction);
                    for x in 0..64 {
                        for y in 0..32 {
                            self.display[x][y] = false;
                        }
                    }
                    self.display_updated = true;
                } else if operation == 0xee {
                    println!("PC: {:#x}    |    Opcode: {:#x}      |    ret",
                             self.reg.read_pc() - 2,
                             instruction);
                    self.reg.return_from_subroutine();
                }
            }
            0x1 => {
                let jump_addr = instruction & 0x0fff;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    jmp {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         jump_addr);
                self.reg.jump_to_address(jump_addr, JumpType::NORMAL);
            }
            0x2 => {
                let subroutine_addr = instruction & 0x0fff;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    call {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         subroutine_addr);
                self.reg.jump_to_address(subroutine_addr, JumpType::SUBROUTINE);
            }
            0x3 => {
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;
                let comparison_byte = (instruction & 0x00ff) as u8;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    se V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         comparison_byte);
                if self.reg.read_register(target_reg) == comparison_byte {
                    self.reg.increment_pc();
                }
            }
            0x4 => {
                let target_reg = ((instruction & 0x0f00) >> 8) as u8;
                let comparison_byte = (instruction & 0x00ff) as u8;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    sne V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         comparison_byte);
                if self.reg.read_register(target_reg) != comparison_byte {
                    self.reg.increment_pc();
                }
            }
            0x5 => {
                let reg_one = ((instruction & 0x0f00) >> 8) as u8;
                let reg_two = ((instruction & 0x00f0) >> 4) as u8;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    se V{} V{}",
                         self.reg.read_pc() - 2,
                         instruction,
                         reg_one,
                         reg_two);
                if self.reg.read_register(reg_one) == self.reg.read_register(reg_two) {
                    self.reg.increment_pc();
                }
            }
            0x6 => {
                let target_reg = ((instruction >> 8) & 0x0f) as u8;
                let data_value = (instruction & 0x00ff) as u8;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    ld V{} {:#x}",
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
                println!("PC: {:#x}    |    Opcode: {:#x}    |    add V{} {:#x}",
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld V{} V{}",
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    or V{} V{}",
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    and V{} V{}",
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    xor V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, data_value);
                    }
                    4 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);

                        let mut result: u32 = (reg_one_value as u32) + (reg_two_value as u32);

                        if result > 255 {
                            self.reg.set_vf();
                        } else {
                            self.reg.clear_vf();
                        }

                        println!("PC: {:#x}    |    Opcode: {:#x}    |    add V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, result as u8);
                    }
                    5 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);

                        if reg_one_value > reg_two_value {
                            self.reg.set_vf();
                        } else {
                            self.reg.clear_vf();
                        }

                        println!("PC: {:#x}    |    Opcode: {:#x}    |    sub V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, reg_one_value.wrapping_sub(reg_two_value));
                    }
                    6 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    shr V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);

                        if (reg_one_value & 1) == 1 {
                            self.reg.set_vf();
                        } else {
                            self.reg.clear_vf();
                        }

                        self.reg.write_register(reg_one, reg_one_value >> 1);
                    }
                    7 => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        let reg_two_value = self.reg.read_register(reg_two);

                        if reg_two_value > reg_one_value {
                            self.reg.set_vf();
                        } else {
                            self.reg.clear_vf();
                        }

                        println!("PC: {:#x}    |    Opcode: {:#x}    |    subn V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);
                        self.reg.write_register(reg_one, reg_two_value.wrapping_sub(reg_one_value));
                    }
                    0xe => {
                        let reg_one_value = self.reg.read_register(reg_one);
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    shl V{} V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 reg_one,
                                 reg_two);

                        if ((reg_one_value >> 7) & 1) == 1 {
                            self.reg.set_vf();
                        } else {
                            self.reg.clear_vf();
                        }

                        self.reg.write_register(reg_one, reg_one_value << 1);
                    }
                    _ => panic!("Unrecognized opcode: {:#x}", instruction),
                }
            }
            0x9 => {
                let reg_one = ((instruction & 0x0f00) >> 8) as u8;
                let reg_two = ((instruction & 0x00f0) >> 4) as u8;
                let reg_one_value = self.reg.read_register(reg_one);
                let reg_two_value = self.reg.read_register(reg_two);

                println!("PC: {:#x}    |    Opcode: {:#x}    |    sne V{} V{}",
                         self.reg.read_pc() - 2,
                         instruction,
                         reg_one,
                         reg_two);

                if reg_one_value != reg_two_value {
                    self.reg.increment_pc();
                }
            }
            0xa => {
                let data_value = instruction & 0x0fff;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    ld i {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         data_value);
                self.reg.write_register_i(data_value);
            }
            0xb => {
                let initial_addr = instruction & 0x0fff;
                let offset = self.reg.read_register(0) as u16;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    jp V0 {:#x}",
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
                println!("PC: {:#x}    |    Opcode: {:#x}    |    rnd V{} {:#x}",
                         self.reg.read_pc() - 2,
                         instruction,
                         target_reg,
                         combination_byte);
                println!("             |   rand_num: {:#x}   |    final byte: {:#x}",
                         rand_num,
                         combination_byte & rand_num);
            }
            0xd => {
                let reg_one = ((instruction & 0x0F00) >> 8) as u8;
                let reg_two = ((instruction & 0x00F0) >> 4) as u8;
                let num_bytes = (instruction & 0x000F) as u8;
                println!("PC: {:#x}    |    Opcode: {:#x}    |    drw V{} V{} {}",
                         self.reg.read_pc() - 2,
                         instruction,
                         reg_one,
                         reg_two,
                         num_bytes);

                let sprite_x = self.reg.read_register(reg_one);
                let sprite_y = self.reg.read_register(reg_two);
                println!("Sprite X: {}  |  Sprite Y: {}", sprite_x, sprite_y);
                let mut bit_vec: Vec<u8> = Vec::new();
                for i in 0..num_bytes {
                    bit_vec.push(self.mem.read_byte(self.reg.read_register_i() + (i as u16)));
                }

                println!("Glyph:");
                for byte in bit_vec.clone() {
                    println!("{:#8b}", byte);
                }
                println!("");

                self.reg.clear_vf();

                let mut y_index = sprite_y as usize;
                let mut x_value = sprite_x as usize;
                for byte in bit_vec.clone() {

                    for i in 0..8 {
                        let mut x_index = x_value + (7 - i);
                        if x_index > 63 {
                            x_index = 69 - x_value;
                        }
                        if y_index > 31 {
                            y_index = y_index - 32;
                        }

                        let mut bit_state: bool = false;
                        if (byte >> i) & 1 == 1 {
                            bit_state = true;
                        }

                        if bit_state != self.display[x_index][y_index] {
                            self.display[x_index][y_index] = true;
                        } else {
                            if self.display[x_index][y_index] == true {
                                self.reg.set_vf();
                            }

                            self.display[x_index][y_index] = false;
                        }
                    }

                    y_index += 1;
                }

                self.display_updated = true;
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    skp V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 target_reg);
                    }
                    0xa1 => {
                        let key = self.reg.read_register(target_reg);
                        if self.keys.keys[key as usize] == false {
                            self.reg.increment_pc();
                        }
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    sknp V{}",
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
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld V{} DT",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let reg_value = self.reg.read_delay_timer();
                        self.reg.write_register(register_index, reg_value);
                    }
                    0x15 => {
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld DT V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let reg_value = self.reg.read_register(register_index);
                        self.reg.write_delay_timer(reg_value);
                    }
                    0x18 => {
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld ST V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        let reg_value = self.reg.read_register(register_index);
                        self.reg.write_sound_timer(reg_value);
                    }
                    0x1e => {
                        let reg_value = self.reg.read_register(register_index);
                        let i_value = self.reg.read_register_i();

                        println!("PC: {:#x}    |    Opcode: {:#x}    |    add I V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                        self.reg.write_register_i((reg_value as u16) + i_value);
                    }
                    0x29 => {
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld F V{}",
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

                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld B V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);

                        self.mem.write_byte(self.reg.read_register_i(), hundreds_digit);
                        self.mem.write_byte(self.reg.read_register_i() + 1, tens_digit);
                        self.mem.write_byte(self.reg.read_register_i() + 2, ones_digit);
                    }
                    0x55 => {
                        let num_reg = register_index as usize;
                        let mut mem_addr = self.reg.read_register_i();
                        for n in 0..num_reg {
                            self.mem
                                .write_byte(mem_addr + (n as u16), self.reg.read_register(n as u8));
                        }
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld [I] V{}",
                                 self.reg.read_pc() - 2,
                                 instruction,
                                 register_index);
                    }
                    0x65 => {
                        println!("PC: {:#x}    |    Opcode: {:#x}    |    ld V{} [I]",
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
                        panic!("PC: {:#x}    |    Opcode: {:#x}    |    various",
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

pub enum JumpType {
    NORMAL,
    SUBROUTINE,
}
