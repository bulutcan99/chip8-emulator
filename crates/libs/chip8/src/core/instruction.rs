use anyhow::Error;
use rand::Rng;
use tracing::error;

use super::{
    chip8::{SCREEN_HEIGHT, SCREEN_WIDTH},
    emulator::Emulator,
};

//TODO: GET_RAM WILL CHANGE TO SET_RAM (for sets)
pub enum Instruction {
    Nop,
    Cls,
    Ret,
    Jmp(u16),
    Call(u16),
    SeVx(u8, u8),
    SneVx(u8, u8),
    SeVxVy(u8, u8),
    LdVx(u8, u8),
    AddVx(u8, u8),
    LdVxVy(u8, u8),
    OrVxVy(u8, u8),
    AndVxVy(u8, u8),
    XorVxVy(u8, u8),
    AddVxVy(u8, u8),
    SubVxVy(u8, u8),
    ShrVx(u8),
    SubnVxVy(u8, u8),
    ShlVx(u8),
    SneVxVy(u8, u8),
    LdI(u16),
    JpV0(u16),
    Rnd(u8, u8),
    Drw(u8, u8, u8),
    SkpVx(u8),
    SknpVx(u8),
    LdVxDt(u8),
    LdVxK(u8),
    LdDtVx(u8),
    LdStVx(u8),
    AddIVx(u8),
    LdFVx(u8),
    LdBVx(u8),
    LdIVx(u8),
    LdVxI(u8),
}

impl Instruction {
    pub fn execute(&self, emu: &mut Emulator, is_inc_pc: &mut bool) -> Result<(), Error> {
        match self {
            Instruction::Nop => {}
            Instruction::Cls => {
                emu.clear_screen();
            }
            Instruction::Ret => {
                emu.stack_pop().map_err(|err| {
                    error!("Failed to return from subroutine: {:?}", err);
                    err
                })?;
            }
            Instruction::Jmp(addr) => {
                emu.set_pc(*addr);
                *is_inc_pc = false; // Set to false
            }
            Instruction::Call(addr) => {
                emu.stack_push(emu.get_pc())?;
                emu.set_pc(*addr);
                *is_inc_pc = false; // Set to false
            }
            Instruction::SeVx(x, byte) => {
                let v = emu.get_v(*x)?;
                // Example implementation
                if v == *byte {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::SneVx(x, byte) => {
                let v = emu.get_v(*x)?;
                // Example implementation
                if v != *byte {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::SeVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                if vx == vy {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::LdVx(x, byte) => {
                emu.set_v(*x, *byte)?;
            }
            Instruction::AddVx(x, byte) => {
                let vx = emu.get_v(*x)?;
                let (result, _) = vx.overflowing_add(*byte);
                emu.set_v(*x, result)?;
            }
            Instruction::LdVxVy(x, y) => {
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vy)?;
            }
            Instruction::OrVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx | vy)?;
            }
            Instruction::AndVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx & vy)?;
            }
            Instruction::XorVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                emu.set_v(*x, vx ^ vy)?;
            }
            Instruction::AddVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vx.overflowing_add(vy);
                emu.set_v(0xF, if overflow { 1 } else { 0 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::SubVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vx.overflowing_sub(vy);
                emu.set_v(0xF, if overflow { 0 } else { 1 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::ShrVx(x) => {
                let vx = emu.get_v(*x)?;
                let lsb = vx & 0b0000_0001;
                emu.set_v(0xF, lsb)?;
                let result = vx >> 1;
                emu.set_v(*x, result)?;
            }
            Instruction::SubnVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                let (result, overflow) = vy.overflowing_sub(vx);
                emu.set_v(0xF, if overflow { 0 } else { 1 })?;
                emu.set_v(*x, result)?;
            }
            Instruction::ShlVx(x) => {
                let vx = emu.get_v(*x)?;
                let msb = (vx & 0b10000000) >> 7;
                emu.set_v(0xF, msb)?;
                let result = vx << 1;
                emu.set_v(*x, result)?;
            }
            Instruction::SneVxVy(x, y) => {
                let vx = emu.get_v(*x)?;
                let vy = emu.get_v(*y)?;
                if vx != vy {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::LdI(addr) => {
                emu.set_i(*addr);
            }
            Instruction::JpV0(addr) => {
                let v0 = emu.get_v(0)?;
                emu.set_pc((*addr).wrapping_add(v0 as u16));
            }
            Instruction::Rnd(x, byte) => {
                let rnd = rand::thread_rng().gen_range(0..=255);
                emu.set_v(*x, rnd & *byte)?;
            }
            Instruction::Drw(x, y, nibble) => {
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
            Instruction::SkpVx(x) => {
                let vx = emu.get_v(*x)?;

                let is_pressed = emu.is_key_pressed(vx);
                if is_pressed {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::SknpVx(x) => {
                let vx = emu.get_v(*x)?;
                let is_pressed = emu.is_key_pressed(vx);
                if !is_pressed {
                    emu.inc_pc_by(2);
                }
            }
            Instruction::LdVxDt(x) => {
                let dt = emu.get_dt();
                emu.set_v(*x, dt)?;
            }
            Instruction::LdVxK(x) => {
                if let Some(key) = emu.check_key_press() {
                    emu.set_v(*x, key)?;
                } else {
                    emu.dec_pc_by(2);
                    *is_inc_pc = false; // Set to false
                }
            }
            Instruction::LdDtVx(x) => {
                let vx = emu.get_v(*x)?;
                emu.set_dt(vx);
            }
            Instruction::LdStVx(x) => {
                let vx = emu.get_v(*x)?;
                emu.set_st(vx);
            }
            Instruction::AddIVx(x) => {
                let vx = emu.get_v(*x)?;
                let i = emu.get_i();
                emu.set_i(i.wrapping_add(vx as u16));
            }
            Instruction::LdFVx(x) => {
                let vx = emu.get_v(*x)?;
                let f = 5 * vx as u16;
                emu.set_i(f);
            }
            Instruction::LdBVx(x) => {
                let vx = emu.get_v(*x)?;
                let hundreds = (vx / 100) as u8;
                let tens = (vx / 10) % 10 as u8;
                let ones = (vx % 10) as u8;

                emu.set_to_ram(emu.get_i() as usize, hundreds)?;
                emu.set_to_ram(emu.get_i() as usize + 1, tens)?;
                emu.set_to_ram(emu.get_i() as usize + 2, ones)?;
            }
            Instruction::LdIVx(x) => {
                let i = emu.get_i();
                for index in 0..=*x {
                    let vx = emu.get_v(index)?;
                    emu.set_to_ram(i as usize + index as usize, vx)?;
                }
            }
            Instruction::LdVxI(x) => {
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
