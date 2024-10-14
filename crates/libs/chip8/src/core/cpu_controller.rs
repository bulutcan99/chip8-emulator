use crate::core::emulator::Emulator;
use crate::shared::data::bit::BitManipulation;
use anyhow::{anyhow, Error};
use rand::Rng;
use tracing::{debug, error, info};

use super::chip8::{SCREEN_HEIGHT, SCREEN_WIDTH};

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

    fn skip_next_instruction(&self, emu: &mut Emulator) {
        emu.inc_pc_by(2)
    }

    fn rollback_instruction(&self, emu: &mut Emulator) {
        emu.dec_pc_by(2)
    }

    // **INSTRUCTIONS**

    // 0000 - NOP
    fn no_operation(&self, emu: &mut Emulator) {
        debug!("NOP executed: No operation performed.");
    }

    // 00E0 - CLS -> will implement after sdl
    fn clear_screen(&self, emu: &mut Emulator) {
        emu.clear_screen();
        debug!("Screen cleared!")
    }

    // 00EE - RET (Return from Subroutine)
    fn return_from_subroutine(&self, emu: &mut Emulator) -> Result<(), Error> {
        // Return to the previous function by popping the stack.
        emu.stack_pop().map_err(|err| {
            error!("Failed to return from subroutine: {:?}", err);
            err
        })
    }

    // 1NNN - JMP NNN (Jump to Address NNN)
    fn jump_to_address(&self, emu: &mut Emulator) -> Result<(), Error> {
        let address = self.extract_12bit_address();
        emu.set_pc(address);
        Ok(())
    }

    // 2NNN - CALL NNN (Call Address NNN)
    fn call_address(&self, emu: &mut Emulator) -> Result<(), Error> {
        let address = self.extract_12bit_address();
        match emu.stack_push(emu.get_pc()) {
            Ok(_) => {
                emu.set_pc(address);
                Ok(())
            }
            Err(err) => {
                error!(
                    "Failed to call subroutine at address: {:X}, error: {:?}",
                    address, err
                );
                Err(err)
            }
        }
    }

    // 3XNN - SE VX
    fn skip_equal_vx_byte(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let v = emu.get_v(x)?;
        let second_byte = self.second_byte();
        if v == second_byte {
            self.skip_next_instruction(emu)
        }
        Ok(())
    }

    // 4XNN - SNE VX
    fn skip_not_equal_vx_byte(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let v = emu.get_v(x)?;
        let second_byte = self.second_byte();
        if v != second_byte {
            self.skip_next_instruction(emu)
        }
        Ok(())
    }

    // 5XY0 - SE VX == VY
    fn skip_equal_vx_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;
        let y = self.x();
        let vy = emu.get_v(y)?;
        if vx == vy {
            self.skip_next_instruction(emu)
        }
        Ok(())
    }

    // 6XNN - LD VX (with byte)
    fn load_vx_with_byte(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let second_byte = self.second_byte();
        emu.set_v(x, second_byte)
    }

    // 7XNN - ADD VX, NN
    fn add_vx_with_byte(&mut self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let nn = self.second_byte();

        let vx = emu.get_v(x)?;
        let result = vx.wrapping_add(nn);

        emu.set_v(x, result)?;

        Ok(())
    }

    // 8XY0 - LD VX (with VY)
    fn load_vx_with_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.x();
        let vy = emu.get_v(y)?;
        emu.set_v(x, vy)?;
        Ok(())
    }

    // 8XY1 - LD VX (with VX or VY)
    fn load_vx_with_vx_or_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.y();
        let vx = emu.get_v(x)?;
        let vy = emu.get_v(y)?;
        emu.set_v(x, vx | vy)?;
        Ok(())
    }

    // 8XY2 - LD VX (with VX and VY)
    fn load_vx_with_vx_and_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.y();
        let vx = emu.get_v(x)?;
        let vy = emu.get_v(y)?;
        emu.set_v(x, vx & vy)?;
        Ok(())
    }

    // 8XY3 - LD VX (with VX xor VY)
    fn load_vx_with_vx_xor_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.y();
        let vx = emu.get_v(x)?;
        let vy = emu.get_v(y)?;
        emu.set_v(x, vx ^ vy)?;
        Ok(())
    }

    // 8XY4 - ADD VX, VY
    fn add_vx_with_vy(&mut self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.y();

        let vx = emu.get_v(x)?;
        let vy = emu.get_v(y)?;

        let (result, overflow) = vx.overflowing_add(vy);

        if overflow {
            emu.set_v(0xF, 1)?
        } else {
            emu.set_v(0xF, 0)?;
        }

        emu.set_v(x, result)?;

        Ok(())
    }

    // 8XY5 - SUB VX, VY
    fn sub_vx_to_vy(&mut self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let y = self.y();

        let vx = emu.get_v(x)?;
        let vy = emu.get_v(y)?;

        let (result, overflow) = vx.overflowing_sub(vy);

        if overflow {
            emu.set_v(0xF, 0)?;
        } else {
            emu.set_v(0xF, 1)?;
        }

        emu.set_v(x, result)?;

        Ok(())
    }

    // 8XY6 - SHIFT VX (to the right) by 1
    fn shift_vx_to_right(&mut self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;
        let lsb = vx & 0b0000_0001;

        emu.set_v(0xF, lsb)?;
        let result = vx >> 1;
        emu.set_v(x, result)?;

        Ok(())
    }

    // 8XY7 - MINUS VX = VY - VX
    fn minus_vx_from_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;

        let y = self.y();
        let vy = emu.get_v(y)?;

        let (result, overflow) = vy.overflowing_sub(vx);

        // Set carry flag (VF) based on overflow
        if overflow {
            emu.set_v(0xF, 0)?;
        } else {
            emu.set_v(0xF, 1)?;
        }

        emu.set_v(x, result)?;
        Ok(())
    }

    // 8XYE - SHIFT VX (to the left) by 1
    fn shift_vx_to_left(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;
        // Extract the MSB (8th bit) and shift it to the least-significant bit position
        let msb = (vx & 0b10000000) >> 7;

        emu.set_v(0xF, msb)?;

        let result = vx << 1;

        emu.set_v(x, result)?;

        Ok(())
    }

    //9XY0 - SNE VX != VY
    fn skip_not_equal_vx_vy(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;

        let y = self.y();
        let vy = emu.get_v(y)?;

        if vx != vy {
            self.skip_next_instruction(emu);
        }
        Ok(())
    }

    // ANNN - LD I (with NNN)
    fn load_i_with_nnn(&self, emu: &mut Emulator) -> Result<(), Error> {
        let address = self.extract_12bit_address();
        emu.set_i(address);
        Ok(())
    }

    // BNNN - JP V0 (addr)
    fn jump_to_address_plus_v0(&self, emu: &mut Emulator) -> Result<(), Error> {
        let v0 = emu.get_v(0)?;
        let address = self.extract_12bit_address();
        let result = (v0 as u16) + address;
        emu.set_pc(result);
        Ok(())
    }

    // CXNN - LD VX (with random)
    fn load_vx_with_random_number(&self, emu: &mut Emulator) -> Result<(), Error> {
        let rnd = rand::thread_rng().gen_range(0..=255);
        let second_byte = self.second_byte();
        let result = second_byte & rnd;
        let x = self.x();
        emu.set_v(x, result)?;
        Ok(())
    }

    // DXYN - DRW Sprite
    fn draw_sprite(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;

        let y = self.y();
        let vy = emu.get_v(y)?;

        let rows = self.fourth_nibble();
        let mut collision = false;
        for ordinate in 0..rows {
            let addr = emu.get_i() + ordinate as u16;
            let pixel_row = emu.get_ram()[addr as usize];
            for abscissa in 0..8 {
                if (pixel_row & (0b1000_0000 >> abscissa)) != 0 {
                    let x = (vx as usize + abscissa) % SCREEN_WIDTH;
                    let y = (vy as usize + ordinate as usize) % SCREEN_HEIGHT;
                    let index = x + y * SCREEN_WIDTH;
                    collision |= emu.get_display()[index];
                    emu.get_display()[index] ^= true;
                }
            }
        }

        if collision {
            emu.set_v(0xF, 1)
        } else {
            emu.set_v(0xF, 0)
        }
    }

    // EX9E - SKP VX == key
    fn skip_by_key(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;

        let is_pressed = emu.is_key_pressed(vx);
        if is_pressed {
            self.skip_next_instruction(emu);
        }
        Ok(())
    }

    // EXA1 - SKP VX != key
    fn skip_by_not_key(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let vx = emu.get_v(x)?;

        let is_pressed = emu.is_key_pressed(vx);
        if !is_pressed {
            self.skip_next_instruction(emu);
        }
        Ok(())
    }

    // FX07 - LD VX (with DT)
    fn load_vx_with_dt(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        let dt = emu.get_dt();
        emu.set_v(x, dt)
    }

    // FX0A - WT KEY (to VX)
    fn wait_for_key(&self, emu: &mut Emulator) -> Result<(), Error> {
        let x = self.x();
        loop {
            if let Some(key) = emu.check_key_press() {
                emu.set_v(x, key)?;
                break;
            }
        }
        Ok(())
    }
}
