#![cfg_attr(not(feature = "std"), no_std)]

use codec::{alloc::vec, Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::sp_runtime::{
	traits::{AtLeast32BitUnsigned,Hash, CheckedAdd, Member, One},
	DispatchError, SaturatedConversion,
};
use sp_runtime::traits::UniqueSaturatedFrom;

use frame_support::{
	inherent::Vec,
	pallet_prelude::{ValueQuery, *},
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
	transactional, Twox64Concat,
};
use frame_system::Config as SystemConfig;
pub use pallet::*;

use nft_gallery::MarketPalceHelper;

pub mod structs;
pub use structs::NFTStructs::Collection;

pub mod types;
pub use types::Types::{AccountOf, BalanceOf, CollectionDetailsOf, HashId};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type PalletWeightInfo: WeightInfo;
		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;

		type NFTGallery: nft_gallery::MarketPalceHelper<
			MarketHash = Self::Hash,
			UserAccountId = Self::AccountId,
		>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An nft Collection was created.
		CreatedCollection { store_id: HashId<T>, collection_id: HashId<T>, issuer: AccountOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		CollectionNotFound,
	}

	/// Store collection info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collections)]
	pub(super) type Collections<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Twox64Concat, AccountOf<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
		),
		CollectionDetailsOf<T>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
			store_hash_id: HashId<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&market_owner_address,
				&store_hash_id,
			)?;

			Self::do_create_collection(issuer, metadata, store_hash_id)
		}
	}

	impl<T: Config> Pallet<T> {
		#[transactional]
		fn do_create_collection(
			issuer: T::AccountId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			store_hash_id: HashId<T>,
		) -> DispatchResult {

			let collection_hash_id = T::Hashing::hash_of(&metadata);

			let collection_details = Collection {
				collection_id: collection_hash_id.clone(),
				metadata,
				issuer: issuer.clone(),
			};

			Collections::<T>::insert(
				issuer.clone(),
				collection_hash_id.clone(),
				store_hash_id.clone(),
				collection_details.clone(),
			);

			Self::deposit_event(Event::CreatedCollection {
				store_id: store_hash_id,
				collection_id: collection_hash_id,
				issuer: issuer.clone(),
			});
			Ok(().into())
		}
	}
}
