use crate::core::emulator::Emulator;
use crate::shared::data::bit::BitManipulation;
use anyhow::{anyhow, Error};
use rand::Rng;
use tracing::{debug, error, info};

use super::{
    chip8::{SCREEN_HEIGHT, SCREEN_WIDTH},
    instruction::Instruction,
};

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
        emulator: &Emulator,
        cycles_per_frame: u32,
        bit_shift_instructions_use_vy: bool,
        store_read_instructions_change_i: bool,
    ) -> Result<Self, Error> {
        // Attempt to get the program counter (PC) and read two bytes
        let lower_addr = emulator.get_pc() as usize;
        let ram = emulator.get_ram();

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

    fn exec_instruction(&self, emulator: &mut Emulator) -> Result<(), anyhow::Error> {
        let first_nibble = self.first_nibble();
        let x = self.x();
        let y = self.y();
        let fourth_nibble = self.fourth_nibble();
        let addr = self.extract_12bit_address();

        match first_nibble {
            0x0 => match self.word {
                0x0000 => {
                    debug!("NOP executed: No operation performed.");
                    Instruction::Nop.execute(emulator)
                }
                0x00E0 => {
                    debug!("Screen cleared!");
                    Instruction::Cls.execute(emulator)
                }
                0x00EE => {
                    debug!("Returned from subroutine!");
                    Instruction::Ret.execute(emulator)
                }
                _ => {
                    error!("Unsupported instruction: {:#04x}", self.word);
                    Err(anyhow::anyhow!("Unsupported instruction"))
                }
            },
            0x1 => {
                debug!("Jump to address: {:#04x}", self.extract_12bit_address());
                Instruction::Jmp(addr).execute(emulator)
            }
            0x2 => {
                debug!(
                    "Call subroutine at address: {:#04x}",
                    self.extract_12bit_address()
                );
                Instruction::Call(addr).execute(emulator)
            }

            _ => Err(anyhow::anyhow!("Unsupported instruction")),
        }
    }
}
