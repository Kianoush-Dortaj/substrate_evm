#![cfg_attr(not(feature = "std"), no_std)]

use codec::{alloc::vec, Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, Hash, Member, One},
	DispatchError, SaturatedConversion,
};

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
pub use structs::NFTStructs::{Collection, Owners, NFT};

pub mod types;
pub use types::Types::{
	AccountOf, BalanceOf, CollectionDetailsOf, HashId, NFTDetailsOf, SahreProfitDetailsOf,
};

pub mod utiles;
pub use utiles::Utility::{calc_royalty_and_fee, do_transfer_nft_share_profit};

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
		CreatedCollection {
			store_id: HashId<T>,
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
		},
		UpdateCollection {
			store_id: HashId<T>,
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
		},
		MintedNFT {
			store_id: HashId<T>,
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
			nft_id: HashId<T>,
		},
		UpdateNFT {
			store_id: HashId<T>,
			collection_id: HashId<T>,
			issuer: AccountOf<T>,
			nft_id: HashId<T>,
		},
		TransferredNFT {
			collection_id: HashId<T>,
			token_id: HashId<T>,
			store_id: HashId<T>,
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
		YouAreNotOwner,
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

	/// Store NFT info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn nfts)]
	pub(super) type NFTs<T: Config> = StorageNMap<
		_,
		(
			NMapKey<Twox64Concat, AccountOf<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
			NMapKey<Twox64Concat, HashId<T>>,
		),
		NFTDetailsOf<T>,
		OptionQuery,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Define the `create_collection` call for the pallet
		// This function allows a user to create a new collection of NFTs
		// It takes in the metadata of the collection, the address of the market owner, and the
		// unique identifier for the store
		#[pallet::call_index(0)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
			store_hash_id: HashId<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			Self::do_create_collection(issuer, metadata, store_hash_id, market_owner_address)
		}
		// Define the `update_collection` call for the pallet
		// This function allows a user to update an existing collection of NFTs
		// It takes in the updated metadata of the collection, the address of the market owner, and
		// the unique identifier for the store
		#[pallet::call_index(1)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn update_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			market_owner_address: AccountOf<T>,
			store_hash_id: HashId<T>,
			collection_hash_id: HashId<T>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&market_owner_address,
				&store_hash_id,
			)?;

			Self::do_update_collection(
				issuer,
				metadata,
				store_hash_id,
				collection_hash_id,
				market_owner_address,
			)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::PalletWeightInfo::do_something())]
		pub fn mint_nft(
			origin: OriginFor<T>,
			store_owner_address: AccountOf<T>,
			collection_id: HashId<T>,
			store_id: HashId<T>,
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
				store_id,
				metadata,
				royalty,
				share_profits,
				price,
				end_date,
			)
		}
	}

	impl<T: Config> Pallet<T> {
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
			owner: &T::AccountId,
			store_id: &HashId<T>,
			collection_id: &HashId<T>,
		) -> Result<CollectionDetailsOf<T>, Error<T>> {
			<Collections<T>>::get((owner.clone(), store_id.clone(), collection_id.clone()))
				.ok_or(Error::<T>::CollectionNotFound)
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
			store_hash_id: HashId<T>,
			market_owner_address: AccountOf<T>,
		) -> DispatchResult {
			// Store Transaction Fee
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&market_owner_address,
				&store_hash_id,
			)?;

			let collection_hash_id = T::Hashing::hash_of(&metadata);

			let collection_details = Collection {
				collection_id: collection_hash_id.clone(),
				metadata,
				issuer: issuer.clone(),
			};

			Collections::<T>::insert(
				(issuer.clone(), collection_hash_id.clone(), store_hash_id.clone()),
				collection_details.clone(),
			);

			Self::deposit_event(Event::CreatedCollection {
				store_id: store_hash_id,
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
			store_hash_id: HashId<T>,
			collection_hash_id: HashId<T>,
			market_owner_address: AccountOf<T>,
		) -> DispatchResult {
			// Store Transaction Fee
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&market_owner_address,
				&store_hash_id,
			)?;

			Collections::<T>::try_mutate(
				(issuer.clone(), collection_hash_id.clone(), store_hash_id.clone()),
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
				store_id: store_hash_id,
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
			store_id: HashId<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty: u64,
			share_profits: Vec<SahreProfitDetailsOf<T>>,
			price: BalanceOf<T>,
			end_date: u64,
		) -> DispatchResult {
			// Store Transaction Fee
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&store_owner_address,
				&store_id,
			)?;
			// Check that the collection exists
			let collection = Self::get_collection(&issuer, &store_id, &collection_id)?;

			ensure!(collection.issuer == issuer, Error::<T>::YouAreNotOwnerOfCollection);

			let sum: u64 = share_profits.iter().map(|info| info.percentage).sum();
			ensure!(sum == 100, Error::<T>::InvalidPercentageSum);

			T::NFTGallery::check_allow_royalty(&store_owner_address, &store_id, royalty.clone())?;

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
				(issuer.clone(), store_id.clone(), collection_id.clone(), nft_hash_id.clone()),
				nft_details,
			);

			// Emit the MintedNFT event
			Self::deposit_event(Event::MintedNFT {
				collection_id,
				issuer: issuer.clone(),
				store_id: store_id.clone(),
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
			T::NFTGallery::send_fee_to_market_place_owner(
				&issuer,
				&store_owner_address,
				&store_id,
			)?;

			// Check that the collection exists
			let collection = Self::get_collection(&issuer, &store_id, &collection_id)?;

			ensure!(collection.issuer == issuer, Error::<T>::YouAreNotOwnerOfCollection);

			let sum: u64 = share_profits.iter().map(|info| info.percentage).sum();
			ensure!(sum == 100, Error::<T>::InvalidPercentageSum);

			T::NFTGallery::check_allow_royalty(&store_owner_address, &store_id, royalty.clone())?;

			// Insert the NFT instance to the NFTs storage
			NFTs::<T>::try_mutate(
				(issuer.clone(), store_id.clone(), collection_id.clone(), nft_id.clone()),
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
				store_id: store_id.clone(),
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
			store_id: HashId<T>,
			nft_id: HashId<T>,
			total_supply: u64,
		) -> DispatchResult {
			let nft = NFTs::<T>::try_mutate(
				(
					nft_owner_address_id.clone(),
					store_id.clone(),
					collection_id.clone(),
					nft_id.clone(),
				),
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
				store_id,
				from: buyer.clone(),
				to: buyer,
				price: nft.price,
			});

			Ok(().into())
		}

		#[transactional]
		fn do_sell_nft(
			seller: AccountOf<T>,
			buyer: AccountOf<T>,
			collection_id: HashId<T>,
			nft_id: HashId<T>,
			price: BalanceOf<T>,
			store_id: HashId<T>,
			auction_start_price: BalanceOf<T>,
			total_supply: u64,
			nft_owner_address_id: AccountOf<T>,
		) -> DispatchResult {
			let store_info = T::NFTGallery::get_market_place_fee(&nft_owner_address_id, &store_id)?;

			// Retrieve the Album.
			NFTs::<T>::try_mutate_exists(
				(
					nft_owner_address_id.clone(),
					store_id.clone(),
					collection_id.clone(),
					nft_id.clone(),
				),
				|nft_option| -> Result<_, DispatchError> {
					let mut nft = nft_option.as_mut().ok_or(Error::<T>::NFTNotFound)?;

					// Unreserve deposits of bidder and owner
					<T as pallet::Config>::Currency::unreserve(&buyer, price);
					<T as pallet::Config>::Currency::unreserve(&seller, auction_start_price);

					// Calculate the royalty.
					let royalty_amount =
						calc_royalty_and_fee::<T>(nft.royalty, &price, store_info.0)?;

					// The remaining amount after subtracting the royalty.
					let remaining_amount = price.clone() - (royalty_amount.0 + royalty_amount.1);

					let store_info = T::NFTGallery::send_royalty_fee_to_market_place_owner(
						&buyer,
						&nft_owner_address_id,
						&store_id,
						&royalty_amount.0,
					)?;

					// Transfer the royalty to the creator.
					// T::Currency::transfer(
					// 	&buyer,
					// 	&nft.issuer,
					// 	royalty_amount.0,
					// 	ExistenceRequirement::KeepAlive,
					// )?;

					// T::Currency::transfer(
					// 	&buyer,
					// 	&nft.issuer,
					// 	royalty_amount.1,
					// 	ExistenceRequirement::KeepAlive,
					// )?;

					// // Transfer the remaining balance to the current owner (seller).
					// T::Currency::transfer(
					// 	&buyer,
					// 	&seller,
					// 	remaining_amount,
					// 	ExistenceRequirement::KeepAlive,
					// )?;

					// let mut index: usize = 0;
					// // Ensure the seller is an owner of this Album.
					// match &mut nft.owners {
					// 	Some(ref mut owners) => {
					// 		index = Self::find_index_owner(&seller, owners)?;
					// 		// If total supply reduces to zero, remove the owner.
					// 		// NOTE: total_supply is u64 type, so no as_mut() is needed.
					// 		owners[index].total_supply = owners[index]
					// 			.total_supply
					// 			.checked_sub(total_supply)
					// 			.ok_or(Error::<T>::ArithmeticUnderflow)?;

					// 		// NOTE: total_supply is u64 type, so compare with 0, not 0.0.
					// 		if owners[index].total_supply == 0 {
					// 			owners.remove(index);
					// 			Self::retain_nft_owners(&seller, &collection_id, &nft_id)?;
					// 		}

					// 		// Add the buyer to the owners.
					// 		owners.push(Owners {
					// 			address: buyer.clone(),
					// 			total_supply: total_supply.clone(),
					// 		});
					// 		Ok(())
					// 	},
					// 	None => Err(Error::<T>::OwnersEmpty),
					// }?;


					// Self::deposit_event(Event::NFTSold {
					// 	collection_id,
					// 	token_id: nft_id,
					// 	price: price.clone(),
					// 	seller: seller.clone(),
					// 	buyer: buyer.clone(),
					// 	royalty: nft.royalty,
					// });

					Ok(())
				},
			)
		}
	}
}
