#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

mod curves;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use crate::curves::*;
	use frame_support::inherent::Vec;
	use frame_support::traits::{Currency, ExistenceRequirement, Get, Randomness};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, transactional, PalletId};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use scale_info::TypeInfo;
	use sp_runtime::traits::{AccountIdConversion, SaturatedConversion};
	use codec;

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;

	#[derive(Encode, Decode, TypeInfo, Clone)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct BondingCurve<T: Config> {
		// The module-owned account for this bonding curve.
		// account: AccountId,
		/// The creator of the bonding curve.
		creator: AccountOf<T>,
		/// The asset id of the bonding curve token.
		asset_id: CurrencyIdOf<T>,
		/// The exponent of the curve.
		exponent: u32,
		/// The slope of the curve.
		slope: u128,
		/// The maximum supply that can be minted from the curve.
		max_supply: u128,
		/// the token name
		token_name: Vec<u8>,
		/// The token symbol
		token_symbol: Vec<u8>,
		/// Token decimals
		token_decimals: u8,
		/// token bonding curve_id
		curve_id: u64,
	}

	impl<T: Config> BondingCurve<T> {
		/// Integral when the curve is at point `x`.
		pub fn integral(&self, x: u128) -> u128 {
			let nexp = self.exponent + 1;
			x.pow(nexp) * self.slope / nexp as u128
		}
	}

	#[derive(Encode, Decode, TypeInfo, Clone)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct BondingToken<T: Config> {
		// The module-owned account for this bonding curve.
		// account: AccountId,
		/// The creator of the bonding curve.
		creator: AccountOf<T>,
		/// The asset id of the bonding curve token.
		asset_id: CurrencyIdOf<T>,
		/// bonding curve type with the config
		curve: CurveType,
		/// curve config
		curve_config: Box<dyn CurveConfig + Clone>,
		/// The maximum supply that can be minted from the curve.
		max_supply: BalanceOf<T>,
		/// the token name
		token_name: Vec<u8>,
		/// The token symbol
		token_symbol: Vec<u8>,
		/// Token decimals
		token_decimals: u8,
		/// token bonding curve_id
		curve_id: u64,
		/// mint
		mint_data: MintingData<T>,
	}

	#[derive(Encode, Decode, TypeInfo, Clone)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct MintingData<T: Config> {
		/// accountId of the minter
		minter: AccountOf<T>,
		/// max minting limit
		minting_cap: Option<BalanceOf<T>>,
		/// current mint amount
		current_mint_amount: Option<BalanceOf<T>>,
	}

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

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: MultiReservableCurrency<Self::AccountId>;

		/// The native currency.
		type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

		/// The deposit required for creating a new bonding curve.
		type CurveDeposit: Get<BalanceOf<Self>>;

		/// The deposit required for creating a new asset with bonding curve.
		type CreatorAssetDeposit: Get<BalanceOf<Self>>;

		/// The module/pallet identifier.
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn curves)]
	pub(super) type Curves<T: Config> =
		StorageMap<_, Twox64Concat, u64, Option<BondingCurve<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_curve_id)]
	pub(super) type NextCurveId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn assets)]
	pub(super) type Assets<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BondingCurve<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn assets_minted)]
	pub(super) type AssetsMinted<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BondingToken<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	pub(super) type Account<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		CurrencyIdOf<T>,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// (AssetId, Creator)
		NewCurve(CurrencyIdOf<T>, AccountOf<T>),
		/// (Buyer, AssetId, Amount, Cost)
		CurveBuy(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Seller, AssetId, Amount, Return)
		CurveSell(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (AssetId, Creator)
		AssetCreated(CurrencyIdOf<T>, AccountOf<T>),
		/// (Account, AssetBalance)
		AssetBalance(AccountOf<T>, BalanceOf<T>),
		/// (Buyer, AssetId, Amount, Cost)
		AssetBought(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Seller, AssetId, Amount, Return)
		AssetSell(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Minter, AssetId, MintAmount)
		AssetMinted(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Sender does not have enough base currency to reserve for a new curve.
		InsufficientBalanceToReserve,
		/// A curve does not exist for this curve id.
		CurveDoesNotExist,
		/// Sender does not have enough base currency to make a purchase.
		InsufficentBalanceForPurchase,
		/// The currency that is trying to be created already exists.
		CurrencyAlreadyExists,
		/// Error when a beneficiary doesnt have free balance
		ErrorWhileBuying,
		/// Error when an asset does not exist
		AssetDoesNotExist,
		/// Error when the mint request is greater that the max_supply of tokensp
		MintAmountGreaterThanMaxSupply,
		/// Error when you try to mint coins with different account
		InvalidMinter,
		/// Error when the token exists but there are no minted tokens
		MintUninitiated,
		/// Error when the curve type is not defined
		CurveTypeNotDefined,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// #[pallet::weight(0 + T::DbWeight::get().writes(1))]
		// pub fn create(
		// 	origin: OriginFor<T>,
		// 	asset_id: CurrencyIdOf<T>,
		// 	exponent: u32,
		// 	slope: u128,
		// 	max_supply: u128,
		// 	token_name: Vec<u8>,
		// 	token_decimals: u8,
		// 	token_symbol: Vec<u8>,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	log::info!("NativeCurrencyId: {:#?}", T::GetNativeCurrencyId::get());
		// 	// Requires an amount to be reserved.
		// 	ensure!(
		// 		T::Currency::can_reserve(
		// 			T::GetNativeCurrencyId::get(),
		// 			&sender,
		// 			T::CurveDeposit::get()
		// 		),
		// 		Error::<T>::InsufficientBalanceToReserve,
		// 	);

		// 	// Ensure that a curve with this id does not already exist.
		// 	ensure!(
		// 		T::Currency::total_issuance(asset_id) == 0u32.into(),
		// 		Error::<T>::CurrencyAlreadyExists,
		// 	);

		// 	// Adds 1 of the token to the module account.
		// 	T::Currency::deposit(
		// 		asset_id,
		// 		&T::PalletId::get().into_account(),
		// 		1u32.saturated_into(),
		// 	)?;
		// 	log::info!("total issuance {:#?}", T::Currency::total_issuance(asset_id));

		// 	// Testing to be removed later
		// 	let acc: AccountOf<T> = T::PalletId::get().into_account();
		// 	log::info!(
		// 		"free_bal account {:#?}, bal: {:#?}",
		// 		acc.clone(),
		// 		T::Currency::free_balance(asset_id, &acc)
		// 	);

		// 	let curve_id = Self::next_id();

		// 	let new_curve = BondingCurve::<T> {
		// 		creator: sender.clone(),
		// 		asset_id,
		// 		exponent,
		// 		slope,
		// 		max_supply,
		// 		token_name,
		// 		token_symbol,
		// 		token_decimals,
		// 		curve_id,
		// 	};

		// 	// Mutations start here
		// 	<Curves<T>>::insert(curve_id.clone(), Some(new_curve.clone()));
		// 	<Assets<T>>::insert(asset_id.clone(), new_curve);

		// 	Self::deposit_event(Event::NewCurve(asset_id, sender));

		// 	Ok(())
		// }

		// #[pallet::weight(0 + T::DbWeight::get().writes(1))]
		// pub fn buy(
		// 	origin: OriginFor<T>,
		// 	asset_id: CurrencyIdOf<T>,
		// 	amount: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	if let Some(asset) = Self::assets(asset_id) {
		// 		let asset_id = asset.asset_id;
		// 		let total_issuance = T::Currency::total_issuance(asset_id).saturated_into::<u128>();

		// 		log::info!("total issuance {:#?}", total_issuance.clone());

		// 		// Todo: ensure that the total_issuance > amount requested

		// 		let issuance_after = total_issuance + amount.saturated_into::<u128>();

		// 		ensure!(issuance_after <= asset.max_supply, "Exceeded max supply.",);

		// 		let integral_before: BalanceOf<T> = asset.integral(total_issuance).saturated_into();
		// 		let integral_after: BalanceOf<T> = asset.integral(issuance_after).saturated_into();

		// 		let cost = integral_after - integral_before;

		// 		log::info!(
		// 			"Buy free_bal of native_currency {:#?}, cost: {:#?}",
		// 			T::Currency::free_balance(T::GetNativeCurrencyId::get(), &sender),
		// 			cost.clone()
		// 		);
		// 		log::info!("NativeCurrencyId: {:#?}", T::GetNativeCurrencyId::get());

		// 		ensure!(
		// 			T::Currency::free_balance(T::GetNativeCurrencyId::get(), &sender)
		// 				>= cost.into(),
		// 			Error::<T>::InsufficentBalanceForPurchase,
		// 		);

		// 		let curve_account = T::PalletId::get().into_sub_account(asset.curve_id);

		// 		// Testing to be removed later
		// 		let acc: AccountOf<T> = T::PalletId::get().into_sub_account(asset.curve_id);
		// 		log::info!(
		// 			"free_bal account {:#?}, bal: {:#?}",
		// 			acc.clone(),
		// 			T::Currency::free_balance(asset_id, &acc)
		// 		);

		// 		T::Currency::transfer(
		// 			T::GetNativeCurrencyId::get(),
		// 			&sender,
		// 			&curve_account,
		// 			cost,
		// 		)?;

		// 		// Error: deposit just adds the amount to the account
		// 		// even if it doesn't exist
		// 		T::Currency::deposit(asset_id, &sender, amount)?;

		// 		log::info!(
		// 			"free_bal after token transfer requestor, {:?}",
		// 			T::Currency::free_balance(asset_id, &sender)
		// 		);
		// 		log::info!(
		// 			"free_bal after token transfer curve_account, {:?}",
		// 			T::Currency::free_balance(T::GetNativeCurrencyId::get(), &curve_account)
		// 		);

		// 		Self::deposit_event(Event::CurveBuy(sender, asset.asset_id, amount, cost));
		// 		Ok(())
		// 	} else {
		// 		Err(Error::<T>::CurveDoesNotExist)?
		// 	}
		// }

		// #[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
		// pub fn sell(
		// 	origin: OriginFor<T>,
		// 	asset_id: CurrencyIdOf<T>,
		// 	beneficiary: AccountOf<T>,
		// 	amount: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	if let Some(asset) = Self::assets(asset_id) {
		// 		let asset_id = asset.asset_id;

		// 		T::Currency::ensure_can_withdraw(asset_id, &sender, amount)?;
		// 		// T::Currency::free_balance(asset_id, &beneficiary);

		// 		let total_issuance = T::Currency::total_issuance(asset_id);
		// 		let issuance_after = total_issuance - amount;

		// 		let integral_before: BalanceOf<T> =
		// 			asset.integral(total_issuance.saturated_into::<u128>()).saturated_into();
		// 		let integral_after: BalanceOf<T> =
		// 			asset.integral(issuance_after.saturated_into::<u128>()).saturated_into();

		// 		let return_amount = integral_before - integral_after;
		// 		let curve_account = T::PalletId::get().into_sub_account(asset.curve_id);

		// 		log::info!(
		// 			"sell free_bal of native_currency {:#?}, return_amount: {:#?}",
		// 			T::Currency::free_balance(T::GetNativeCurrencyId::get(), &sender),
		// 			return_amount.clone()
		// 		);

		// 		T::Currency::withdraw(asset_id, &sender, amount)?;

		// 		log::info!("NativeCurrencyId: {:#?}", T::GetNativeCurrencyId::get());
		// 		log::info!("AssetID: {:#?}", asset_id.clone());

		// 		T::Currency::transfer(asset_id, &curve_account, &beneficiary, return_amount)?;

		// 		Self::deposit_event(Event::CurveSell(
		// 			beneficiary,
		// 			asset.asset_id,
		// 			amount,
		// 			return_amount,
		// 		));
		// 		Ok(())
		// 	} else {
		// 		Err(Error::<T>::CurveDoesNotExist)?
		// 	}
		// }

		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_asset(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			max_supply: BalanceOf<T>,
			curve_type: CurveType,
			mint: BalanceOf<T>,
			token_name: Vec<u8>,
			token_symbol: Vec<u8>,
			token_decimals: u8,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;

			// Requires an amount to be reserved.
			ensure!(
				T::Currency::can_reserve(
					T::GetNativeCurrencyId::get(),
					&sender,
					T::CreatorAssetDeposit::get()
				),
				Error::<T>::InsufficientBalanceToReserve,
			);

			// Ensure that a curve with this id does not already exist.
			ensure!(
				T::Currency::total_issuance(asset_id) == 0u32.into(),
				Error::<T>::CurrencyAlreadyExists,
			);

			// Ensure that the mint amount < max_supply
			ensure!(mint.clone() < max_supply.clone(), Error::<T>::MintAmountGreaterThanMaxSupply);

			log::info!("total issuance {:#?}", T::Currency::total_issuance(asset_id));

			log::info!("total issuance {:#?}", T::Currency::total_issuance(asset_id));

			let curve_config = match curve_type {
				CurveType::Linear => Ok(Linear::default()),
				_ => Err(Error::<T>::CurveTypeNotDefined),
			}?;
			let curve_id = Self::next_id();
			let new_mint_details = MintingData::<T> {
				minter: sender.clone(),
				minting_cap: Some(max_supply.clone()),
				current_mint_amount: None,
			};
			let new_token = BondingToken::<T> {
				creator: sender.clone(),
				asset_id,
				curve: curve_type,
				max_supply,
				token_name,
				token_symbol,
				token_decimals,
				curve_id,
				mint_data: new_mint_details,
			};
			<AssetsMinted<T>>::insert(asset_id.clone(), new_token);

			// initial_supply of the tokens is added to
			// the creator's account
			Self::mint_asset(origin.clone(), asset_id, mint)?;

			Self::deposit_event(Event::AssetCreated(asset_id, sender));

			Ok(())
		}

		#[transactional]
		#[pallet::weight(10000)]
		pub fn mint_asset(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let minter = ensure_signed(origin)?;

			if let Some(mut token) = Self::assets_minted(asset_id) {
				ensure!(minter == token.mint_data.minter, Error::<T>::InvalidMinter);

				token.mint_data.current_mint_amount = Some(amount);
				T::Currency::deposit(asset_id, &minter, amount)?;
				Self::deposit_event(Event::AssetMinted(minter, asset_id, amount));

				Ok(())
			} else {
				Err(Error::<T>::AssetDoesNotExist)?
			}
		}

		#[pallet::weight(10000)]
		pub fn buy_asset(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;

			let token = Self::assets_minted(asset_id).ok_or(<Error<T>>::AssetDoesNotExist)?;

			// Todo: ensure that the current mint amount > amount requested
			ensure!(
				amount.clone()
					< token
						.mint_data
						.current_mint_amount
						.clone()
						.ok_or(Error::<T>::MintUninitiated)?,
				<Error<T>>::MintAmountGreaterThanMaxSupply
			);

			let total_issuance = T::Currency::total_issuance(asset_id).saturated_into::<u128>();

			log::info!("total issuance {:#?}", total_issuance.clone());

			let issuance_after = total_issuance + amount.saturated_into::<u128>();
			ensure!(
				issuance_after <= token.mint_data.minting_cap.unwrap().saturated_into::<u128>(),
				"Exceeded max supply.",
			);

			let curve = match token.curve {
				CurveType::Linear(curve_data) => Ok(curve_data),
				_ => Err(Error::<T>::CurveTypeNotDefined),
			}?;

			let integral_before: BalanceOf<T> = curve.integral(total_issuance).saturated_into();
			let integral_after: BalanceOf<T> = curve.integral(issuance_after).saturated_into();

			let cost = integral_after - integral_before;

			let token_account = token.mint_data.minter;

			// Transfer the network tokens from the buyers' acoount
			// to the admin account
			T::Currency::transfer(T::GetNativeCurrencyId::get(), &buyer, &token_account, cost)?;

			// Transfer the creator tokens from the minters' acoount
			// to the buyers' account
			T::Currency::transfer(asset_id, &token_account, &buyer, amount)?;

			Self::deposit_event(Event::AssetBought(buyer, token.asset_id, amount, cost));
			Ok(())
		}

		#[pallet::weight(10000)]
		pub fn sell_asset(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			beneficiary: AccountOf<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;

			let token = Self::assets_minted(asset_id).ok_or(<Error<T>>::AssetDoesNotExist)?;

			T::Currency::ensure_can_withdraw(asset_id, &seller, amount)?;

			let total_issuance = T::Currency::total_issuance(asset_id);
			let issuance_after = total_issuance - amount;

			let curve = match token.curve {
				CurveType::Linear(curve_data) => Ok(curve_data),
				_ => Err(Error::<T>::CurveTypeNotDefined),
			}?;

			let integral_before: BalanceOf<T> =
				curve.integral(total_issuance.saturated_into::<u128>()).saturated_into();
			let integral_after: BalanceOf<T> =
				curve.integral(issuance_after.saturated_into::<u128>()).saturated_into();
			let return_amount = integral_before - integral_after;

			let token_account = token.mint_data.minter;

			// transfer network tokens from the seller to admin
			T::Currency::transfer(
				T::GetNativeCurrencyId::get(),
				&seller,
				&token_account,
				return_amount,
			)?;

			// transfer the creator tokens from the seller to beneficiary
			T::Currency::transfer(asset_id, &seller, &beneficiary, amount)?;

			Self::deposit_event(Event::AssetSell(
				beneficiary,
				token.asset_id,
				amount,
				return_amount,
			));
			Ok(())
		}

		#[pallet::weight(10000)]
		pub fn asset_balance(origin: OriginFor<T>, asset_id: CurrencyIdOf<T>) -> DispatchResult {
			let from = ensure_signed(origin)?;
			let bal = T::Currency::free_balance(asset_id, &from);
			Self::deposit_event(Event::AssetBalance(from, bal));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// DANGER - Mutates storage
	fn next_id() -> u64 {
		let id = Self::next_curve_id();
		<NextCurveId<T>>::mutate(|n| *n += 1);
		id
	}
}
