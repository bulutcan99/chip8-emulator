pub struct Math2d;

impl Math2d {
	pub fn wrap_coord(axis: u8, win_size: u32) -> u8 {
		if axis as u32 > win_size - 1 {
			axis % win_size as u8
		} else { axis }
	}
}