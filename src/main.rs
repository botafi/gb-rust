#![feature(is_some_with)]

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

struct MMU<'a> {
    booted: bool,
    // [0000-00FF] bios during boot
    bios: [u8; 256],

    // [0000-3FFF] cartridge bank0 after boot
    // [0100-014F] cartridge header
    bank0: &'a [u8],

    // [4000-7FFF] cartridge other banks
    loaded_bank: &'a [u8],

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

impl Default for MMU<'_> {
    fn default() -> Self {
        MMU {
            booted: false,
            bios: [0; 256],
            bank0: &[0; 16384],
            loaded_bank: &[0; 16384],
            graphics: [0; 8192],
            external_ram: [0; 8192],
            ram: [0; 8192],
            sprites: [0; 160],
            io: [0; 128],
            work_ram: [0; 128],
        }
    }
}

impl<'a> MMU<'a> {
    fn new() -> Self {
        Default::default()
    }
    fn rb(&self, addr: u16) -> u8 {
        match addr {
            // bank 0 & bios
            0x000..=0x00ff => match self.booted {
                false => self.bios[(addr - 0x000) as usize],
                true => self.bank0[(addr - 0x000) as usize],
            },
            0x0100..=0x3fff => self.bank0[(addr - 0x000) as usize],

            0x4000..=0x7fff => self.loaded_bank[(addr - 0x4000) as usize],

            0x8000..=0x9fff => self.graphics[(addr - 0x8000) as usize],

            0xa000..=0xbfff => self.external_ram[(addr - 0xa000) as usize],

            0xc000..=0xfdff => self.ram[(addr % 8192) as usize],

            0xfe00..=0xfe9f => self.sprites[(addr - 0xfe00) as usize],

            0xfea0..=0xfeff => panic!("Trying to read non-existent memory"),

            0xff00..=0xff7f => self.io[(addr - 0xff00) as usize],

            0xff80..=0xffff => self.work_ram[(addr - 0xff80) as usize],
        }
    }
    fn r2b(&self, addr: u16) -> u16 {
        let head = self.rb(addr) as u16;
        let tail = self.rb(addr + 1) as u16;
        (head << 8) | tail 
    }
    fn wb(&mut self, addr: u16, val: u8) {
        match addr {
            // bank 0 & bios
            0x000..=0x00ff => panic!("Trying to write to non-writable memory - bios / bank 0"),
            0x0100..=0x3fff => panic!("Trying to write to non-writable memory - bank 0"),

            0x4000..=0x7fff => panic!("Trying to write to non-writable memory - loaded bank"),

            0x8000..=0x9fff => self.graphics[(addr - 0x8000) as usize] = val,

            0xa000..=0xbfff => self.external_ram[(addr - 0xa000) as usize] = val,

            0xc000..=0xfdff => self.ram[(addr % 8192) as usize] = val,

            0xfe00..=0xfe9f => self.sprites[(addr - 0xfe00) as usize] = val,

            0xfea0..=0xfeff => panic!("Trying to write non-existent memory"),

            0xff00..=0xff7f => self.io[(addr - 0xff00) as usize] = val,

            0xff80..=0xffff => self.work_ram[(addr - 0xff80) as usize] = val,
        }
    }
}

#[derive(Default)]
struct Z80 {
    // clock for last istr
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
    pc: u16,  // program counter
    sp: u16,  // stack pointer
}

impl Z80 {
    fn new() -> Self {
        Default::default()
    }
}

struct GB<'a> {
    z80: Z80,
    mmu: MMU<'a>,
    clockM: u64,
    clockT: u64,
    rom_data: &'a Vec<u8>,
}

impl<'a> GB<'a> {
    fn new(rom_data: &'a Vec<u8>) -> Self {
        let mut instance = Self {
            z80: Default::default(),
            mmu: Default::default(),
            clockM: Default::default(),
            clockT: Default::default(),
            rom_data
        };
        instance.mmu.bank0 = &rom_data[0..16384];
        instance
    }

    fn load_rom(&mut self, rom_data: &'a Vec<u8>) {
        self.rom_data = rom_data;
        self.mmu.bank0 = &rom_data[0..16384]
    }

    fn cycle(&mut self) {
        let instr = self.mmu.rb(self.z80.pc);
        self.run_instr(instr);
        self.clockM += self.z80.m as u64;
        self.clockT += self.z80.t as u64;
    }

    fn run_instr(&mut self, instr: u8) {
        match instr {
            // NOP
            0x00 => {
                self.z80.m = 1;
                self.z80.t = 4;
            },
            // LD ** BC
            0x01 => {
                self.z80.c = self.mmu.rb(self.z80.pc);
                self.z80.b = self.mmu.rb(self.z80.pc + 1);
                self.z80.m = 3;
                self.z80.t = 12;
            },
            _ => todo!("Instruction not implemented"),
        }
        self.z80.pc += 1;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(!args.is_empty(), "Expected path to ROM");
    let rom_data_result = fs::read(args.first().unwrap());
    assert!(
        rom_data_result.is_ok_and(|r| r.len() > 0x014f),
        "Expected file to exist and have data"
    );
    let rom_data = rom_data_result.unwrap();
    let mut gb = GB::new(&rom_data);
    loop {
        gb.cycle()
    }
}
