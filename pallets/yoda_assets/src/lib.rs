#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

// Re-export to use implementation details in dependent crates:
pub use pallet_assets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	// use frame_support::traits::{Currency, ExistenceRequirement, Randomness, UnfilteredDispatchable};
	// use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	// use frame_system::pallet_prelude::*;
	// use sp_runtime::traits::StaticLookup;

	// use pallet_assets::WeightInfo;

	// use frame_support::dispatch::Dispatchable;
	use frame_support::pallet_prelude::*;
	// use frame_support::traits::IsSubType;
	// use frame_support::weights::GetDispatchInfo;
	// use frame_support::weights::PostDispatchInfo;
	use frame_support::{
		traits::{Currency, ExistenceRequirement, UnfilteredDispatchable, WithdrawReasons},
		transactional,
	};
	use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};
	use frame_system::{pallet_prelude::*, RawOrigin};
	use sp_runtime::traits::{One, StaticLookup, Zero};
	use sp_std::{prelude::*, vec::Vec};

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type AssetsAssetIdOf<T> = <T as pallet_assets::Config>::AssetId;
	pub(crate) type AssetsBalanceOf<T> = <T as pallet_assets::Config>::Balance;
	type AssetsWeightInfoOf<T> = <T as pallet_assets::Config>::WeightInfo;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//type Currency: Currency<Self::AccountId>;

		// type Call: frame_support::traits::IsSubType<pallet_assets::Call<Self>>
		// 	+ Parameter
		// 	+ Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo>
		// 	+ GetDispatchInfo
		// 	+ From<frame_system::pallet::Call<Self>>
		// 	+ UnfilteredDispatchable<Origin = Self::Origin>
		// 	+ frame_support::dispatch::Codec
		// 	+ IsSubType<Call<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_asset(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			admin: <T::Lookup as StaticLookup>::Source,
			min_balance: AssetsBalanceOf<T>,
			token_name: Vec<u8>,
			token_symbol: Vec<u8>,
			token_decimals: u8,
		) -> DispatchResult {
			let _account = ensure_signed(origin.clone())?;
			pallet_assets::Pallet::<T>::create(origin.clone(), asset_id, admin, min_balance)?;

			// Todo: based the creators "YODA" balance we limt the total supply
			// for the creator token

			pallet_assets::Pallet::<T>::set_metadata(
				origin,
				asset_id,
				token_name,
				token_symbol,
				token_decimals,
			)

			// if let IsSubType::is_sub_type(pallet_assets::pallet::Call::create { .. }) = Some(_) {
			// 	// skip
			// 	// todo!()
			// } else {
			// 	// default impl
			// 	// let (_fee, imbalance) = self.withdraw_fee(who, call, info, len)?;
			// 	// Ok((self.0, who.clone(), imbalance))
			// }
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn asset_mint(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			beneficiary: <T::Lookup as StaticLookup>::Source,
			amount: T::Balance,
		) -> DispatchResult {
			let _account = ensure_signed(origin.clone())?;
			pallet_assets::Pallet::<T>::mint(origin, asset_id, beneficiary, amount)
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn asset_buy(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			admin: <T::Lookup as StaticLookup>::Source,
			beneficiary: <T::Lookup as StaticLookup>::Source,
			tokens: AssetsBalanceOf<T>,
		
		) -> DispatchResult {
			let _account = ensure_signed(origin.clone())?;
			pallet_assets::Pallet::<T>::transfer(origin.clone(), asset_id, beneficiary, tokens)

			// Todo: based the creators "YODA" balance we limt the total supply
			// for the creator token

			
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn asset_sell(
			origin: OriginFor<T>,
			#[pallet::compact] asset_id: T::AssetId,
			admin: <T::Lookup as StaticLookup>::Source,
			beneficiary: <T::Lookup as StaticLookup>::Source,
			tokens: AssetsBalanceOf<T>,
		
		) -> DispatchResult {
			let _account = ensure_signed(origin.clone())?;
			pallet_assets::Pallet::<T>::transfer(origin.clone(), asset_id, beneficiary, tokens)

			// Todo: based the creators "YODA" balance we limt the total supply
			// for the creator token

			
		}
	}
	
}
