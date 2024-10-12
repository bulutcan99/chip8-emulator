// Total size of the Chip-8 system's memory (in bytes)
const RAM_SIZE: usize = 4096;

// Size of the CPU's call stack (for storing return addresses)
const STACK_SIZE: usize = 16;

// Number of general-purpose registers available in the Chip-8 CPU
const NUM_REGS: usize = 16;

// Screen refresh rate (frames per second)
const REFRESH_RATE: usize = 60;

// Chip-8 standard that the beginning of all Chip-8 programs will be loaded in starting at RAM address 0x200.
const START_ADDR: u16 = 0x200;

// Default display resolution.
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;


pub struct CHIP8 {
	// Will address from 0x000 (0) to 0x1ff (511)
	// ROM data will get after first 512 byte
	// 4096 bytes total
	pub ram: [u8; RAM_SIZE],

	// The stack allows for up to *STACK_SIZE* nested subroutines.
	pub stack: [u16; STACK_SIZE],

	// Variable register; General purpose registers.
	pub v_reg: [u8; NUM_REGS],

	// Index register; acts as a pointer for accessing and manipulating memory
	pub i_reg: u16,

	// Stack pointer; used to point to the top level of the stack.
	pub sp: u8,

	// Program counter; stores the memory of the current executed instruction.
	pub pc: u16,

	// Delay timer; if non-zero (activated), will decrease by 1 at *REFRESH_RATE*
	// until reaches zero (deactivated).
	pub dt: u8,

	// Sound timer; if non-zero (activated) sounds the buzzer sound.
	pub st: u8,

	// Display (64x32 pixels, true = pixel on, false = pixel off)
	pub display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
}

impl CHIP8 {
	pub fn new() -> Self {
		let mut new_chip8 = Self {
			ram: [0; RAM_SIZE],
			stack: [0; STACK_SIZE],
			v_reg: [0; NUM_REGS],
			i_reg: 0,
			sp: 0,
			pc: START_ADDR,
			dt: 0,
			st: 0,
			display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
		};

		new_chip8
	}

	pub fn reset(&mut self) {
		self.ram = [0; RAM_SIZE];

		self.stack = [0; STACK_SIZE];
		self.v_reg = [0; NUM_REGS];
		self.i_reg = 0;
		self.sp = 0;
		self.pc = START_ADDR;
		self.dt = 0;
		self.st = 0;

		self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
	}

	fn get_reserved_memory(&self) -> &[u8] {
		&self.ram[0..512]
	}
}