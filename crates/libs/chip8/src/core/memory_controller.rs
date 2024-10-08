use crate::memory::Memory;
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

	pub fn init_ram(&mut self, rom_path: &str) {
		self.load_rom_file(rom_path);
		self.load_hex_digits();
	}

	pub fn get_ram(&self) -> [u8; 4096] {
		self.memory.ram
	}

	pub fn set_ram(&mut self, index: usize, val: u8) {
		self.memory.ram[index] = val;
	}

	pub fn get_v(&mut self, index: u8) -> u8 {
		match index {
			0 => self.memory.v_reg[0],
			1 => self.memory.v_reg[1],
			2 => self.memory.v_reg[2],
			3 => self.memory.v_reg[3],
			4 => self.memory.v_reg[4],
			5 => self.memory.v_reg[5],
			6 => self.memory.v_reg[6],
			7 => self.memory.v_reg[7],
			8 => self.memory.v_reg[8],
			9 => self.memory.v_reg[9],
			0xa => self.memory.v_reg[10],
			0xb => self.memory.v_reg[11],
			0xc => self.memory.v_reg[12],
			0xd => self.memory.v_reg[13],
			0xe => self.memory.v_reg[14],
			0xf => self.memory.v_reg[15],
			_ => 0
		}
	}

	pub fn set_v(&mut self, index: u8, val: u8) {
		match index {
			0 => self.memory.v_reg[0] = val,
			1 => self.memory.v_reg[1] = val,
			2 => self.memory.v_reg[2] = val,
			3 => self.memory.v_reg[3] = val,
			4 => self.memory.v_reg[4] = val,
			5 => self.memory.v_reg[5] = val,
			6 => self.memory.v_reg[6] = val,
			7 => self.memory.v_reg[7] = val,
			8 => self.memory.v_reg[8] = val,
			9 => self.memory.v_reg[9] = val,
			0xa => self.memory.v_reg[10] = val,
			0xb => self.memory.v_reg[11] = val,
			0xc => self.memory.v_reg[12] = val,
			0xd => self.memory.v_reg[13] = val,
			0xe => self.memory.v_reg[14] = val,
			0xf => self.memory.v_reg[15] = val,
			_ => ()
		}
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

	pub fn stack_pop(&mut self) {
		self.memory.pc = self.memory.stack[(self.memory.sp - 1) as usize];
		self.memory.stack[(self.memory.sp - 1) as usize] = 0;
		self.memory.sp -= 1;
	}

	pub fn stack_push(&mut self, new_pc_addr: u16) {
		self.memory.sp += 1;
		self.memory.stack[(self.memory.sp - 1) as usize] = self.memory.pc;
		self.memory.pc = new_pc_addr
	}

	fn load_hex_digits(&mut self) {
		for i in 0..HEX_DIGITS.len() {
			self.memory.ram[i] = HEX_DIGITS[i];
		}
	}

	fn load_rom_file(&mut self, path: &str) {
		let mut byte_vec: Vec<u8> = Vec::new();
		File::open(path).unwrap().read_to_end(&mut byte_vec).unwrap();
		// 4096 (RAM size) - 512 (Reserved RAM)
		if byte_vec.len() > 3584 {
			panic!("The selected ROM size will overflow beyond the limit of RAM!")
		}
	}
}