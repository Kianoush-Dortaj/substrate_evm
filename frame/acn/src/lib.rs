#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use sp_std::vec::Vec;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
use codec::{alloc::vec, HasCompact, MaxEncodedLen};
use frame_support::{
	pallet_prelude::{ValueQuery, *},
	sp_runtime::traits::{AtLeast32BitUnsigned, Hash, Member},
	traits::{Currency, ExistenceRequirement, ReservableCurrency},
};
use frame_system::Config as SystemConfig;
use pallet_nfts::NFTHelper;
use sp_arithmetic::traits::Zero;
use sp_runtime::{
	traits::{CheckedAdd, One, Saturating, StaticLookup},
	DispatchError, Perbill, RuntimeDebug, SaturatedConversion,
};
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub struct NFTAuction<AccountId, CollectionId, NFTId, Balance> {
		pub collection_id: CollectionId,
		pub nft_id: NFTId,
		pub issuer: AccountId,
		pub start_price: Balance,
		pub highest_bid: Balance,
		pub total_supply: u64,
		pub highest_bidder: AccountId,
		pub deposit: Balance,
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type HashId<T> = <T as frame_system::Config>::Hash;

	pub type NFTAuctionOf<T> = NFTAuction<
		<T as SystemConfig>::AccountId,
		<T as pallet_nfts::Config>::CollectionId,
		<T as pallet_nfts::Config>::NFTId,
		BalanceOf<T>,
	>;

	type Key<T> = (AccountIdOf<T>, <T as frame_system::Config>::Index);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_nfts::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;
		type NFTsPallet: pallet_nfts::NFTHelper<
			AccountId = Self::AccountId,
			CollectionId = Self::CollectionId,
			NFTId = Self::NFTId,
		>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn nft_auctions)]
	pub(super) type NFTAuctions<T: Config> =
		StorageMap<_, Twox64Concat, Key<T>, NFTAuctionOf<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn bids)]
	pub(super) type Bids<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		Key<T>,
		Twox64Concat,
		HashId<T>,
		(Key<T>, BalanceOf<T>),
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored {
			something: u32,
			who: T::AccountId,
		},
		AuctionCreated {
			auction_key: Key<T>,
			start_price: BalanceOf<T>,
		},
		Bid {
			auction_key: Key<T>,
			bid_key: Key<T>,
			price: BalanceOf<T>,
			hash: HashId<T>,
		},
		Confirmed {
			auction_key: Key<T>,
		},
		Retracted {
			auction_key: Key<T>,
			bid_key: Key<T>,
			price: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		AuctionKeyNotFound,
		InvalidNextAuctionId,
		AuctionAssigned,
		OwnerRequired,
		OriginProhibited,
		AuctionNotAssigned,
		TopBidRequired,
		InsufficientBalance,
		PriceTooLow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn create_auction(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			nft_id: T::NFTId,
			start_price: BalanceOf<T>,
			total_supply: u64,
			deposit: BalanceOf<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			T::NFTsPallet::has_permission_to_add_nft_in_Auction(
				&issuer,
				&collection_id,
				&nft_id,
				total_supply,
			)?;

			let auction = NFTAuction {
				collection_id,
				nft_id,
				issuer: issuer.clone(),
				start_price,
				highest_bid: start_price,
				highest_bidder: issuer.clone(),
				total_supply,
				deposit,
			};

			let nonce = frame_system::Pallet::<T>::account_nonce(&issuer);
			let auction_key = (issuer.clone(), nonce);

			NFTAuctions::<T>::insert(&auction_key, auction);

			Self::deposit_event(Event::<T>::AuctionCreated { auction_key, start_price });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn extend(
			origin: OriginFor<T>,
			auction_key: Key<T>,
			new_price: BalanceOf<T>,
			hash_id: Option<HashId<T>>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;
			let mut auction =
				NFTAuctions::<T>::get(&auction_key).ok_or(Error::<T>::AuctionKeyNotFound)?;
			ensure!(owner == auction_key.0, Error::<T>::OwnerRequired);

			// Generate a hash if the hash_id parameter is None
			let hash = match &hash_id {
				Some(hash) => hash.clone(),
				None => T::Hashing::hash_of(&auction_key),
			};

			// check if there is a previous bid
			if let Some((_, price)) = Bids::<T>::get(&auction_key, &hash) {
				ensure!(price <= new_price, Error::<T>::AuctionAssigned);
			}

			// Update auction
			auction.start_price = new_price;
			NFTAuctions::<T>::insert(&auction_key, auction);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn bid(
			origin: OriginFor<T>,
			auction_key: Key<T>,
			price: BalanceOf<T>,
			hash_id: Option<HashId<T>>,
		) -> DispatchResult {
			let bidder = ensure_signed(origin.clone())?;

			ensure!(
				<T as pallet::Config>::Currency::free_balance(&bidder) >= price,
				Error::<T>::InsufficientBalance
			);

			let mut auction =
				NFTAuctions::<T>::get(&auction_key).ok_or(Error::<T>::AuctionKeyNotFound)?;

			ensure!(bidder != auction_key.0, Error::<T>::OriginProhibited);

			// The bid price must be higher than the current highest bid.
			ensure!(price > auction.highest_bid, Error::<T>::PriceTooLow);

			// Generate a hash if the hash_id parameter is None
			let hash = match &hash_id {
				Some(hash) => hash.clone(),
				None => T::Hashing::hash_of(&bidder),
			};

			let prev_key = match Bids::<T>::get(&auction_key, &hash) {
				Some((prev_key, prev_price)) => {
					ensure!(price > prev_price, Error::<T>::AuctionAssigned);
					<T as pallet::Config>::Currency::unreserve(&prev_key.0, prev_price);
					prev_key
				},
				None => {
					let bid_key = (bidder.clone(), 1u8.into());
					Bids::<T>::insert(&auction_key, &hash, (bid_key.clone(), price));
					bid_key
				},
			};

			if auction.highest_bid > Zero::zero() {
				<T as pallet::Config>::Currency::unreserve(
					&auction.highest_bidder,
					auction.highest_bid,
				);
			}

			<T as pallet::Config>::Currency::reserve(&bidder, price)?;

			auction.highest_bid = price;
			auction.highest_bidder = bidder.clone();

			Bids::<T>::insert(&auction_key, &hash, (prev_key.clone(), price));
			NFTAuctions::<T>::insert(&auction_key, auction);

			Self::deposit_event(Event::<T>::Bid { auction_key, hash, bid_key: prev_key, price });
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn confirm(
			origin: OriginFor<T>,
			auction_key: Key<T>,
			hash: HashId<T>,
		) -> DispatchResult {
			let owner = ensure_signed(origin)?;

			let auction =
				NFTAuctions::<T>::get(&auction_key).ok_or(Error::<T>::AuctionKeyNotFound)?;

			ensure!(owner == auction_key.0, Error::<T>::OwnerRequired);

			let ((bidder, _), price) =
				Bids::<T>::get(&auction_key, &hash).ok_or(Error::<T>::AuctionNotAssigned)?;

			ensure!(price >= auction.start_price, Error::<T>::AuctionNotAssigned);

			let price_calc: u64 = price.saturated_into::<u64>();
			let start_price: u64 = auction.start_price.saturated_into::<u64>();

			T::NFTsPallet::sell_nft(
				&owner,
				&bidder,
				&auction.collection_id,
				&auction.nft_id,
				price_calc.clone(),
				start_price.clone(),
				auction.total_supply,
			)?;

			// Delete auction from storage
			Bids::<T>::remove_prefix(&auction_key, None);
			NFTAuctions::<T>::remove(&auction_key);

			Self::deposit_event(Event::<T>::Confirmed { auction_key });

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn retract(
			origin: OriginFor<T>,
			auction_key: Key<T>,
			hash_id: HashId<T>,
		) -> DispatchResult {
			let bidder = ensure_signed(origin)?;
			// fetch auction and previous bid
			let mut auction =
				NFTAuctions::<T>::get(&auction_key).ok_or(Error::<T>::AuctionKeyNotFound)?;

			let (top_key, top_price) =
				Bids::<T>::get(&auction_key, hash_id).ok_or(Error::<T>::TopBidRequired)?;

			// only the top bid can be retracted
			ensure!(bidder == top_key.0, Error::<T>::TopBidRequired);

			// bidder unreserves the bid amount
			<T as pallet::Config>::Currency::unreserve(&bidder, top_price);

			// Remove the top bid
			Bids::<T>::remove(&auction_key, &hash_id);

			// Retrieve the previous highest bid
			let bid_history = Bids::<T>::iter_prefix_values(&auction_key).collect::<Vec<_>>();
			let prev_bid = bid_history.iter().max_by(|x, y| x.1.cmp(&y.1));

			// if there is a previous bid, set it to be the new highest bid
			match prev_bid {
				Some((prev_bidder, prev_price)) => {
					auction.highest_bid = *prev_price;
					auction.highest_bidder = prev_bidder.0.clone();
					Bids::<T>::insert(&auction_key, hash_id, (prev_bidder.clone(), *prev_price));
				},
				None => {
					// if no previous bids exist, just remove the bid for this auction
					auction.highest_bid = auction.start_price;
					auction.highest_bidder = auction.issuer.clone(); // Reset highest_bidder
				},
			}
			NFTAuctions::<T>::insert(&auction_key, auction);

			Self::deposit_event(Event::<T>::Retracted {
				auction_key,
				bid_key: top_key,
				price: top_price,
			});
			Ok(())
		}
	}
}
