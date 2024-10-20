pub struct BitManipulation;


impl BitManipulation {
	pub fn combine_nibbles_to_16bit_address(x: u8, y: u8, fourth_nibble: u8) -> u16 {
		(x as u16) << 8 | (y as u16) << 4 | (fourth_nibble as u16)
	}

	pub fn combine_bytes_to_16bit_instruction(first_byte: u8, second_byte: u8) -> u16 {
		(first_byte as u16) << 8 | (second_byte as u16)
	}

	pub fn convert_decimal_to_bcd_tuple(decimal: u8) -> (u8, u8, u8) {
		let hundreds = decimal / 100;
		let tens = (decimal % 100) / 10;
		let ones = decimal % 10;

		(hundreds, tens, ones)
	}
}