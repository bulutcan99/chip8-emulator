use anyhow::Error;
use rand::Rng;
use tracing::error;

use super::{
    chip8::{SCREEN_HEIGHT, SCREEN_WIDTH},
    emulator::Emulator,
};

pub enum Instruction {
    Op0000,
    Op00E0,
    Op00EE,
    Op1NNN(u16),
    Op2NNN(u16),
    Op3XNN(u8, u8),
    Op4XNN(u8, u8),
    Op5XY0(u8, u8),
    Op6XNN(u8, u8),
    Op7XNN(u8, u8),
    Op8XY0(u8, u8),
    Op8XY1(u8, u8),
    Op8XY2(u8, u8),
    Op8XY3(u8, u8),
    Op8XY4(u8, u8),
    Op8XY5(u8, u8),
    Op8XY6(u8),
    Op8XY7(u8, u8),
    Op8XYE(u8),
    Op9XY0(u8, u8),
    OpANNN(u16),
    OpBNNN(u16),
    OpCXNN(u8, u8),
    OpDXYN(u8, u8, u8),
    OpEX9E(u8),
    OpEXA1(u8),
    OpFX07(u8),
    OpFX0A(u8),
    OpFX15(u8),
    OpFX18(u8),
    OpFX1E(u8),
    OpFX29(u8),
    OpFX33(u8),
    OpFX55(u8),
    OpFX65(u8),
}

impl Instruction {
    pub fn call(&self, emu: &mut Emulator) -> Result<(), Error> {
        match self {
            Instruction::Op0000 => {} // NOP
            Instruction::Op00E0 => {
                emu.clear_screen();
            }
            Instruction::Op00EE => {
                emu.stack_pop().map_err(|err| {
                    error!("Failed to return from subroutine: {:?}", err);
                    err
                })?;
            }
            Instruction::Op1NNN(addr) => {
                emu.set_pc(*addr);
            }
            Instruction::Op2NNN(addr) => {
                emu.stack_push(emu.get_pc())?;
                emu.set_pc(*addr);
            }
            Instruction::Op3XNN(x, byte) => {
                let v = emu.get_v(*x)?;
                if v == *byte {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::Op4XNN(x, byte) => {
                let v = emu.get_v(*x)?;
                if v != *byte {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::Op5XY0(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                if vx == vy {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::Op6XNN(x, byte) => {
                emu.set_v(*x, *byte)?;
            }
            Instruction::Op7XNN(x, byte) => {
                let vx = emu.get_v(*x)?;
                let result = vx.wrapping_add(*byte as u8);
                emu.set_v(*x, result)?;
            }
            Instruction::Op8XY0(x, y) => {
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vy)?;
            }
            Instruction::Op8XY1(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx | vy)?;
            }
            Instruction::Op8XY2(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx & vy)?;
            }
            Instruction::Op8XY3(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx ^ vy)?;
            }
            Instruction::Op8XY4(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vx.overflowing_add(vy);
                emu.set_v(0xF, if overflow { 1 } else { 0 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::Op8XY5(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vx.overflowing_sub(vy);
                emu.set_v(0xF, if overflow { 0 } else { 1 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::Op8XY6(x) => {
                let vx = emu.get_v(*x)?;
                let lsb = vx & 0b0000_0001;
                emu.set_v(0xF, lsb)?;
                let result = vx >> 1;
                emu.set_v(*x, result)?;
            }
            Instruction::Op8XY7(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vy.overflowing_sub(vx);
                emu.set_v(0xF, if overflow { 0 } else { 1 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::Op8XYE(x) => {
                let vx = emu.get_v(*x)?;
                let msb = (vx & 0b10000000) >> 7;
                emu.set_v(0xF, msb)?;
                let result = vx << 1;
                emu.set_v(*x, result)?;
            }
            Instruction::Op9XY0(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                if vx != vy {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::OpANNN(addr) => {
                emu.set_i(*addr);
            }
            Instruction::OpBNNN(addr) => {
                let v0 = emu.get_v(0)?;
                emu.set_pc((*addr) + (v0 as u16));
            }
            Instruction::OpCXNN(x, byte) => {
                let rnd = rand::thread_rng().gen_range(0..=255);
                emu.set_v(*x, rnd & *byte)?;
            }
            Instruction::OpDXYN(x, y, nibble) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let rows = *nibble;
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
                    emu.set_v(0xF, 1)?;
                } else {
                    emu.set_v(0xF, 0)?;
                }
            }
            Instruction::OpEX9E(x) => {
                let vx = emu.get_v(*x)?;
                let is_pressed = emu.is_key_pressed(vx);
                if is_pressed? {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::OpEXA1(x) => {
                let vx = emu.get_v(*x)?;
                let is_pressed = emu.is_key_pressed(vx);
                if !is_pressed? {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::OpFX07(x) => {
                let dt = emu.get_dt();
                emu.set_v(*x, dt)?;
            }
            Instruction::OpFX0A(x) => {
                if let Some(key) = emu.check_key_press() {
                    emu.set_v(*x, key)?;
                } else {
                    emu.dec_pc_by(2);
                }
            }
            Instruction::OpFX15(x) => {
                let vx = emu.get_v(*x)?;
                emu.set_dt(vx);
            }
            Instruction::OpFX18(x) => {
                let vx = emu.get_v(*x)?;
                emu.set_st(vx);
            }
            Instruction::OpFX1E(x) => {
                let vx = emu.get_v(*x)?;
                let i = emu.get_i();
                emu.set_i(i.wrapping_add(vx as u16));
            }
            Instruction::OpFX29(x) => {
                let vx = emu.get_v(*x)?;
                let f = 5 * vx as u16;
                emu.set_i(f);
            }
            Instruction::OpFX33(x) => {
                let vx = emu.get_v(*x)?;
                let hundreds = (vx / 100) as u8;
                let tens = (vx / 10) % 10 as u8;
                let ones = (vx % 10) as u8;

                emu.set_to_ram(emu.get_i() as usize, hundreds)?;
                emu.set_to_ram(emu.get_i() as usize + 1, tens)?;
                emu.set_to_ram(emu.get_i() as usize + 2, ones)?;
            }
            Instruction::OpFX55(x) => {
                let i = emu.get_i();
                for index in 0..=*x {
                    let vx = emu.get_v(index)?;
                    emu.set_to_ram(i as usize + index as usize, vx)?;
                }
            }
            Instruction::OpFX65(x) => {
                let i = emu.get_i();
                for idx in 0..=*x {
                    let value = emu.get_ram()[i as usize + idx as usize];
                    emu.set_v(idx, value)?;
                }
            }
        }
        Ok(())
    }
}
