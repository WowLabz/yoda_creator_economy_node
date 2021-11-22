// use frame_support::pallet_prelude::*;
// use frame_support::traits::{Currency, ExistenceRequirement, Get, Randomness};
// use frame_system::pallet_prelude::*;
// use frame_system::Config;
// use orml_traits::{MultiCurrency, MultiReservableCurrency};
// use scale_info::TypeInfo;


// #[derive(Encode, Decode, TypeInfo, Clone)]
// #[scale_info(skip_type_params(T))]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub struct BondingToken<T: Config> {
// 	// The module-owned account for this bonding curve.
// 	// account: AccountId,
// 	/// The creator of the bonding curve.
// 	creator: AccountOf<T>,
// 	/// The asset id of the bonding curve token.
// 	asset_id: CurrencyIdOf<T>,
// 	/// The exponent of the curve.
// 	exponent: u32,
// 	/// The slope of the curve.
// 	slope: u128,
// 	/// The maximum supply that can be minted from the curve.
// 	max_supply: u128,
// 	/// the token name
// 	token_name: Vec<u8>,
// 	/// The token symbol
// 	token_symbol: Vec<u8>,
// 	/// Token decimals
// 	token_decimals: u8,
// 	/// token bonding curve_id
// 	curve_id: u64,
// 	/// mint
// 	mint: MintingData<T>,
// }

// impl<T: Config> BondingToken<T> {
// 	/// Integral when the curve is at point `x`.
// 	pub fn integral(&self, x: u128) -> u128 {
// 		let nexp = self.exponent + 1;
// 		x.pow(nexp) * self.slope / nexp as u128
// 	}
// }

// #[derive(Encode, Decode, TypeInfo, Clone)]
// #[scale_info(skip_type_params(T))]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub struct MintingData<T: Config> {
// 	///
// 	minter: AccountOf<T>,
// 	///
// 	tokens_to_mint: Option<BalanceOf<T>>,
// }
