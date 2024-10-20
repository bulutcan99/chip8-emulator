pub struct LogicManipulation;

impl LogicManipulation {
	pub fn convert_bool_to_u8(exp: bool) -> u8 {
		match exp {
			true => 1,
			false => 0
		}
	}
}