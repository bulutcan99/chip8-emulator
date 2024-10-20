use super::{emulator::Emulator, instruction::Instruction};
use anyhow::{anyhow, Error};
use log::{debug, error};
use shared::data::bit::BitManipulation;
use tracing::info;

pub struct CpuController;

impl CpuController {
    pub fn fetch(&self, emulator: &mut Emulator) -> Result<u16, anyhow::Error> {
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

        // Combine the two bytes into a 16-bit word (instruction)
        let word = BitManipulation::combine_bytes_to_16bit_instruction(first_byte, second_byte);
        emulator.inc_pc_by(2);
        info!("CPU initialized with instruction word: {:#04x}", word);

        Ok(word)
    }

    pub fn tick(&self, emulator: &mut Emulator) -> Result<(), Error> {
        // Fetch the next instruction
        let word = self.fetch(emulator)?;
        // Execute the instruction
        self.exec_instruction(emulator, word)?;
        Ok(())
    }

    fn first_nibble(word: u16) -> u8 {
        (word >> 12) as u8
    }

    fn second_byte(word: u16) -> u8 {
        word as u8
    }

    fn x(word: u16) -> u8 {
        ((word >> 8) & 0x0F) as u8
    }

    fn y(word: u16) -> u8 {
        ((word >> 4) & 0x0F) as u8
    }

    fn fourth_nibble(word: u16) -> u8 {
        (word & 0x0F) as u8
    }

    fn extract_12bit_address(word: u16) -> u16 {
        word & 0x0FFF
    }

    fn exec_instruction(&self, emulator: &mut Emulator, word: u16) -> Result<(), anyhow::Error> {
        let first_nibble = CpuController::first_nibble(word);
        let x = CpuController::x(word);
        let y = CpuController::y(word);
        let nibble = CpuController::fourth_nibble(word);
        let addr = CpuController::extract_12bit_address(word);
        let byte = CpuController::second_byte(word);

        match first_nibble {
            0x0 => match word {
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
                    error!("Unsupported instruction: {:#04x}", word);
                    return Err(anyhow!("Unsupported instruction"));
                }
            },
            0x1 => {
                debug!("Jump to address: {:#04x}", addr);
                Instruction::Op1NNN(addr).call(emulator)?;
            }
            0x2 => {
                debug!("Call subroutine at address: {:#04x}", addr);
                Instruction::Op2NNN(addr).call(emulator)?;
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
                debug!("Add {:#04x} to V{:X}", byte, x);
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
                _ => return Err(anyhow!("Unsupported instruction")),
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
                _ => return Err(anyhow!("Unsupported instruction")),
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
                _ => return Err(anyhow!("Unsupported instruction")),
            },
            _ => {
                error!("Unsupported instruction: {:#04x}", word);
                return Err(anyhow!("Unsupported instruction"));
            }
        }

        Ok(())
    }
}
