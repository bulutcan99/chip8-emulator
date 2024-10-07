use crate::memory::Memory;
use std::fs::File;
use std::io::Read;

pub struct MemoryController {
	memory: Memory,
}

impl MemoryController {
	pub fn new(mem: Memory) -> Self {
		Self {
			memory: mem,
		}
	}

	fn load_rom_file(&mut self, path: &str) {
		let mut byte_vec: Vec<u8> = Vec::new();
		File::open(path).unwrap().read_to_end(&mut byte_vec).unwrap();
		// 4096 (RAM size) - 200 (Reserved RAM)
		if byte_vec.len() > 3584 {
			panic!("The selected ROM size will overflow beyond the limit of RAM!")
		}
	}
}