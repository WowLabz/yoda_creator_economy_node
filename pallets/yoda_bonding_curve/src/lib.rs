#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::traits::{Currency, ExistenceRequirement, Randomness, Get};
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use orml_traits::{MultiCurrency, MultiReservableCurrency};
	use scale_info::TypeInfo;
	use frame_support::PalletId;
	use sp_runtime::traits::{AccountIdConversion, SaturatedConversion};

	type BalanceOf<T> =
		<<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<
		<T as frame_system::Config>::AccountId,
	>>::CurrencyId;

	#[derive(Encode, Decode, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct BondingCurve<T: Config> {
		// The module-owned account for this bonding curve.
		// account: AccountId,
		/// The creator of the bonding curve.
		creator: AccountOf<T>,
		/// The currency id of the bonding curve token.
		currency_id: CurrencyIdOf<T>,
		/// The exponent of the curve.
		exponent: u32,
		/// The slope of the curve.
		slope: u128,
		/// The maximum supply that can be minted from the curve.
		max_supply: u128,
	}

	impl<T: Config> BondingCurve::<T> {
		/// Integral when the curve is at point `x`.
		pub fn integral(&self, x: u128) -> u128 {
			let nexp = self.exponent + 1;
			x.pow(nexp) * self.slope / nexp as u128
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

		/// The module/pallet identifier.
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn curves)]
	pub(super) type Curves<T: Config> = StorageMap<_, Twox64Concat, u64, Option<BondingCurve<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_curve_id)]
	pub(super) type NextCurveId<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// (CurveId, Creator)
        NewCurve(u64, AccountOf<T>),
        /// (Buyer, CurveId, Amount, Cost)
        CurveBuy(AccountOf<T>, u64, BalanceOf<T>, BalanceOf<T>),
        /// (Seller, CurveId, Amount, Return)
        CurveSell(AccountOf<T>, u64, BalanceOf<T>, BalanceOf<T>),
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
		/// Error while using deposit from orml currencies
		ErrorWhileDeposit,
		/// Error while using ensure_withdraw from orml currencies
		ErrorWhileWithdraw
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0 + T::DbWeight::get().writes(1))]
		pub fn create(origin: OriginFor<T>, currency_id: CurrencyIdOf<T>, exponent: u32, slope: u128, max_supply: u128) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Requires an amount to be reserved.
            ensure!(
                T::Currency::can_reserve(T::GetNativeCurrencyId::get(), &sender, T::CurveDeposit::get()),
                Error::<T>::InsufficientBalanceToReserve,
            );

			// Ensure that a curve with this id does not already exist.
            ensure!(
                T::Currency::total_issuance(currency_id) == 0u32.into(),
                Error::<T>::CurrencyAlreadyExists,
            );

			// Adds 1 of the token to the module account.
            T::Currency::deposit(currency_id, &T::PalletId::get().into_account(), 1u32.saturated_into()).map_err(|e| <Error<T>>::ErrorWhileWithdraw)?;

			 let new_curve = BondingCurve::<T> {
                creator: sender.clone(),
                currency_id,
                exponent,
                slope,
                max_supply,
            };

			// Mutations start here
            let curve_id = Self::next_id();
            <Curves<T>>::insert(curve_id, Some(new_curve));

			Self::deposit_event(Event::NewCurve(curve_id, sender));

			Ok(())
		}

		#[pallet::weight(0 + T::DbWeight::get().writes(1))]
		pub fn buy(origin: OriginFor<T>, curve_id: u64, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			if let Some(curve) = Self::curves(curve_id) {
                let currency_id = curve.currency_id;
                let total_issuance = T::Currency::total_issuance(currency_id).saturated_into::<u128>();
                let issuance_after = total_issuance + amount.saturated_into::<u128>();

                ensure!(
                    issuance_after <= curve.max_supply,
                    "Exceeded max supply.",
                );

                let integral_before: BalanceOf<T> = curve.integral(total_issuance).saturated_into();
                let integral_after: BalanceOf<T> = curve.integral(issuance_after).saturated_into();
                
                let cost = integral_after - integral_before;
                ensure!(
                    T::Currency::free_balance(T::GetNativeCurrencyId::get(), &sender) >= cost.into(),
                    Error::<T>::InsufficentBalanceForPurchase,
                );
				let curve_account = T::PalletId::get().into_sub_account(curve_id);

                T::Currency::transfer(T::GetNativeCurrencyId::get(), &sender, &curve_account, cost).ok(); // <- Why does the `?` operator not work?

                T::Currency::deposit(currency_id, &sender, amount).ok();

                Self::deposit_event(Event::CurveBuy(sender, curve_id, amount, cost));
				Ok(())
            } else {
                Err(Error::<T>::CurveDoesNotExist)?
            }
		}

		#[pallet::weight(0 + T::DbWeight::get().reads_writes(1,1))]
		pub fn sell(origin: OriginFor<T>, curve_id: u64, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			  if let Some(curve) = Self::curves(curve_id) {
                let currency_id = curve.currency_id;

                T::Currency::ensure_can_withdraw(currency_id, &sender, amount).map_err(|e| <Error<T>>::ErrorWhileWithdraw)?;
                
                let total_issuance = T::Currency::total_issuance(currency_id);
                let issuance_after = total_issuance - amount;

                let integral_before: BalanceOf<T> = curve.integral(total_issuance.saturated_into::<u128>()).saturated_into();
                let integral_after: BalanceOf<T> = curve.integral(issuance_after.saturated_into::<u128>()).saturated_into();

                let return_amount = integral_before - integral_after;

				let curve_account = T::PalletId::get().into_sub_account(curve_id);

                T::Currency::withdraw(currency_id, &sender, amount).ok();

                T::Currency::transfer(T::GetNativeCurrencyId::get(), &curve_account, &sender, return_amount).ok();

                Self::deposit_event(Event::CurveSell(sender, curve_id, amount, return_amount));
				Ok(())
            } else {
                Err(Error::<T>::CurveDoesNotExist)?
            }
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