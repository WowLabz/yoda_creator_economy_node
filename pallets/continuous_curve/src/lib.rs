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
	use frame_support::traits::{
		tokens::currency::Currency, ExistenceRequirement, Get, Randomness, ReservableCurrency,
	};
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
	pub type ReserveBalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
	pub trait Config: frame_system::Config + pallet_balances::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: MultiReservableCurrency<Self::AccountId>;

		type ReserveCurrency: ReservableCurrency<Self::AccountId>;

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
		pub fn create_token(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			max_supply: BalanceOf<T>,
			curve_type: CurveType,
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
			// ensure!(mint.clone() < max_supply.clone(), Error::<T>::MintAmountGreaterThanMaxSupply);

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
			Self::mint(origin.clone(), asset_id, 1u32.saturated_into())?;

			Self::deposit_event(Event::AssetCreated(asset_id, sender));
			Ok(())
		}

		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn mint(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			reserve_amt: BalanceOf<T>,
		) -> DispatchResult {
			let minter = ensure_signed(origin.clone())?;

			if let Some(mut token) = Self::assets_minted(asset_id) {
				const INITIAL_MINT_AMT: u32 = 1;
				let current_total_issuance = T::Currency::total_issuance(asset_id);

				if current_total_issuance == 0u32.saturated_into() {
					// adding 1 continouis token into the minters account
					T::Currency::deposit(asset_id, &minter, INITIAL_MINT_AMT.saturated_into())?;
					token.mint_data.current_mint_amount = Some(INITIAL_MINT_AMT.saturated_into());
				}

				let reserve_balance = T::ReserveCurrency::reserved_balance(&minter);
				log::info!("before_reserve_balance {:#?}", reserve_balance.clone());

				if reserve_balance == 0u32.saturated_into() {
					let test_bal = 1u128;
					T::ReserveCurrency::reserve(&minter, test_bal.saturated_into())?;
				}

				let reserve_balance = T::ReserveCurrency::reserved_balance(&minter);
				log::info!("pre_function_reserve_balance {:#?}", reserve_balance.clone());

				let (n, _d) = token.curve.get_reserve_ratio();
				// let reserve_ratio = sp_runtime::Permill::from_rational(50u32, 100u32);

				let current_token_model = CalculatePurchaseAndSellReturn {
					supply: current_total_issuance.saturated_into(),
					reserve_balance: reserve_balance.saturated_into(),
					reserve_ratio: 1u128,
					deposit_amount: reserve_amt.saturated_into(),
					power: Power { base_N: 20, base_D: 10, exp_N: 2, exp_D: 1 },
				};

				log::info!("current_token_model {:#?}", current_token_model.clone());

				// equilent amount of continous tokens for the reserved amount
				// of reserve token (native token)
				let calculated_purchase_return = CalculatePurchaseAndSellReturn::purchase_return(
					current_token_model,
					current_total_issuance.saturated_into(),
					reserve_balance.saturated_into(),
					50u128,
					reserve_amt.saturated_into(),
				);
				log::info!("calculated_tokens_to_mint {:#?}", calculated_purchase_return.clone());

				// the reserved amount of the reserve token is locked/reserved
				// into the minter's account
				let res_bal = reserve_amt.saturated_into::<u128>();
				T::ReserveCurrency::reserve(&minter, res_bal.saturated_into())?;

				let reserve_balance = T::ReserveCurrency::reserved_balance(&minter);
				log::info!("after_transaction_reserve_balance {:#?}", reserve_balance.clone());

				let free_bal = T::ReserveCurrency::free_balance(&minter);
				log::info!("free_balance {:#?}", free_bal.clone());

				// T::ReserveCurrency::set_balance(minter.clone(), 2, 2);
				// pallet_balances::Pallet::set_balance(origin, minter.clone(), 2, 2);

				// minting the continous token to the minter's account
				T::Currency::deposit(
					asset_id,
					&minter,
					calculated_purchase_return.saturated_into(),
				)?;

				if let Some(prev_amt) = token.mint_data.current_mint_amount {
					let new_amt = CheckedAdd::checked_add(
						&prev_amt,
						&calculated_purchase_return.saturated_into(),
					)
					.ok_or(Error::<T>::MintAmountOverflow)?;
					token.mint_data.current_mint_amount = Some(new_amt);
				} else {
					token.mint_data.current_mint_amount =
						Some(calculated_purchase_return.saturated_into());
				}

				<AssetsMinted<T>>::insert(asset_id.clone(), token);
				Self::deposit_event(Event::AssetMinted(
					minter,
					asset_id,
					calculated_purchase_return.saturated_into(),
				));

				Ok(())
			} else {
				Err(Error::<T>::AssetDoesNotExist)?
			}
		}

		#[pallet::weight(10000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn burn(
			origin: OriginFor<T>,
			asset_id: CurrencyIdOf<T>,
			burn_amt: BalanceOf<T>,
		) -> DispatchResult {
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
