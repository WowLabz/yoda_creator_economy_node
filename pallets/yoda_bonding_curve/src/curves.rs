use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::SaturatedConversion;
use std::clone::Clone;
use std::fmt::Debug;
use std::marker::Sized;

pub trait CurveConfig {
	fn integral_before(&self, issuance: u128) -> u128;
	fn integral_after(&self, issuance: u128) -> u128;
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum CurveType {
	Linear,
}

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Linear {
	exponent: u32,
	slope: u128,
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

impl CurveConfig for Linear {
	fn integral_before(&self, issuance: u128) -> u128 {
		self.integral(issuance).saturated_into()
	}
	fn integral_after(&self, issuance: u128) -> u128 {
		self.integral(issuance).saturated_into()
	}
}
