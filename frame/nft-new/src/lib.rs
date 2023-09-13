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
pub use structs::NFTStructs::{Collection, ConfigMarketPlace, Owners, NFT};

pub mod types;
pub use types::Types::{
	AccountOf, BalanceOf, CollectionDetailsOf, HashId, NFTDetailsOf, SahreProfitDetailsOf,
};

pub mod utiles;
pub use utiles::Utility::{calc_royalty_and_fee, do_transfer_nft_share_profit};

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
		type WeightInfo: WeightInfo;
		/// The currency mechanism, used for paying for reserves.
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An nft Collection was created.
		CreatedCollection {
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
		},
		UpdateCollection {
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
		},
		MintedNFT {
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
			nft_id: HashId<T>,
		},
		NFTSold {
			collection_id: HashId<T>,
			token_id: HashId<T>,
			price: BalanceOf<T>,
			seller: AccountOf<T>,
			buyer: AccountOf<T>,
			royalty: u64,
		},
		UpdateNFT {
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
			nft_id: HashId<T>,
		},
		TransferredNFT {
			collection_id: HashId<T>,
			token_id: HashId<T>,
			quantity: u64,
			from: AccountOf<T>,
			to: AccountOf<T>,
			price: BalanceOf<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		CollectionNotFound,
		YouAreNotOwnerOfCollection,
		NFTNotFound,
		InvalidPercentageSum,
		NFTHasOwner,
		OwnerNotFound,
		OwnersEmpty,
		ArithmeticUnderflow,
		YouAreNotOwner,
		OwnerNotHaveEnoughTotalSupply,
	}

	#[pallet::storage]
	pub(super) type NameOf<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>>;

	/// Store collection info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collections)]
	pub(super) type Collections<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		HashId<T>,
		Twox64Concat,
		AccountOf<T>,
		CollectionDetailsOf<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn config_market_place)]
	pub type ConfigInfo<T: Config> = StorageValue<_, ConfigMarketPlace, ValueQuery>;

	/// Store NFT info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn nfts)]
	pub(super) type NFTs<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Twox64Concat, HashId<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
			NMapKey<Twox64Concat, AccountOf<T>>,
		),
		NFTDetailsOf<T>,
		OptionQuery,
	>;


	pub trait NFTHelper {
		type AccountId;
		type CollectionId;
		type NFTId;
		type Balance;
		type HashId;

		fn has_permission_to_add_nft_in_auction(
			nft_owner: &Self::AccountId,
			bidder: &Self::AccountId,
			collection_id: &Self::CollectionId,
			nft_id: &Self::NFTId,
			total_supply: u64,
		) -> DispatchResult;

		fn sell_nft(
			nft_owner: &Self::AccountId,
			seller: &Self::AccountId,
			buyer: &Self::AccountId,
			collection_id: &Self::CollectionId,
			nft_id: &Self::NFTId,
			price_input: u64,
			auction_start_price_input: u64,
			total_supply_input: u64,
		) -> DispatchResult;
	}

	impl<T: Config> NFTHelper for Pallet<T> {
		type AccountId = AccountOf<T>;
		type CollectionId = HashId<T>;
		type NFTId = HashId<T>;
		type HashId = HashId<T>;
		type Balance = BalanceOf<T>;

		fn has_permission_to_add_nft_in_auction(
			nft_owner: &Self::AccountId,
			bidder: &Self::AccountId,
			collection_id: &Self::CollectionId,
			nft_id: &Self::NFTId,
			total_supply: u64,
		) -> DispatchResult {
			Self::has_permission_add_nft_in_auction(
				&nft_owner,
				&bidder,
				&collection_id,
				&nft_id,
				total_supply,
			)
		}

		fn sell_nft(
			nft_owner: &Self::AccountId,
			seller: &Self::AccountId,
			buyer: &Self::AccountId,
			collection_id: &Self::CollectionId,
			nft_id: &Self::NFTId,
			price_input: u64,
			auction_start_price_input: u64,
			total_supply: u64,
		) -> DispatchResult {
			let price: BalanceOf<T> = price_input.saturated_into::<BalanceOf<T>>();

			let auction_start_price: BalanceOf<T> =
				auction_start_price_input.saturated_into::<BalanceOf<T>>();

			Self::do_sell_nft(
				seller.clone(),
				buyer.clone(),
				collection_id.clone(),
				nft_id.clone(),
				price,
				auction_start_price,
				total_supply,
				nft_owner.clone(),
			)
		}
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Define the `create_collection` call for the pallet
		// This function allows a user to create a new collection of NFTs
		// It takes in the metadata of the collection, the address of the market owner, and the
		// unique identifier for the store
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
			store_hash_id: HashId<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_create_collection(issuer, metadata, market_owner_address)
		}
		// Define the `update_collection` call for the pallet
		// This function allows a user to update an existing collection of NFTs
		// It takes in the updated metadata of the collection, the address of the market owner, and
		// the unique identifier for the store
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn update_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
			store_hash_id: HashId<T>,
			collection_hash_id: HashId<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			// T::NFTGallery::send_fee_to_market_place_owner(
			// 	&issuer,
			// 	&market_owner_address,
			// 	&store_hash_id,
			// )?;

			Self::do_update_collection(issuer, metadata, collection_hash_id, market_owner_address)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn mint_nft(
			origin: OriginFor<T>,
			store_owner_address: AccountOf<T>,
			collection_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty: u64,
			share_profits: Vec<SahreProfitDetailsOf<T>>,
			price: BalanceOf<T>,
			end_date: u64,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_mint_nft(
				issuer,
				store_owner_address,
				collection_id,
				metadata,
				royalty,
				share_profits,
				price,
				end_date,
			)
		}
	}

	impl<T: Config> Pallet<T> {
		pub(crate) fn has_permission_add_nft_in_auction(
			nft_owner: &AccountOf<T>,
			bidder: &AccountOf<T>,
			collection_id: &HashId<T>,
			nft_id: &HashId<T>,
			total_supply: u64,
		) -> DispatchResult {
			Self::get_collection(&nft_owner, &collection_id)?;

			let find_nft = Self::get_nft(&nft_owner, &collection_id, &nft_id)?;

			if let Some(owners) = find_nft.owners {
				Self::check_owner_hash_enogh_total_suppply(&bidder, &owners, total_supply)?;
			}

			Ok(().into())
		}

		pub(crate) fn check_owner_hash_enogh_total_suppply(
			seller: &T::AccountId,
			owners: &Vec<Owners<T::AccountId>>,
			total_supply: u64,
		) -> Result<usize, Error<T>> {
			let user_id = owners
				.iter()
				.position(|x| x.address == *seller && x.total_supply >= total_supply)
				.ok_or(Error::<T>::OwnerNotHaveEnoughTotalSupply);

			user_id
		}

		/// This function retrieves the details of a specified collection from the storage.
		///
		/// # Arguments
		///
		/// * `owner` - The account identifier of the collection owner.
		/// * `store_id` - The unique identifier of the store that the collection belongs to.
		/// * `collection_id` - The unique identifier of the collection.
		///
		/// # Returns
		///
		/// * `Result<CollectionDetailsOf<T>, Error<T>>` - If the collection is found in the
		///   storage, the function will return a `CollectionDetailsOf<T>` struct, containing the
		///   details of the collection. If the collection cannot be found, the function will return
		///   an `Error<T>` with the `CollectionNotFound` error.
		///
		/// # Storage Access
		///
		/// This function accesses the `Collections<T>` storage item. It first constructs a tuple
		/// key with the owner's account id, store id and collection id, and uses this key to try to
		/// get the collection details from the storage. If no value is found, it will return an
		/// error indicating that the collection was not found.
		pub(crate) fn get_collection(
			collection_owner: &AccountOf<T>,
			collection_id: &HashId<T>,
		) -> Result<CollectionDetailsOf<T>, Error<T>> {
			<Collections<T>>::get(collection_id.clone(), collection_owner.clone())
				.ok_or(Error::<T>::CollectionNotFound)
		}

		pub(crate) fn get_nft(
			owner: &AccountOf<T>,
			collection_id: &HashId<T>,
			nft_id: &HashId<T>,
		) -> Result<NFTDetailsOf<T>, Error<T>> {
			<NFTs<T>>::get((nft_id, collection_id, owner)).ok_or(Error::<T>::NFTNotFound)
		}

		/// This is a private method that carries out the process of creating a new NFT collection.
		///
		/// # Arguments
		///
		/// * `issuer` - The account identifier of the user who is creating the NFT collection.
		/// * `metadata` - The metadata for the collection, limited in size by `ConstU32<32>`.
		/// * `store_hash_id` - The unique identifier of the store where the collection will be
		///   listed.
		///
		/// # Returns
		///
		/// * `DispatchResult` - The result of the function, indicating success or failure.
		///
		/// # Process
		///
		/// The function creates a unique hash for the collection based on its metadata, builds a
		/// `Collection` struct with the hash, metadata and issuer, and then inserts this struct
		/// into the `Collections` storage map.
		///
		/// # Events
		///
		/// This function emits a `CreatedCollection` event upon successfully creating a new
		/// collection.
		///
		/// # Transactional
		///
		/// This function is flagged as `transactional`. If it fails, all changes to storage will be
		/// rolled back.

		#[transactional]
		fn do_create_collection(
			issuer: T::AccountId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
		) -> DispatchResult {
			// Store Transaction Fee
			// T::NFTGallery::send_fee_to_market_place_owner(
			// 	&issuer,
			// 	&market_owner_address,
			// 	&store_hash_id,
			// )?;

			let collection_hash_id = T::Hashing::hash_of(&metadata);

			let collection_details = Collection {
				collection_id: collection_hash_id.clone(),
				metadata,
				issuer: issuer.clone(),
			};

			Collections::<T>::insert(
				collection_hash_id.clone(),
				issuer.clone(),
				collection_details.clone(),
			);

			Self::deposit_event(Event::CreatedCollection {
				collection_id: collection_hash_id,
				issuer: issuer.clone(),
			});
			Ok(().into())
		}
		/// This is a private method that carries out the process of updating an existing NFT
		/// collection's metadata.
		///
		/// # Arguments
		///
		/// * `issuer` - The account identifier of the user who issued the NFT collection.
		/// * `metadata` - The new metadata for the collection, limited in size by `ConstU32<32>`.
		/// * `store_hash_id` - The unique identifier of the store where the collection is listed.
		/// * `collection_hash_id` - The unique identifier of the collection that is to be updated.
		///
		/// # Returns
		///
		/// * `DispatchResult` - The result of the function, indicating success or failure.
		///
		/// # Errors
		///
		/// This function will return `CollectionNotFound` if the referenced collection does not
		/// exist.
		///
		/// # Events
		///
		/// This function emits an `UpdateCollection` event upon successfully updating the
		/// collection's metadata.
		///
		/// # Transactional
		///
		/// This function is flagged as `transactional`. If it fails, all changes to storage will be
		/// rolled back.
		#[transactional]
		fn do_update_collection(
			issuer: T::AccountId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			collection_hash_id: HashId<T>,
			market_owner_address: AccountOf<T>,
		) -> DispatchResult {
			// Store Transaction Fee
			// T::NFTGallery::send_fee_to_market_place_owner(
			// 	&issuer,
			// 	&market_owner_address,
			// 	&store_hash_id,
			// )?;

			Collections::<T>::try_mutate(
				collection_hash_id.clone(),
				issuer.clone(),
				|collection_info| -> Result<(), DispatchError> {
					match collection_info {
						Some(info) => {
							if info.issuer != issuer.clone() {
								return Err(Error::<T>::YouAreNotOwnerOfCollection.into())
							}
							info.metadata = metadata.clone();

							Ok(())
						},
						None => Err(Error::<T>::CollectionNotFound.into()),
					}
				},
			)?;

			Self::deposit_event(Event::UpdateCollection {
				collection_id: collection_hash_id,
				issuer: issuer.clone(),
			});
			Ok(().into())
		}
		/// This is a private method that carries out the process of minting a Non-Fungible Token
		/// (NFT).
		///
		/// # Arguments
		///
		/// * `issuer` - The account identifier of the user issuing the NFT.
		/// * `store_owner_address` - The account identifier of the store owner where the NFT will
		///   be listed.
		/// * `collection_id` - The unique identifier of the collection to which this NFT belongs.
		/// * `store_id` - The unique identifier of the store where the NFT will be listed.
		/// * `metadata` - The metadata associated with the NFT, limited in size by `ConstU32<32>`.
		/// * `royalty` - The royalty percentage for any future sales of the NFT.
		/// * `share_profits` - The details of how profits will be shared, likely among multiple
		///   stakeholders.
		/// * `price` - The price at which the NFT will be listed.
		/// * `end_date` - The end date for the listing of the NFT.
		///
		/// # Returns
		///
		/// * `DispatchResult` - The result of the function, indicating success or failure.
		///
		/// # Errors
		///
		/// This function can return `CollectionNotFound` if the referenced collection does not
		/// exist. It can also return `YouAreNotPwnerOfCollection` if the issuer is not the owner of
		/// the collection. Additionally, it will fail if `check_allow_royalty` fails for the given
		/// `store_owner_address` and `store_id`.
		///
		/// # Events
		///
		/// This function emits a `MintedNFT` event upon successfully minting an NFT.
		///
		/// # Transactional
		///
		/// This function is flagged as `transactional`. If it fails, all changes to storage will be
		/// rolled back.
		#[transactional]
		fn do_mint_nft(
			issuer: AccountOf<T>,
			store_owner_address: AccountOf<T>,
			collection_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty: u64,
			share_profits: Vec<SahreProfitDetailsOf<T>>,
			price: BalanceOf<T>,
			end_date: u64,
		) -> DispatchResult {
			// Store Transaction Fee
			// T::NFTGallery::send_fee_to_market_place_owner(
			// 	&issuer,
			// 	&store_owner_address,
			// 	&store_id,
			// )?;
			// Check that the collection exists
			let collection = Self::get_collection(&issuer, &collection_id)?;

			ensure!(collection.issuer == issuer, Error::<T>::YouAreNotOwnerOfCollection);

			let sum: u64 = share_profits.iter().map(|info| info.percentage).sum();
			ensure!(sum == 100, Error::<T>::InvalidPercentageSum);

			// T::NFTGallery::check_allow_royalty(&store_owner_address, &store_id,
			// royalty.clone())?;

			// Create the NFT instance
			let nft_details = NFT {
				metadata: metadata.clone(),
				issuer: issuer.clone(),
				royalty,
				owners: Some(vec![]),
				share_profits,
				price,
				end_date,
			};

			let nft_hash_id = T::Hashing::hash_of(&metadata);
			// Insert the NFT instance to the NFTs storage
			NFTs::<T>::insert(
				(nft_hash_id.clone(), collection_id.clone(), issuer.clone()),
				nft_details,
			);

			// Emit the MintedNFT event
			Self::deposit_event(Event::MintedNFT {
				collection_id,
				issuer: issuer.clone(),
				nft_id: nft_hash_id.clone(),
			});
			Ok(().into())
		}
		/// This is a private method that carries out the process of updating a Non-Fungible Token
		/// (NFT).
		///
		/// # Arguments
		///
		/// * `issuer` - The account identifier of the user issuing the NFT.
		/// * `store_owner_address` - The account identifier of the store owner where the NFT will
		///   be listed.
		/// * `collection_id` - The unique identifier of the collection to which this NFT belongs.
		/// * `store_id` - The unique identifier of the store where the NFT will be listed.
		/// * `nft_id` - The unique identifier of the NFT being updated.
		/// * `metadata` - The metadata associated with the NFT, limited in size by `ConstU32<32>`.
		/// * `royalty` - The royalty percentage for any future sales of the NFT.
		/// * `share_profits` - The details of how profits will be shared, likely among multiple
		///   stakeholders.
		/// * `price` - The price at which the NFT will be listed.
		/// * `end_date` - The end date for the listing of the NFT.
		///
		/// # Returns
		///
		/// * `DispatchResult` - The result of the function, indicating success or failure.
		///
		/// # Errors
		///
		/// This function can return `CollectionNotFound` if the referenced collection does not
		/// exist. It can also return `YouAreNotPwnerOfCollection` if the issuer is not the owner of
		/// the collection, `InvalidPercentageSum` if the sum of share profit percentages does not
		/// equal 100, `NFTHasOwner` if the NFT already has an owner, and `YouAreNotOwner` if the
		/// issuer is not the current owner of the NFT. Additionally, it will fail if
		/// `check_allow_royalty` fails for the given `store_owner_address` and `store_id`, and
		/// `NFTNotFound` if the given NFT doesn't exist.
		///
		/// # Events
		///
		/// This function emits an `UpdateNFT` event upon successfully updating an NFT.
		///
		/// # Transactional
		///
		/// This function is flagged as `transactional`. If it fails, all changes to storage will be
		/// rolled back.
		#[transactional]
		fn do_update_mint_nft(
			issuer: AccountOf<T>,
			store_owner_address: AccountOf<T>,
			collection_id: HashId<T>,
			store_id: HashId<T>,
			nft_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty: u64,
			share_profits: Vec<SahreProfitDetailsOf<T>>,
			price: BalanceOf<T>,
			end_date: u64,
		) -> DispatchResult {
			// Store Transaction Fee
			// T::NFTGallery::send_fee_to_market_place_owner(
			// 	&issuer,
			// 	&store_owner_address,
			// 	&store_id,
			// )?;

			// Check that the collection exists
			let collection = Self::get_collection(&issuer, &collection_id)?;

			ensure!(collection.issuer == issuer, Error::<T>::YouAreNotOwnerOfCollection);

			let sum: u64 = share_profits.iter().map(|info| info.percentage).sum();
			ensure!(sum == 100, Error::<T>::InvalidPercentageSum);

			// T::NFTGallery::check_allow_royalty(&store_owner_address, &store_id,
			// royalty.clone())?;

			// Insert the NFT instance to the NFTs storage
			NFTs::<T>::try_mutate(
				(nft_id.clone(), collection_id.clone(), issuer.clone()),
				|nft_details| -> Result<(), DispatchError> {
					match nft_details {
						Some(info) => {
							if let Some(owners) = &info.owners {
								if owners.len() > 0 {
									return Err(Error::<T>::NFTHasOwner.into())
								}
							}

							if info.issuer != issuer.clone() {
								return Err(Error::<T>::YouAreNotOwner.into())
							}

							info.metadata = metadata.clone();
							info.share_profits = share_profits.clone();
							info.price = price.clone();
							info.end_date = end_date.clone();

							Ok(())
						},
						None => Err(Error::<T>::NFTNotFound.into()),
					}
				},
			)?;

			// Emit the UpdateNFT event
			Self::deposit_event(Event::UpdateNFT {
				collection_id,
				issuer: issuer.clone(),
				nft_id: nft_id.clone(),
			});
			Ok(().into())
		}
		/// This function performs the operation of buying an NFT.
		///
		/// It is flagged as `transactional`, which means if any operation inside the function
		/// fails, all changes to storage made within the function will be rolled back.
		///
		/// # Arguments
		///
		/// * `buyer` - The account identifier of the user buying the NFT.
		/// * `nft_owner_address_id` - The account identifier of the current owner of the NFT.
		/// * `collection_id` - The unique identifier of the collection to which the NFT belongs.
		/// * `store_id` - The unique identifier of the store where the NFT is listed.
		/// * `nft_id` - The unique identifier of the NFT being bought.
		/// * `total_supply` - The total supply of the NFTs for sale.
		///
		/// # Returns
		///
		/// * `DispatchResult` - The result of the function, indicating success or failure.
		///
		/// # Errors
		///
		/// This function can return `NFTNotFound` if the NFT does not exist. It can also return
		/// `DispatchError` when either `Currency::reserve` or `do_transfer_nft_share_profit` fails.
		///
		/// # Events
		///
		/// This function does not emit any events, but an event could be added to indicate the
		/// successful transfer of the NFT.
		///
		/// # Usage
		///
		/// This function is used when a user wants to buy an NFT. It updates the owner of the NFT
		/// and handles the transfer of the purchase price, including the distribution of profits
		/// among stakeholders according to their share percentages.
		#[transactional]
		fn do_buy_nft(
			buyer: AccountOf<T>,
			nft_owner_address_id: AccountOf<T>,
			collection_id: HashId<T>,
			nft_id: HashId<T>,
			total_supply: u64,
		) -> DispatchResult {
			let nft = NFTs::<T>::try_mutate(
				(nft_id.clone(), collection_id.clone(), nft_owner_address_id.clone()),
				|nft_option| -> Result<_, DispatchError> {
					let mut nft = nft_option.as_mut().ok_or(Error::<T>::NFTNotFound)?;

					// Reserve the buyer's balance.
					T::Currency::reserve(&buyer, nft.price.clone())
						.map_err(|_| DispatchError::Other("Cannot reserve balance"))?;

					do_transfer_nft_share_profit::<T>(
						&buyer,
						&nft.share_profits,
						&nft.price,
						total_supply,
					)?;

					let ownres = Owners::<AccountOf<T>> {
						address: buyer.clone(),
						total_supply: total_supply.clone(),
					};

					if let Some(owners_vec) = &mut nft.owners {
						owners_vec.push(ownres);
					} else {
						nft.owners = Some(vec![ownres]);
					}

					Ok(nft.clone())
				},
			)?;

			// Emit event
			Self::deposit_event(Event::TransferredNFT {
				collection_id,
				token_id: nft_id,
				quantity: total_supply,
				from: buyer.clone(),
				to: buyer,
				price: nft.price,
			});

			Ok(().into())
		}
		/// This `do_sell_nft` function performs the sale of a Non-Fungible Token (NFT) on a
		/// marketplace. This function is tagged with `#[transactional]` which ensures that state
		/// changes are reverted in case of an error.
		///
		/// # Arguments
		///
		/// * `seller` - An AccountId representing the seller of the NFT.
		/// * `buyer` - An AccountId representing the buyer of the NFT.
		/// * `collection_id` - An identifier for the NFT collection to which the NFT belongs.
		/// * `nft_id` - An identifier for the NFT being sold.
		/// * `price` - The selling price of the NFT.
		/// * `store_id` - An identifier for the marketplace store.
		/// * `auction_start_price` - The starting price of the NFT for auction.
		/// * `total_supply` - The total supply of the NFT.
		/// * `nft_owner_address_id` - The AccountId of the NFT owner.
		///
		/// # Returns
		///
		/// * On success, it returns `Ok(())`.
		/// * On failure, it returns an Err wrapped in DispatchError.
		///
		/// # Errors
		///
		/// This function will return an error if:
		/// 1. There is a problem getting the market place fee.
		/// 2. The NFT does not exist.
		/// 3. There is an underflow when calculating the total supply.
		/// 4. The seller is not an owner of the NFT.
		/// 5. There is an error when transferring the currency.
		///
		/// # Events
		///
		/// This function will deposit an `NFTSold` event upon successful sale of the NFT.
		///
		/// # Panics
		///
		/// This function does not panic.
		#[transactional]
		fn do_sell_nft(
			seller: AccountOf<T>,
			buyer: AccountOf<T>,
			collection_id: HashId<T>,
			nft_id: HashId<T>,
			price: BalanceOf<T>,
			auction_start_price: BalanceOf<T>,
			total_supply: u64,
			nft_owner_address_id: AccountOf<T>,
		) -> DispatchResult {
			// let store_info = T::NFTGallery::get_market_place_fee(&buyer, &store_id)?;

			// Retrieve the Album.
			NFTs::<T>::try_mutate_exists(
				(nft_id.clone(), collection_id.clone(), nft_owner_address_id.clone()),
				|nft_option| -> Result<_, DispatchError> {
					let mut nft = nft_option.as_mut().ok_or(Error::<T>::NFTNotFound)?;

					// Unreserve deposits of bidder and owner
					<T as pallet::Config>::Currency::unreserve(&buyer, price);
					<T as pallet::Config>::Currency::unreserve(&seller, auction_start_price);

					let config = <ConfigInfo<T>>::get();

					// Calculate the royalty.
					let royalty_amount =
						calc_royalty_and_fee::<T>(nft.royalty, &price, config.royalty_fee)?;

					// The remaining amount after subtracting the royalty.
					let remaining_amount = price.clone() - (royalty_amount.0 + royalty_amount.1);

					// let store_info = T::NFTGallery::send_royalty_fee_to_market_place_owner(
					// 	&buyer,
					// 	&nft_owner_address_id,
					// 	&store_id,
					// 	&royalty_amount.0,
					// )?;

					T::Currency::transfer(
						&buyer,
						&nft.issuer,
						royalty_amount.1,
						ExistenceRequirement::KeepAlive,
					)?;

					// Transfer the remaining balance to the current owner (seller).
					T::Currency::transfer(
						&buyer,
						&seller,
						remaining_amount,
						ExistenceRequirement::KeepAlive,
					)?;

					let mut index: usize = 0;
					// Ensure the seller is an owner of this Album.
					match &mut nft.owners {
						Some(ref mut owners) => {
							index = Self::find_index_owner(&seller, owners)?;
							// If total supply reduces to zero, remove the owner.
							// NOTE: total_supply is u64 type, so no as_mut() is needed.
							owners[index].total_supply = owners[index]
								.total_supply
								.checked_sub(total_supply)
								.ok_or(Error::<T>::ArithmeticUnderflow)?;
							// Add the buyer to the owners.
							owners.push(Owners {
								address: buyer.clone(),
								total_supply: total_supply.clone(),
							});
							Ok(())
						},
						None => Err(Error::<T>::OwnersEmpty),
					}?;

					Self::deposit_event(Event::NFTSold {
						collection_id,
						token_id: nft_id,
						price: price.clone(),
						seller: seller.clone(),
						buyer: buyer.clone(),
						royalty: nft.royalty,
					});

					Ok(())
				},
			)
		}

		/// The `find_index_owner` function is a method for finding the index of a specific owner
		/// within a vector of owners.
		///
		/// # Arguments
		///
		/// * `seller` - A reference to an AccountId representing the owner we want to find.
		/// * `owners` - A reference to a vector of owners from which we are searching.
		///
		/// # Returns
		///
		/// * On success, it returns `Ok(usize)` where `usize` is the index of the `seller` in the
		///   `owners` vector.
		/// * On failure, when the seller is not found in the `owners` vector, it returns
		///   `Err(Error::<T>::OwnerNotFound)`.
		///
		/// # Errors
		///
		/// This function will return an `Error::<T>::OwnerNotFound` if the `seller` is not found in
		/// the `owners` vector.
		///
		/// # Example
		///
		/// ```
		/// let seller = AccountId::from([0; 32]);
		/// let owners = vec![Owners::new(AccountId::from([0; 32])), Owners::new(AccountId::from([1; 32]))];
		/// let result = find_index_owner(&seller, &owners);
		/// assert_eq!(result, Ok(0));
		/// ```
		///
		/// # Panics
		///
		/// This function does not panic.
		pub(crate) fn find_index_owner(
			seller: &T::AccountId,
			owners: &Vec<Owners<T::AccountId>>,
		) -> Result<usize, Error<T>> {
			let user_id = owners
				.iter()
				.position(|x| x.address == *seller)
				.ok_or(Error::<T>::OwnerNotFound);

			user_id
		}
	}
}
