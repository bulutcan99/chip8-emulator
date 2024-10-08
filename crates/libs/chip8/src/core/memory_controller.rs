use crate::core::memory::Memory;
use anyhow::{anyhow, Error};
use std::fs::File;
use std::io::Read;

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
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct MemoryController {
	memory: Memory,
}

impl MemoryController {
	pub fn new(mem: Memory) -> Self {
		Self {
			memory: mem,
		}
	}

	pub fn init_ram(&mut self, rom_path: &str) -> Result<(), Error> {
		self.load_rom_file(rom_path)?;
		self.load_hex_digits()?;
		Ok(())
	}

	pub fn get_ram(&self) -> [u8; 4096] {
		self.memory.ram
	}

	pub fn set_to_ram(&mut self, index: usize, val: u8) -> Result<(), Error> {
		if index >= self.memory.ram.len() {
			return Err(anyhow!("Index out of bounds for RAM!"));
		}
		self.memory.ram[index] = val;
		Ok(())
	}


	pub fn get_v(&self, index: u8) -> Result<u8, Error> {
		if index > 0xF {
			return Err(anyhow!("Index out of bounds for V register!"));
		}
		Ok(self.memory.v_reg[index as usize])
	}

	pub fn set_v(&mut self, index: u8, val: u8) -> Result<(), Error> {
		if index > 0xF {
			return Err(anyhow!("Index out of bounds for V register!"));
		}
		self.memory.v_reg[index as usize] = val;
		Ok(())
	}

	pub fn get_dt(&self) -> u8 {
		self.memory.dt
	}

	pub fn set_dt(&mut self, val: u8) {
		self.memory.dt = val;
	}

	pub fn dec_dt(&mut self) {
		if self.memory.dt > 0 {
			self.memory.dt -= 1;
		}
	}

	pub fn get_st(&self) -> u8 {
		self.memory.st
	}

	pub fn set_st(&mut self, val: u8) {
		self.memory.st = val;
	}

	pub fn dec_st(&mut self) {
		if self.memory.st > 0 {
			self.memory.st -= 1;
		}
	}

	pub fn dec_all_timers(&mut self) {
		self.dec_dt();
		self.dec_st();
	}

	pub fn get_pc(&self) -> u16 {
		self.memory.pc
	}

	pub fn set_pc(&mut self, val: u16) {
		self.memory.pc = val;
	}

	pub fn inc_pc_by(&mut self, val: u16) {
		self.memory.pc += val;
	}

	pub fn get_i(&self) -> u16 {
		self.memory.i_reg
	}

	pub fn set_i(&mut self, val: u16) {
		self.memory.i_reg = val;
	}

	pub fn inc_i_by(&mut self, val: u16) {
		self.memory.i_reg += val;
	}


	pub fn stack_pop(&mut self) -> Result<(), Error> {
		if self.memory.sp == 0 {
			return Err(anyhow!("Stack underflow: No more elements to pop!"));
		}

		self.memory.pc = self.memory.stack[(self.memory.sp - 1) as usize];
		self.memory.stack[(self.memory.sp - 1) as usize] = 0;
		self.memory.sp -= 1;

		Ok(())
	}

	pub fn stack_push(&mut self, new_pc_addr: u16) -> Result<(), Error> {
		if self.memory.sp >= self.memory.stack.len() as u8 {
			return Err(anyhow!("Stack overflow: No more space to push new element!"));
		}

		self.memory.sp += 1;
		self.memory.stack[(self.memory.sp - 1) as usize] = self.memory.pc;
		self.memory.pc = new_pc_addr;

		Ok(())
	}

	pub fn load_hex_digits(&mut self) -> Result<(), Error> {
		if HEX_DIGITS.len() > self.memory.ram.len() {
			return Err(anyhow!("HEX_DIGITS exceeds RAM size!"));
		}

		for i in 0..HEX_DIGITS.len() {
			self.memory.ram[i] = HEX_DIGITS[i];
		}

		Ok(())
	}

	fn load_rom_file(&mut self, path: &str) -> Result<(), Error> {
		let mut byte_vec: Vec<u8> = Vec::new();
		File::open(path)?.read_to_end(&mut byte_vec)?;

		// 4096 (RAM size) - 512 (Reserved RAM)
		if byte_vec.len() > 3584 {
			return Err(anyhow!("The selected ROM size will overflow beyond the limit of RAM!"));
		}

		Ok(())
	}
}