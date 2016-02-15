use std::fs::File;
use std::fmt;
use std::io::Read;
use std::io::Write;

const MEM_SIZE: usize = 4096;
const ROM_ADDR: usize = 0x200;

pub struct Memory {
    pub mem: [u8; MEM_SIZE],
}

impl Memory {
    pub fn store_program_data(&mut self, rom: File) {
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

    pub fn load_fonts(&mut self) {
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

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    pub fn write_byte(&mut self, address: u16, new_byte: u8) {
        self.mem[address as usize] = new_byte;
    }

    pub fn _dump_mem_to_disk(&self) {
        let mut out = File::create("./memdump.dmp").unwrap();
        out.write_all(&self.mem);
        println!("Dumped memory to disk.");
    }

    pub fn _display_pong_rom(&self) {
        let mut addr = ROM_ADDR;
        for _ in 1..100 {
            println!("{:#x}", self.mem[addr]);
            addr += 1;
        }
    }

    pub fn _display_font_data(&self) {
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
