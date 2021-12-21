#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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
	use scale_info::prelude::boxed::Box;
	use scale_info::TypeInfo;
	use sp_runtime::traits::{AccountIdConversion, CheckedAdd, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct BondingToken<T: Config> {
		// The module-owned account for this bonding curve.
		// account: AccountId,
		/// The creator of the bonding curve.
		pub(super) creator: AccountOf<T>,
		/// The asset id of the bonding curve token.
		pub(super) asset_id: CurrencyIdOf<T>,
		/// bonding curve type with the config
		pub(super) curve: CurveType,
		/// The maximum supply that can be minted from the curve.
		pub(super) max_supply: BalanceOf<T>,
		/// the token name
		pub(super) token_name: Vec<u8>,
		/// The token symbol
		pub(super) token_symbol: Vec<u8>,
		/// Token decimals
		pub(super) token_decimals: u8,
		/// token bonding curve_id
		pub(super) curve_id: u64,
		/// mint
		pub(super) mint_data: MintingData<T>,
	}

	impl<T: Config> BondingToken<T> {
		fn get_curve_config(&self) -> Result<Box<dyn CurveConfig>, DispatchError> {
			let curve_config = match self.curve {
				CurveType::Linear => Ok(Linear::default()),
				_ => Err(Error::<T>::CurveTypeNotDefined),
			}?;
			Ok(Box::new(curve_config))
		}
	}

	#[derive(Encode, Decode, TypeInfo, Clone, PartialEq)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct MintingData<T: Config> {
		/// accountId of the minter
		pub(super) minter: AccountOf<T>,
		/// max minting limit
		pub(super) minting_cap: Option<BalanceOf<T>>,
		/// current mint amount
		pub(super) current_mint_amount: Option<BalanceOf<T>>,
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
	#[pallet::getter(fn next_curve_id)]
	pub(super) type NextCurveId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn assets_minted)]
	pub(super) type AssetsMinted<T: Config> =
		StorageMap<_, Twox64Concat, CurrencyIdOf<T>, BondingToken<T>, OptionQuery>;

	// Todo: Storage integration with the pallet_calls
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
		AssetCreated(CurrencyIdOf<T>, AccountOf<T>),
		/// (Account, AssetBalance)
		AssetBalance(AccountOf<T>, BalanceOf<T>),
		/// (Buyer, AssetId, Amount, Cost)
		AssetBought(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Seller, AssetId, Amount, Return)
		AssetSell(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>, BalanceOf<T>),
		/// (Minter, AssetId, MintAmount)
		AssetMinted(AccountOf<T>, CurrencyIdOf<T>, BalanceOf<T>),
		/// (AssetId, Amount, FromAccount, ToAccounts)
		AssetTokensAirDropped(CurrencyIdOf<T>, BalanceOf<T>, AccountOf<T>, Vec<AccountOf<T>>),
		/// (AssetId, Amount)
		AssetCurrentPrice(CurrencyIdOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Sender does not have enough base currency to reserve for a new curve.
		InsufficientBalanceToReserve,
		/// Sender does not have enough base currency to make a purchase.
		InsufficentBalanceForPurchase,
		/// The currency that is trying to be created already exists.
		AssetAlreadyExists,
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
		/// Error when there is a overflow in the mint amounts
		MintAmountOverflow,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
				Error::<T>::AssetAlreadyExists,
			);

			// Ensure that the mint amount < max_supply
			ensure!(mint.clone() < max_supply.clone(), Error::<T>::MintAmountGreaterThanMaxSupply);

			log::info!("total issuance {:#?}", T::Currency::total_issuance(asset_id));

			log::info!("total issuance {:#?}", T::Currency::total_issuance(asset_id));

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

				if let Some(prev_amt) = token.mint_data.current_mint_amount {
					let new_amt = CheckedAdd::checked_add(&prev_amt, &amount)
						.ok_or(Error::<T>::MintAmountOverflow)?;
					token.mint_data.current_mint_amount = Some(new_amt);
				} else {
					token.mint_data.current_mint_amount = Some(amount);
				}

				T::Currency::deposit(asset_id, &minter, amount)?;

				<AssetsMinted<T>>::insert(asset_id.clone(), token);
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

			let curve = token.get_curve_config()?;

			let integral_before: BalanceOf<T> =
				curve.integral_before(total_issuance).saturated_into();
			let integral_after: BalanceOf<T> =
				curve.integral_after(issuance_after).saturated_into();

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

			let curve = token.get_curve_config()?;

			let integral_before: BalanceOf<T> =
				curve.integral_before(total_issuance.saturated_into::<u128>()).saturated_into();
			let integral_after: BalanceOf<T> =
				curve.integral_after(issuance_after.saturated_into::<u128>()).saturated_into();
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
		pub fn air_drop(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			beneficiaries: Vec<AccountOf<T>>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin)?;

			let token = Self::assets_minted(asset_id).ok_or(<Error<T>>::AssetDoesNotExist)?;
			let total_withdraw_amount: BalanceOf<T> =
				(amount.saturated_into::<u128>() * beneficiaries.len() as u128).saturated_into();

			T::Currency::ensure_can_withdraw(
				asset_id,
				&token.mint_data.minter,
				total_withdraw_amount,
			)?;

			for beneficiary in &beneficiaries {
				T::Currency::transfer(asset_id, &token.mint_data.minter, beneficiary, amount)?;
			}

			Self::deposit_event(Event::AssetTokensAirDropped(
				token.asset_id,
				amount,
				token.mint_data.minter,
				beneficiaries.clone(),
			));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn current_asset_price(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
		) -> DispatchResult {
			let _caller = ensure_signed(origin.clone())?;
			// base_price = 0.01 YODA
			// market_cap_at_genesis = 0.01 * max_token_supply
			// 0.01 * 10000 = 100
			// spot_price = market_cap / total_assets_minted
			// 100 / 1000 = 0.1
			// market_cap = spot_price * max_token_supply
			// 0.1 * 10000 = 1000
			// spot_price = market_cap / total_assets_minted
			// 1000 / 2000 = 0.5
			// market_cap = total_supply(total_issuance) * price;
			// price = market_cap / circulating_supply (minted you can say)

			// const BASE_PRICE: u128 = 1;
			// const CHECK_PRICE: u128 = 0;

			let token = Self::assets_minted(asset_id).ok_or(<Error<T>>::AssetDoesNotExist)?;
			// let max_token_supply = token.mint_data.minting_cap.unwrap_or(0u128.saturated_into());
			// let total_assets_minted =
			// 	T::Currency::total_issuance(asset_id).saturated_into::<u128>();
			// 	if BASE_PRICE >= 1  {
			// 		let market_cap_at_genesis = BASE_PRICE * max_token_supply.saturated_into::<u128>();
			// 		let spot_price_at_genesis = market_cap_at_genesis / total_assets_minted;
			// 	} else if BASE_PRICE < spot_price_at_genesis {
			// 		let market_cap = spot_price_at_genesis * max_token_supply.saturated_into::<u128>();
			// 		let spot_price = market_cap / total_assets_minted;
			// 	}
			let curve = token.get_curve_config()?;
			let total_issuance = T::Currency::total_issuance(asset_id).saturated_into::<u128>();
			let current_price: BalanceOf<T> =
				curve.integral_before(total_issuance).saturated_into();

			Self::deposit_event(Event::AssetCurrentPrice(asset_id, current_price));
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
