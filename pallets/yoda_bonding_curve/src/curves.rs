use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub enum CurveType {
	Linear,
}

#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq)]
pub struct Linear {
	pub exponent: u32,
	pub slope: u128,
}

impl Linear {
	pub fn new(exponent: u32, slope: u128) -> Self {
		Self { exponent, slope }
	}
	/// Integral when the curve is at point `x`.
	pub fn integral(&self, x: u128) -> u128 {
		let nexp = self.exponent + 1;
		x.pow(nexp) * self.slope / nexp as u128
	}
}

impl Default for Linear {
	fn default() -> Self {
		Self { exponent: 1, slope: 1 }
	}
}
