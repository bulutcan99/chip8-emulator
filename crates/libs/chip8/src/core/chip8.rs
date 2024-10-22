use std::default::Default;

const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_REGS: usize = 16;
const START_ADDR: u16 = 0x200;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct CHIP8 {
    pub ram: [u8; RAM_SIZE],
    pub stack: [u16; STACK_SIZE],
    pub v_reg: [u8; NUM_REGS],
    pub i_reg: u16,
    pub sp: u8,
    pub pc: u16,
    pub dt: u8,
    pub st: u8,
}

impl Default for CHIP8 {
    fn default() -> Self {
        Self {
            ram: [0; RAM_SIZE],
            stack: [0; STACK_SIZE],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            pc: START_ADDR,
            dt: 0,
            st: 0,
        }
    }
}

impl CHIP8 {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
