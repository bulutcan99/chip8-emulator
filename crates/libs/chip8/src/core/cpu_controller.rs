use crate::core::memory_controller::MemoryController;
use crate::shared::data::bit::BitManipulation;
use anyhow::{anyhow, Error};
use tracing::{error, info};

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
	) -> Result<Self, Error> {
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

	fn extract_12bit_address(&self) -> u16 {
		let x = self.x();
		let y = self.y();
		let fourth = self.fourth_nibble();
		BitManipulation::combine_nibbles_to_16bit_address(x, y, fourth)
	}

	fn skip_next_instruction(&self, mem_ctr: &mut MemoryController) {
		mem_ctr.inc_pc_by(2)
	}

	// **INSTRUCTIONS**

	// 0000 - NOP
	fn no_operation(&self, mem_ctrl: &mut MemoryController) {
		info!("NOP executed: No operation performed.");
	}

	// 00E0 - CLS -> will implement after sdl

	// 00EE - RET (Return from Subroutine)
	fn return_from_subroutine(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		// Return to the previous function by popping the stack.
		mem_ctr.stack_pop().map_err(|err| {
			error!("Failed to return from subroutine: {:?}", err);
			err
		})
	}

	// 1NNN - JMP NNN (Jump to Address NNN)
	fn jump_to_address(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let address = self.extract_12bit_address();
		mem_ctr.set_pc(address);
		Ok(())
	}

	// 2NNN - CALL NNN (Call Address NNN)
	fn call_address(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let address = self.extract_12bit_address();
		match mem_ctr.stack_push(mem_ctr.get_pc()) {
			Ok(_) => {
				mem_ctr.set_pc(address);
				Ok(())
			}
			Err(err) => {
				error!("Failed to call subroutine at address: {:X}, error: {:?}", address, err);
				Err(err)
			}
		}
	}

	// 3XNN - SE VX
	fn skip_equal_vx_byte(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let v = mem_ctr.get_v(x)?;
		let second_byte = self.second_byte();
		if v == second_byte {
			self.skip_next_instruction(mem_ctr)
		}
		Ok(())
	}

	// 4XNN - SNE VX
	fn skip_not_equal_vx_byte(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let v = mem_ctr.get_v(x)?;
		let second_byte = self.second_byte();
		if v != second_byte {
			self.skip_next_instruction(mem_ctr)
		}
		Ok(())
	}

	// 5XY0 - SE VX == VY
	fn skip_equal_vx_vy(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let vx = mem_ctr.get_v(x)?;
		let y = self.x();
		let vy = mem_ctr.get_v(y)?;
		if vx == vy {
			self.skip_next_instruction(mem_ctr)
		}
		Ok(())
	}

	// 6XNN - LD VX with byte
	fn load_vx_with_byte(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let second_byte = self.second_byte();
		mem_ctr.set_v(x, second_byte)
	}

	// 7XNN - ADD VX, NN
	fn add_vx_with_byte(&mut self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let nn = self.second_byte();

		let vx = mem_ctr.get_v(x)?;
		let result = vx.wrapping_add(nn);

		mem_ctr.set_v(x, result)?;

		Ok(())
	}

	// 8XY0 - LD VX with VY
	fn load_vx_with_vy(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.x();
		let vy = mem_ctr.get_v(y)?;
		mem_ctr.set_v(x, vy)?;
		Ok(())
	}

	// 8XY1 - LD VX with (VX or VY)
	fn load_vx_with_vx_or_vy(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.y();
		let vx = mem_ctr.get_v(x)?;
		let vy = mem_ctr.get_v(y)?;
		mem_ctr.set_v(x, vx | vy)?;
		Ok(())
	}

	// 8XY2 - LD VX with (VX and VY)
	fn load_vx_with_vx_and_vy(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.y();
		let vx = mem_ctr.get_v(x)?;
		let vy = mem_ctr.get_v(y)?;
		mem_ctr.set_v(x, vx & vy)?;
		Ok(())
	}

	// 8XY3 - LD VX with (VX xor VY)
	fn load_vx_with_vx_xor_vy(&self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.y();
		let vx = mem_ctr.get_v(x)?;
		let vy = mem_ctr.get_v(y)?;
		mem_ctr.set_v(x, vx ^ vy)?;
		Ok(())
	}

	// 8XY4 - ADD VX, VY
	fn add_vx_with_vy(&mut self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.y();

		let vx = mem_ctr.get_v(x)?;
		let vy = mem_ctr.get_v(y)?;

		let (result, overflow) = vx.overflowing_add(vy);

		if overflow {
			mem_ctr.set_v(0xF, 1)?
		} else {
			mem_ctr.set_v(0xF, 0)?;
		}

		mem_ctr.set_v(x, result)?;

		Ok(())
	}

	// 8XY5 - SUB VX, VY
	fn sub_vx_to_vy(&mut self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let y = self.y();

		let vx = mem_ctr.get_v(x)?;
		let vy = mem_ctr.get_v(y)?;

		let (result, overflow) = vx.overflowing_sub(vy);

		if overflow {
			mem_ctr.set_v(0xF, 0)?;
		} else {
			mem_ctr.set_v(0xF, 1)?;
		}

		mem_ctr.set_v(x, result)?;

		Ok(())
	}

	// 8XY6 - SHIFT VX (to the right) by 1
	fn shift_vx_to_right(&mut self, mem_ctr: &mut MemoryController) -> Result<(), Error> {
		let x = self.x();
		let vx = mem_ctr.get_v(x)?;
		let lsb = vx & 0b0000_0001;

		mem_ctr.set_v(0xF, lsb)?;
		let result = vx >> 1;
		mem_ctr.set_v(x, result)?;

		Ok(())
	}
}
