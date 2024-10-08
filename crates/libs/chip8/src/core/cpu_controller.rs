use crate::core::memory_controller::MemoryController;
use crate::shared::data::bit::BitManipulation;

enum CpuState {
	Halted,
	NotHalted,
}

pub struct CpuController {
	// The 16-bit word representing an instruction (combination of two 8-bit bytes).
	word: u16,

	// Flag to indicate whether the program counter (PC) should be incremented.
	inc_pc: bool,

	// Number of cycles per frame, used to control how many CPU cycles should be executed within one frame.
	cycles_per_frame: u32,

	// Determines whether bit shift instructions should use the value of register VY.
	// If true, shift instructions will involve register VY, otherwise they will use VX.
	bit_shift_instructions_use_vy: bool,

	// Determines whether store/read instructions modify the I-Register (index register).
	// If true, I register will be modified by store and read instructions.
	store_read_instructions_change_i: bool,
}


impl CpuController {
	pub fn new(
		mem_ctrl: &MemoryController,
		cycles_per_frame: u32,
		bit_shift_instructions_use_vy: bool,
		store_read_instructions_change_i: bool) -> Self {
		let lower_addr = mem_ctrl.get_pc() as usize;
		let first_byte = mem_ctrl.get_ram()[lower_addr];
		let second_byte = mem_ctrl.get_ram()[lower_addr + 1];

		let word = BitManipulation::combine_bytes_to_16bit_instruction(first_byte, second_byte);

		Self {
			word,
			inc_pc: true,
			cycles_per_frame,
			bit_shift_instructions_use_vy,
			store_read_instructions_change_i,
		}
	}

	pub fn first_byte(&self) -> u8 {
		(self.word >> 8) as u8
	}

	pub fn second_byte(&self) -> u8 {
		self.word as u8
	}

	pub fn first_nibble(&self) -> u8 {
		(self.word >> 12) as u8
	}

	pub fn x(&self) -> u8 {
		((self.word >> 8) & 0x0F) as u8
	}

	pub fn y(&self) -> u8 {
		((self.word >> 4) & 0x0F) as u8
	}

	pub fn fourth_nibble(&self) -> u8 {
		(self.word & 0x0F) as u8
	}
}
