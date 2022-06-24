use std::env;
use std::fs;

extern crate bitflags;


bitflags::bitflags! {
    struct Flags: u8 {
        const NONE = 0x00;
        const CARRY = 0x10;
        const HALF_CARRY = 0x20;
        const SUBSTRACTION = 0x40;
        const ZERO = 0x80;
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags::NONE
    }
}

struct MMU {
    booted: bool,
    // [0000-00FF] bios during boot
    bios: [u8; 256],

    // [0000-3FFF] cartridge bank0 after boot
    // [0100-014F] cartridge header
    bank0: [u8; 16384],

    // [4000-7FFF] cartridge other banks
    loaded_bank: [u8; 16384],

    // [8000-9FFF] graphics
    graphics: [u8; 8192],

    // [A000-BFFF] external cartridge ram
    external_ram: [u8; 8192],

    // [C000-DFFF] (+ repeat at [E000-FDFF]) internal working ram
    ram: [u8; 8192],

    // [FE00-FE9F] sprites
    sprites: [u8; 160],

    // [FF00-FF7F] IO
    io: [u8; 128],

    // [FF80-FFFF] 
    work_ram: [u8; 128],
}

impl Default for MMU {
    fn default() -> Self {
        MMU {
            booted: false,
            bios: [0;256],
            bank0: [0;16384],
            loaded_bank: [0;16384],
            graphics: [0;8192],
            external_ram: [0;8192],
            ram: [0;8192],
            sprites: [0;160],
            io: [0;128],
            work_ram: [0;128],
        }
    }
}

impl MMU {
    fn new() -> Self {
        Default::default()
    }
    fn rb(&self, addr: u16) -> u8 {
        match addr {
            // bank 0 & bios
            0x000..=0x00ff => match self.booted {
                false => self.bios[(addr-0x000) as usize],
                true => self.bank0[(addr-0x000) as usize]
            },
            0x0100..=0x3fff => self.bank0[(addr-0x000) as usize],

            0x4000..=0x7fff => self.loaded_bank[(addr-0x4000) as usize],

            0x8000..=0x9fff => self.graphics[(addr-0x8000) as usize],

            0xa000..=0xbfff => self.external_ram[(addr-0xa000) as usize],

            0xc000..=0xfdff => self.ram[(addr % 8192) as usize],

            0xfe00..=0xfe9f => self.sprites[(addr-0xfe00) as usize],

            0xfea0..=0xfeff => panic!("Trying to read unreadable memory"),

            0xff00..=0xff7f => self.io[(addr-0xff00) as usize],

            0xff80..=0xffff => self.work_ram[(addr-0xff80) as usize],
        }
    }
    fn wb(&mut self, addr: u16, val: u8) {
        match addr {
            // bank 0 & bios
            0x000..=0x00ff => match self.booted {
                false => self.bios[(addr-0x000) as usize] = val,
                true => self.bank0[(addr-0x000) as usize] = val
            },
            0x0100..=0x3fff => self.bank0[(addr-0x000) as usize] = val,

            0x4000..=0x7fff => self.loaded_bank[(addr-0x4000) as usize] = val,

            0x8000..=0x9fff => self.graphics[(addr-0x8000) as usize] = val,

            0xa000..=0xbfff => self.external_ram[(addr-0xa000) as usize] = val,

            0xc000..=0xfdff => self.ram[(addr % 8192) as usize] = val,

            0xfe00..=0xfe9f => self.sprites[(addr-0xfe00) as usize] = val,

            0xfea0..=0xfeff => panic!("Trying to write unwritable memory"),

            0xff00..=0xff7f => self.io[(addr-0xff00) as usize] = val,

            0xff80..=0xffff => self.work_ram[(addr-0xff80) as usize] = val,
        }
    }
}

#[derive(Default)]
struct Z80 {
    // clock for last i
    m: u8,
    t: u8,
    // registers
    b: u8,
    a: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    // special registers
    f: Flags, // flags
    pc: u16, // program counter
    sp: u16, // stack pointer
}

impl Z80 {
    fn new() -> Self {
        Default::default()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(!args.is_empty(), "Expected path to ROM");
    let rom_data = fs::read(args.first().unwrap());
    let mut mmu = MMU::new();
    let mut z80 = Z80::new();
    let mut clockM: u64 = 0;
    let mut clockT: u64 = 0;
    loop {
        let instr = mmu.rb(z80.pc);
        z80.pc += 1;
        clockM += z80.m as u64;
        clockT += z80.t as u64;
    }
}
