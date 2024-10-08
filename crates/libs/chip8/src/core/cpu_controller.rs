use crate::core::memory_controller::MemoryController;
use crate::shared::data::bit::BitManipulation;
use tracing::info;

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
		store_read_instructions_change_i: bool,
	) -> Result<Self> {
		// Attempt to get the program counter (PC) and read two bytes
		let lower_addr = mem_ctrl.get_pc() as usize;
		let ram = mem_ctrl.get_ram();

		// Check if we can read the instruction bytes
		if lower_addr + 1 >= ram.len() {
			error!("Failed to read instruction bytes: Address out of bounds");
			return Err(anyhow!("Address out of bounds for instruction read!"));
		}

		let first_byte = ram[lower_addr];
		let second_byte = ram[lower_addr + 1];

		let word = BitManipulation::combine_bytes_to_16bit_instruction(first_byte, second_byte);

		info!("CPU initialized with instruction word: {:#04x}", word);

		Ok(Self {
			word,
			inc_pc: true,
			cycles_per_frame,
			bit_shift_instructions_use_vy,
			store_read_instructions_change_i,
		})
	}

	//  [xxxx xxxx 0000 0000]
	pub fn first_byte(&self) -> u8 {
		(self.word >> 8) as u8
	}

	// [0000 0000 xxxx xxxx]
	pub fn second_byte(&self) -> u8 {
		self.word as u8
	}

	// [xxxx 0000 0000 0000]

	pub fn first_nibble(&self) -> u8 {
		(self.word >> 12) as u8
	}

	// [0000 xxxx 0000 0000]
	pub fn x(&self) -> u8 {
		((self.word >> 8) & 0x0F) as u8
	}

	// [0000 0000 xxxx 0000]
	pub fn y(&self) -> u8 {
		((self.word >> 4) & 0x0F) as u8
	}

	// [0000 0000 0000 xxxx]
	pub fn fourth_nibble(&self) -> u8 {
		(self.word & 0x0F) as u8
	}

	pub fn get_cyples_per_frame(&self) -> u32 {
		self.cycles_per_frame
	}

	// **INSTRUCTIONS**

	// 0000 - NOP
	fn no_operation(&self, mem_ctrl: &mut MemoryController) {
		info!("NOP executed: No operation performed.");
	}

	// 00E0 - CLS -> will implement after sdl

	// 00EE - RET
	fn return_from_subroutine(&self, mem_ctr: &mut MemoryController) {
		// Works like return -> returned to the upper func.
		mem_ctr.stack_pop().unwrap()
	}
}