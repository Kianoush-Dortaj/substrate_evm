#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
use codec::{alloc::vec, Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{
	inherent::Vec,
	pallet_prelude::{ValueQuery, *},
	sp_runtime::{
		traits::{AtLeast32BitUnsigned, CheckedAdd, Hash, Member, One},
		DispatchError, SaturatedConversion,
	},
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
	transactional, Twox64Concat,
};
use frame_system::Config as SystemConfig;
use sp_runtime::traits::UniqueSaturatedFrom;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

pub mod structs;
pub use structs::MarketPlaceStructs;

pub mod types;
pub use types::Types::{AccountOf, BalanceOf, MarketPlace};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	type HashId<T> = <T as frame_system::Config>::Hash;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type PalletWeightInfo: WeightInfo;
		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	pub trait MarketPalceHelper {
		type MarketHash;
		type UserAccountId;
		type Balance;

		fn send_fee_to_market_place_owner(
			issuer: &Self::UserAccountId,
			owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
		) -> DispatchResult;

		fn check_allow_royalty(
			store_owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
			royalty_fee: u64,
		) -> DispatchResult;

		fn get_market_place_fee(
			issuer: &Self::UserAccountId,
			store_id: &Self::MarketHash,
		) -> Result<(u64, u64), DispatchError>;

		fn send_royalty_fee_to_market_place_owner(
			issuer: &Self::UserAccountId,
			owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
			fee: &Self::Balance,
		) -> DispatchResult;
	}

	impl<T: Config> MarketPalceHelper for Pallet<T> {
		type MarketHash = T::Hash;
		type UserAccountId = T::AccountId;
		type Balance =
			<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

		fn send_fee_to_market_place_owner(
			issuer: &Self::UserAccountId,
			owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
		) -> DispatchResult {
			let market_place_info = MarketplaceStorage::<T>::get(owner, store_hash)
				.ok_or(Error::<T>::MarketNotFound)?;

			let _ = T::Currency::transfer(
				&issuer,
				&market_place_info.owner,
				market_place_info.fee,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| Error::<T>::ErrorTransferMarketPlaceFee)?;

			Ok(())
		}

		fn send_royalty_fee_to_market_place_owner(
			issuer: &Self::UserAccountId,
			owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
			fee: &Self::Balance,
		) -> DispatchResult {
			let market_place_info = MarketplaceStorage::<T>::get(owner, store_hash)
				.ok_or(Error::<T>::MarketNotFound)?;

			let _ = T::Currency::transfer(
				&issuer,
				&market_place_info.owner,
				*fee,
				ExistenceRequirement::AllowDeath,
			)
			.map_err(|_| Error::<T>::ErrorTransferMarketPlaceFee)?;

			Ok(())
		}

		fn check_allow_royalty(
			store_owner: &Self::UserAccountId,
			store_hash: &Self::MarketHash,
			royalty_fee: u64,
		) -> DispatchResult {
			let market_place_info = MarketplaceStorage::<T>::get(store_owner, store_hash)
				.ok_or(Error::<T>::MarketNotFound)?;

			ensure!(
				market_place_info.max_royalty >= royalty_fee,
				Error::<T>::NotAllowSetRoyaltyOverThanStoreConfig
			);

			Ok(())
		}

		fn get_market_place_fee(
			issuer: &Self::UserAccountId,
			store_id: &Self::MarketHash,
		) -> Result<(u64, u64), DispatchError> {
			let market_place_info =
				MarketplaceStorage::<T>::get(issuer, store_id).ok_or(Error::<T>::MarketNotFound)?;

			Ok((market_place_info.max_royalty, market_place_info.royalty_fee))
		}
	}
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateMarketPlace {
			owner: AccountOf<T>,
			issuer: AccountOf<T>,
			fee: BalanceOf<T>,
			hash_id: HashId<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		},
		MarketplaceOwnerChanges {
			old_owner: AccountOf<T>,
			new_owner: AccountOf<T>,
			price: BalanceOf<T>,
			hash_id: HashId<T>,
		},
		UpdateMarketPlace {
			owner: AccountOf<T>,
			fee: BalanceOf<T>,
			hash_id: HashId<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MarketNotFound,
		ErrorTransferMarketPlaceFee,
		NotAllowSetRoyaltyOverThanStoreConfig,
	}

	/// Store nft info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn market_place)]
	pub type MarketplaceStorage<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		AccountOf<T>,
		Twox64Concat,
		HashId<T>,
		MarketPlace<T>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(1)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn create_market_place(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			fee: BalanceOf<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_create_market_place(
				issuer,
				metadata,
				fee,
				export_fee,
				import_fee,
				max_royalty,
				royalty_fee,
			)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn change_owner(
			origin: OriginFor<T>,
			new_owner: T::AccountId,
			store_hash_id: HashId<T>,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_change_owner(issuer, new_owner, store_hash_id, price)
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn change_marketplace_info(
			origin: OriginFor<T>,
			store_hash_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			fee: BalanceOf<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_change_marketplace_info(
				issuer,
				store_hash_id,
				metadata,
				fee,
				export_fee,
				import_fee,
				max_royalty,
				royalty_fee,
			)
		}
	}

	impl<T: Config> Pallet<T> {
		#[transactional]
		pub fn do_create_market_place(
			issuer: T::AccountId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			fee: BalanceOf<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		) -> DispatchResult {
			let hash_id = T::Hashing::hash_of(&metadata);

			let market_place = MarketPlaceStructs::Marketplace {
				fee,
				issuer: issuer.clone(),
				owner: issuer.clone(),
				metadata,
				hash_id: hash_id.clone(),
				export_fee: export_fee.clone(),
				import_fee: import_fee.clone(),
				max_royalty: max_royalty.clone(),
				royalty_fee: royalty_fee.clone(),
			};

			MarketplaceStorage::<T>::insert(issuer.clone(), hash_id.clone(), market_place);

			Self::deposit_event(Event::CreateMarketPlace {
				fee,
				issuer: issuer.clone(),
				owner: issuer,
				hash_id: hash_id.clone(),
				export_fee: export_fee.clone(),
				import_fee: import_fee.clone(),
				max_royalty: max_royalty.clone(),
				royalty_fee: royalty_fee.clone(),
			});

			Ok(())
		}

		#[transactional]
		pub fn do_change_owner(
			original_owner: T::AccountId,
			new_owner: T::AccountId,
			store_hash_id: HashId<T>,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let market_info = MarketplaceStorage::<T>::take(&original_owner, &store_hash_id)
				.ok_or(Error::<T>::MarketNotFound)?;

			let mut new_market_info = market_info;
			new_market_info.owner = new_owner.clone();

			T::Currency::transfer(
				&new_owner,
				&original_owner,
				price,
				ExistenceRequirement::KeepAlive,
			)?;

			MarketplaceStorage::<T>::insert(&new_owner, &store_hash_id, new_market_info.clone());

			Self::deposit_event(Event::MarketplaceOwnerChanges {
				old_owner: original_owner,
				new_owner: new_owner.clone(),
				price,
				hash_id: store_hash_id,
			});

			Ok(())
		}

		#[transactional]
		pub fn do_change_marketplace_info(
			issuer: T::AccountId,
			store_hash_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			fee: BalanceOf<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
			max_royalty: u64,
			royalty_fee: u64,
		) -> DispatchResult {
			MarketplaceStorage::<T>::try_mutate(
				issuer.clone(),
				store_hash_id.clone(),
				|market_info| -> Result<(), DispatchError> {
					match market_info {
						Some(info) => {
							info.fee = fee.clone();
							info.metadata = metadata.clone();
							info.export_fee = export_fee.clone();
							info.import_fee = import_fee.clone();
							info.max_royalty = max_royalty.clone();
							info.royalty_fee = royalty_fee.clone();
							Ok(())
						},
						None => Err(Error::<T>::MarketNotFound.into()),
					}
				},
			)?;

			Self::deposit_event(Event::UpdateMarketPlace {
				fee,
				owner: issuer,
				hash_id: store_hash_id,
				export_fee,
				import_fee,
				max_royalty,
				royalty_fee: royalty_fee.clone(),
			});

			Ok(())
		}
	}
}
