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
pub use pallet::*;
use sp_runtime::traits::UniqueSaturatedFrom;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

pub mod structs;
pub use structs::NFTStructs;

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
	pub trait Config: frame_system::Config + pallet_timestamp::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type PalletWeightInfo: WeightInfo;
		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;
		/// Nft quantity
		type Quantity: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ MaxEncodedLen;

		type NFTId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ MaxEncodedLen;

		type CollectionId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ MaxEncodedLen;

		/// The basic amount of funds that must be reserved for an asset class.
		#[pallet::constant]
		type CollectionNFTDeposit: Get<BalanceOf<Self>>;

		/// The basic amount of funds that must be reserved for an asset instance.
		#[pallet::constant]
		type NFTDeposit: Get<BalanceOf<Self>>;

		/// The basic amount of funds that must be reserved for an asset instance.
		#[pallet::constant]
		type MetaDataByteDeposit: Get<BalanceOf<Self>>;
	}

	pub trait MarketPalceHelper {
		type MarketHash;
		type OwnerAccountId;

		fn get_market_palce_info(
			owner: &Self::OwnerAccountId,
			store_hash: &Self::MarketHash,
		) -> DispatchResult;
	}

	 impl<T: Config> MarketPalceHelper for Pallet<T> {
		type MarketHash = HashId<T>;
		type OwnerAccountId = AccountOf<T>;

		fn get_market_palce_info(
			owner: &Self::OwnerAccountId,
			store_hash: &Self::MarketHash,
		) -> DispatchResult {
			let market_place = MarketplaceStorage::<T>::get(owner, store_hash)
				.ok_or(Error::<T>::MarketNotFound)?;
			Ok(())
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
		},
		MarketplaceOwnerChanges {
			old_owner: AccountOf<T>,
			new_owner: AccountOf<T>,
			price: BalanceOf<T>,
			hash_id: HashId<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MarketNotFound,
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
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_create_market_place(issuer, metadata, fee, export_fee, import_fee)
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
	}

	impl<T: Config> Pallet<T> {
		#[transactional]
		pub fn do_create_market_place(
			issuer: T::AccountId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			fee: BalanceOf<T>,
			export_fee: BalanceOf<T>,
			import_fee: BalanceOf<T>,
		) -> DispatchResult {
			let hash_id = T::Hashing::hash_of(&metadata);

			let market_place = NFTStructs::Marketplace {
				fee,
				issuer: issuer.clone(),
				owner: issuer.clone(),
				metadata,
				hash_id: hash_id.clone(),
				export_fee: export_fee.clone(),
				import_fee: import_fee.clone(),
			};

			MarketplaceStorage::<T>::insert(issuer.clone(), hash_id.clone(), market_place);

			Self::deposit_event(Event::CreateMarketPlace {
				fee,
				issuer: issuer.clone(),
				owner: issuer,
				hash_id: hash_id.clone(),
				export_fee: export_fee.clone(),
				import_fee: import_fee.clone(),
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

			MarketplaceStorage::<T>::insert(&new_owner, &store_hash_id, new_market_info.clone());

			Self::deposit_event(Event::MarketplaceOwnerChanges {
				old_owner: original_owner,
				new_owner: new_owner.clone(),
				price,
				hash_id: store_hash_id,
			});

			Ok(())
		}
	}
}
