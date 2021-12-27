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

#[derive(Copy, Clone, Debug)]
pub struct Power {
	pub base_N: u32,
	pub base_D: u32,
	pub exp_N: u32,
	pub exp_D: u32,
}
#[derive(Copy, Clone, Debug)]
pub struct CalculatePurchaseAndSellReturn {
	pub power: Power,
}

impl CalculatePurchaseAndSellReturn {
	pub fn integral_purchase(&self, precision: u32, base_N: u32) -> u32 {
		let power =
			(base_N / self.power.base_D).pow(self.power.exp_N / self.power.exp_D) * 2;
		let power_with_precision = power.pow(precision);
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
			log::info!("purchase_return supply: {}, reserve_balance: {}, reserve_ratio: {}, deposit_amount: {}", supply.clone(), reserve_balance.clone(), reserve_ratio.clone(), deposit_amount.clone());
			let result = supply.mul(deposit_amount).div(reserve_balance);
			log::info!("purchase_return result: {:?}", result.clone());
			let precision: u32 = 1;
			let base_N = deposit_amount + reserve_balance;
			log::info!("purchase_return base_N: {:?}", base_N.clone());
			let value = self.integral_purchase(precision, base_N as u32);
			log::info!("purchase_return value: {:?}", value.clone());
			let new_token_supply = supply.mul(value as u128) >> precision;
			log::info!("purchase_return new_token_supply: {:?}", new_token_supply.clone());
			return new_token_supply - supply;
		}
	}

	pub fn integral_sell(&self, precision: u32, base_D: u32) -> u32 {
		let power =
			(self.power.base_N / base_D).pow(self.power.exp_N / self.power.exp_D) * 2;
		let power_with_precision = power.pow(precision);
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
			log::info!(
				"sell_return supply: {}, reserve_balance: {}, reserve_ratio: {}, sell_amount: {}",
				supply.clone(),
				reserve_balance.clone(),
				reserve_ratio.clone(),
				sell_amount.clone()
			);
			let test = reserve_balance * sell_amount / supply;
			log::info!("sale_return test_result: {:?}", test.clone());
			let result = reserve_balance.mul(sell_amount).div(supply);
			log::info!("sale_return result: {:?}", result.clone());
			let precision: u32 = 1;
			let base_D = supply - sell_amount;
			log::info!("sale_return base_D: {:?}", base_D.clone());
			let value = self.integral_sell(precision, base_D as u32);
			log::info!("sale_return value: {:?}", value.clone());
			let old_balance = reserve_balance.mul(value as u128);
			log::info!("sale_return old_balance: {:?}", old_balance.clone());
			let new_balance = reserve_balance << precision;
			log::info!("sale_return new_balance: {:?}", new_balance.clone());
			return (old_balance - new_balance).div(result);
		}
	}
}
