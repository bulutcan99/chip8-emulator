use crate::core::chip8::{CHIP8, SCREEN_HEIGHT, SCREEN_WIDTH};
use anyhow::{anyhow, Error};
use std::fs::File;
use std::io::Read;
use tracing::{error, info};

const HEX_DIGITS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator {
    chip8: CHIP8,
}

impl Emulator {
    pub fn new(chip8: CHIP8) -> Self {
        Self { chip8 }
    }

    pub fn init_ram(&mut self, rom_path: &str) -> Result<(), Error> {
        info!("Initializing RAM with ROM file: {}", rom_path);
        self.load_rom_file(rom_path)?;
        self.load_hex_digits()?;
        Ok(())
    }

    pub fn get_ram(&self) -> [u8; 4096] {
        self.chip8.ram
    }

    pub fn set_to_ram(&mut self, index: usize, val: u8) -> Result<(), Error> {
        if index >= self.chip8.ram.len() {
            error!("Index out of bounds for RAM!");
            return Err(anyhow!("Index out of bounds for RAM!"));
        }
        self.chip8.ram[index] = val;
        Ok(())
    }

    pub fn get_v(&self, index: u8) -> Result<u8, Error> {
        if index > 0xF {
            error!("Index out of range while getting V-Reg");
            return Err(anyhow!("Index out of bounds for V register!"));
        }
        Ok(self.chip8.v_reg[index as usize])
    }

    pub fn set_v(&mut self, index: u8, val: u8) -> Result<(), Error> {
        if index > 0xF {
            error!("Index out of range while setting V-Reg");
            return Err(anyhow!("Index out of bounds for V register!"));
        }
        self.chip8.v_reg[index as usize] = val;
        Ok(())
    }

    pub fn get_dt(&self) -> u8 {
        self.chip8.dt
    }

    pub fn set_dt(&mut self, val: u8) {
        self.chip8.dt = val;
    }

    pub fn dec_dt(&mut self) {
        if self.chip8.dt > 0 {
            self.chip8.dt -= 1;
        }
    }

    pub fn get_st(&self) -> u8 {
        self.chip8.st
    }

    pub fn set_st(&mut self, val: u8) {
        self.chip8.st = val;
    }

    pub fn dec_st(&mut self) {
        if self.chip8.st > 0 {
            self.chip8.st -= 1;
        }
    }

    pub fn dec_all_timers(&mut self) {
        self.dec_dt();
        self.dec_st();
    }

    pub fn get_pc(&self) -> u16 {
        self.chip8.pc
    }

    pub fn set_pc(&mut self, val: u16) {
        self.chip8.pc = val;
    }

    pub fn inc_pc_by(&mut self, val: u16) {
        self.chip8.pc += val;
    }

    pub fn dec_pc_by(&mut self, val: u16) {
        if self.chip8.pc > 0 {
            self.chip8.pc -= val;
        }
    }

    pub fn get_i(&self) -> u16 {
        self.chip8.i_reg
    }

    pub fn set_i(&mut self, val: u16) {
        self.chip8.i_reg = val;
    }

    pub fn inc_i_by(&mut self, val: u16) {
        self.chip8.i_reg += val;
    }

    pub fn stack_pop(&mut self) -> Result<(), Error> {
        if self.chip8.sp == 0 {
            error!("Stack underflowed!");
            return Err(anyhow!("Stack underflow: No more elements to pop!"));
        }

        self.chip8.pc = self.chip8.stack[(self.chip8.sp - 1) as usize];
        self.chip8.stack[(self.chip8.sp - 1) as usize] = 0;
        self.chip8.sp -= 1;

        Ok(())
    }

    pub fn stack_push(&mut self, new_pc_addr: u16) -> Result<(), Error> {
        if self.chip8.sp >= self.chip8.stack.len() as u8 {
            return Err(anyhow!(
                "Stack overflow: No more space to push new element!"
            ));
        }

        self.chip8.sp += 1;
        self.chip8.stack[(self.chip8.sp - 1) as usize] = self.chip8.pc;
        self.chip8.pc = new_pc_addr;

        Ok(())
    }

    pub fn load_hex_digits(&mut self) -> Result<(), Error> {
        info!("Loading HEX_DIGITS into RAM");
        if HEX_DIGITS.len() > self.chip8.ram.len() {
            error!("HEX_DIGITS exceeds RAM size!");
            return Err(anyhow!("HEX_DIGITS exceeds RAM size!"));
        }

        for i in 0..HEX_DIGITS.len() {
            self.chip8.ram[i] = HEX_DIGITS[i];
        }

        Ok(())
    }

    fn load_rom_file(&mut self, path: &str) -> Result<(), Error> {
        info!("Loading ROM file from path: {}", path);
        let mut byte_vec: Vec<u8> = Vec::new();
        File::open(path)
            .and_then(|mut file| file.read_to_end(&mut byte_vec))
            .map_err(|e| {
                error!("Failed to read ROM file: {}", e);
                anyhow!("Failed to read ROM file: {}", e)
            })?;

        // 4096 (RAM size) - 512 (Reserved RAM)
        if byte_vec.len() > 3584 {
            error!("The selected ROM size will overflow beyond the limit of RAM!");
            return Err(anyhow!(
                "The selected ROM size will overflow beyond the limit of RAM!"
            ));
        }

        Ok(())
    }
    pub fn get_display(&self) -> [bool; SCREEN_WIDTH * SCREEN_HEIGHT] {
        self.chip8.display
    }

    pub fn clear_screen(&mut self) {
        self.chip8.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
    }

    pub fn is_key_pressed(&self, idx: u8) -> bool {
        self.chip8.keys[idx as usize]
    }

    pub fn check_key_press(&self) -> Option<u8> {
        for i in 0..16 {
            if self.chip8.keys[i] {
                return Some(i as u8);
            }
        }
        None
    }
}
