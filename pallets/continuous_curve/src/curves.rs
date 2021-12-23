use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::Scale, SaturatedConversion};

#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum CurveType {
	Linear,
	Exponential,
	Flat,
	Logarithmic,
}

impl CurveType {
	pub fn get_reserve_ratio(&self) -> (u128, u128) {
		match &self {
			CurveType::Exponential => (10, 100),
			CurveType::Flat => (100, 100),
			CurveType::Linear => (50, 100),
			CurveType::Logarithmic => (90, 100),
		}
	}
}

const MAX_RESERVE_RATIO: u128 = 1000000;

#[derive(Copy, Clone)]
pub struct Power {
	pub base_N: u128,
	pub base_D: u128,
	pub exp_N: u128,
	pub exp_D: u128,
}
#[derive(Copy, Clone)]
pub struct CalculatePurchaseAndSellReturn {
	pub supply: u128,
	pub reserve_balance: u128,
	pub reserve_ratio: u128,
	pub deposit_amount: u128,
	pub power: Power,
}

impl CalculatePurchaseAndSellReturn {
	pub fn integral_purchase(&self, precision: u128, base_N: u128) -> u128 {
		let power: u128 =
			(base_N / self.power.base_D).pow(self.power.exp_N as u32 / self.power.exp_D as u32) * 2;
		let power_with_precision = power.pow(precision as u32);
		return power_with_precision;
	}

	pub fn purchase_return(
		self,
		supply: u128,
		reserve_balance: u128,
		reserve_ratio: u128,
		deposit_amount: u128,
	) -> u128 {
		// assert!(
		// 	supply > 0
		// 		&& reserve_balance > 0
		// 		&& reserve_ratio > 0
		// 		&& reserve_ratio <= MAX_RESERVE_RATIO
		// );

		if deposit_amount == 0 {
			return 0;
		} else {
			let result: u128 = supply.mul(deposit_amount).div(reserve_balance);
			let precision: u128 = 10;
			let base_N: u128 = deposit_amount + reserve_balance;
			let value: u128 = self.integral_purchase(precision, base_N);
			let new_token_supply: u128 = supply.mul(value) >> precision;
			return new_token_supply - supply;
		}
	}
	pub fn integral_sell(&self, precision: u128, base_D: u128) -> u128 {
		let power: u128 =
			(self.power.base_N / base_D).pow(self.power.exp_N as u32 / self.power.exp_D as u32) * 2;
		let power_with_precision = power.pow(precision as u32);
		return power_with_precision;
	}

	pub fn calculate_sale_return(
		self,
		supply: u128,
		reserve_balance: u128,
		reserve_ratio: u128,
		sell_amount: u128,
	) -> u128 {
		// assert!(
		// 	supply > 0
		// 		&& reserve_balance > 0
		// 		&& reserve_ratio > 0
		// 		&& reserve_ratio <= MAX_RESERVE_RATIO
		// );
		if sell_amount == 0 {
			return 0;
		} else {
			let result: u128 = reserve_balance.mul(sell_amount).div(supply);
			let precision: u128 = 10;
			let base_D: u128 = supply - sell_amount;
			let value: u128 = self.integral_sell(precision, base_D);
			let old_balance: u128 = reserve_balance.mul(value);
			let new_balance: u128 = reserve_balance << precision;
			return (old_balance - new_balance).div(result);
		}
	}
}
