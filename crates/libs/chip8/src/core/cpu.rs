use crate::core::emulator::Emulator;
use crate::shared::data::bit::BitManipulation;
use anyhow::{anyhow, Error};
use tracing::{debug, error, info};

use super::instruction::Instruction;

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

    pub fn fetch_exec(&self, emulator: &mut Emulator) -> Result<(), Error> {
        let mut is_inc_pc = true;
        self.ld_instruction(emulator)
        self.exec_instruction(emulator, &mut is_inc_pc)?;
        if is_inc_pc {
            emulator.inc_pc();
        }
        Ok(())
    }

    fn ld_next_instruction(&self, emulator: &mut Emulator) -> Result<(), Error> {

    }

    fn exec_instruction(
        &self,
        emulator: &mut Emulator,
        is_inc_pc: &mut bool,
    ) -> Result<(), anyhow::Error> {
        let first_nibble = self.first_nibble();
        let x = self.x();
        let y = self.y();
        let nibble = self.fourth_nibble();
        let addr = self.extract_12bit_address();
        let byte = self.second_byte();

        match first_nibble {
            0x0 => match self.word {
                0x0000 => {
                    debug!("NOP executed: No operation performed.");
                    Instruction::Op0000.call(emulator)?;
                }
                0x00E0 => {
                    debug!("Screen cleared!");
                    Instruction::Op00E0.call(emulator)?;
                }
                0x00EE => {
                    debug!("Returned from subroutine!");
                    Instruction::Op00EE.call(emulator)?;
                }
                _ => {
                    error!("Unsupported instruction: {:#04x}", self.word);
                    return Err(anyhow::anyhow!("Unsupported instruction"));
                }
            },
            0x1 => {
                debug!("Jump to address: {:#04x}", addr);
                Instruction::Op1NNN(addr).call(emulator)?;
                *is_inc_pc = false;
            }
            0x2 => {
                debug!("Call subroutine at address: {:#04x}", addr);
                Instruction::Op2NNN(addr).call(emulator)?;
                *is_inc_pc = false;
            }
            0x3 => {
                debug!("Skip next instruction if V{:X} == {:#04x}", x, byte);
                Instruction::Op3XNN(x, byte).call(emulator)?;
            }
            0x4 => {
                debug!("Skip next instruction if V{:X} != {:#04x}", x, byte);
                Instruction::Op4XNN(x, byte).call(emulator)?;
            }
            0x5 => {
                debug!("Skip next instruction if V{:X} == V{:X}", x, y);
                Instruction::Op5XY0(x, y).call(emulator)?;
            }
            0x6 => {
                debug!("Set V{:X} = {:#04x}", x, byte);
                Instruction::Op6XNN(x, byte).call(emulator)?;
            }
            0x7 => {
                Instruction::Op7XNN(x, byte).call(emulator)?;
            }
            0x8 => match nibble {
                0x0 => {
                    debug!("Set V{:X} = V{:X}", x, y);
                    Instruction::Op8XY0(x, y).call(emulator)?;
                }
                0x1 => {
                    debug!("Set V{:X} = V{:X} | V{:X}", x, y, y);
                    Instruction::Op8XY1(x, y).call(emulator)?;
                }
                0x2 => {
                    debug!("Set V{:X} = V{:X} & V{:X}", x, y, y);
                    Instruction::Op8XY2(x, y).call(emulator)?;
                }
                0x3 => {
                    debug!("Set V{:X} = V{:X} ^ V{:X}", x, y, y);
                    Instruction::Op8XY3(x, y).call(emulator)?;
                }
                0x4 => {
                    debug!("Add V{:X} to V{:X} with carry", x, y);
                    Instruction::Op8XY4(x, y).call(emulator)?;
                }
                0x5 => {
                    debug!("Subtract V{:X} from V{:X} with borrow", y, x);
                    Instruction::Op8XY5(x, y).call(emulator)?;
                }
                0x6 => {
                    debug!("Right shift V{:X} by 1", x);
                    Instruction::Op8XY6(x).call(emulator)?;
                }
                0x7 => {
                    debug!("Set V{:X} = V{:X} - V{:X} with borrow", x, y, x);
                    Instruction::Op8XY7(x, y).call(emulator)?;
                }
                0xE => {
                    debug!("Left shift V{:X} by 1", x);
                    Instruction::Op8XYE(x).call(emulator)?;
                }
                _ => return Err(anyhow::anyhow!("Unsupported instruction")),
            },
            0x9 => {
                debug!("Skip next instruction if V{:X} != V{:X}", x, y);
                Instruction::Op9XY0(x, y).call(emulator)?;
            }
            0xA => {
                debug!("Set I = {:#04x}", addr);
                Instruction::OpANNN(addr).call(emulator)?;
            }
            0xB => {
                debug!("Jump to address V0 + {:#04x}", addr);
                Instruction::OpBNNN(addr).call(emulator)?;
                *is_inc_pc = false;
            }
            0xC => {
                debug!("Set V{:X} = random byte AND {:#04x}", x, byte);
                Instruction::OpCXNN(x, byte).call(emulator)?;
            }
            0xD => {
                debug!(
                    "Draw sprite at V{:X}, V{:X} with height {:#X}",
                    x, y, nibble
                );
                Instruction::OpDXYN(x, y, nibble).call(emulator)?;
            }
            0xE => match byte {
                0x9E => {
                    debug!("Skip next instruction if key V{:X} is pressed", x);
                    Instruction::OpEX9E(x).call(emulator)?;
                }
                0xA1 => {
                    debug!("Skip next instruction if key V{:X} is not pressed", x);
                    Instruction::OpEXA1(x).call(emulator)?;
                }
                _ => return Err(anyhow::anyhow!("Unsupported instruction")),
            },
            0xF => match byte {
                0x07 => {
                    debug!("Set V{:X} = delay timer value", x);
                    Instruction::OpFX07(x).call(emulator)?;
                }
                0x0A => {
                    debug!("Wait for a key press, store in V{:X}", x);
                    Instruction::OpFX0A(x).call(emulator)?;
                }
                0x15 => {
                    debug!("Set delay timer = V{:X}", x);
                    Instruction::OpFX15(x).call(emulator)?;
                }
                0x18 => {
                    debug!("Set sound timer = V{:X}", x);
                    Instruction::OpFX18(x).call(emulator)?;
                }
                0x1E => {
                    debug!("Set I = I + V{:X}", x);
                    Instruction::OpFX1E(x).call(emulator)?;
                }
                0x29 => {
                    debug!("Set I = location of sprite for digit V{:X}", x);
                    Instruction::OpFX29(x).call(emulator)?;
                }
                0x33 => {
                    debug!("Store BCD of V{:X} in memory locations I, I+1, I+2", x);
                    Instruction::OpFX33(x).call(emulator)?;
                }
                0x55 => {
                    debug!(
                        "Store registers V0 through V{:X} in memory starting at I",
                        x
                    );
                    Instruction::OpFX55(x).call(emulator)?;
                }
                0x65 => {
                    debug!(
                        "Read registers V0 through V{:X} from memory starting at I",
                        x
                    );
                    Instruction::OpFX65(x).call(emulator)?;
                }
                _ => return Err(anyhow::anyhow!("Unsupported instruction")),
            },
            _ => {
                error!("Unsupported instruction: {:#04x}", self.word);
                return Err(anyhow::anyhow!("Unsupported instruction"));
            }
        }

        Ok(())
    }
}
