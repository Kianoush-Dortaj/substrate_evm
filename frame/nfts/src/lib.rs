#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
use codec::{alloc::vec, Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, Member, One},
	DispatchError,
};


use frame_support::traits::UnixTime;

use frame_support::{
	inherent::Vec,
	pallet_prelude::{ValueQuery, *},
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
	Twox64Concat,
};
use frame_system::Config as SystemConfig;

pub use pallet::*;

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

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	pub type CollectionDetailsOf<T> =
		Collection<<T as SystemConfig>::AccountId, <T as Config>::CollectionId>;

	pub type NFTDetailsOf<T> =
		NFT<<T as SystemConfig>::AccountId, BalanceOf<T>>;

	pub type AlbumDetailsOf<T> =
		Album<<T as SystemConfig>::AccountId, BalanceOf<T>,<T as Config>::AlbumId>;

	// A value placed in storage that represents the current version of the Scheduler storage.
	// This value is used by the `on_runtime_upgrade` logic to determine whether we run
	// storage migration logic.
	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Releases {
		V0,
		V1,
		V2,
	}

	impl Default for Releases {
		fn default() -> Self {
			Releases::V0
		}
	}

	/// Class info
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, T))]
	pub struct Collection<AccountId, CollectionId> {
		pub collection_id: CollectionId,
		/// Class metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		/// Class owner
		pub issuer: AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId))]
	pub struct ShareProfitsInfo<AccountId> {
		/// Token metadata
		pub percentage: BoundedVec<u8, ConstU32<32>>,
		/// Token owner
		pub owner_address: AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId,Balance))]
	pub struct Owners<AccountId,Balance> {
		/// Token metadata
		pub total_supply: Balance, // change this according to your needs
		/// Token owner
		pub address: AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance))]
	pub struct NFT<AccountId, Balance> {
		/// Token metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		/// NFT Issuer
		pub issuer: AccountId,
		/// Token owner
		pub owners: Vec<Owners<AccountId,Balance>>,
		///  Share Profits
		pub share_profits: Vec<ShareProfitsInfo<AccountId>>,
		pub price: Balance,
		pub royalty: Balance,
		pub end_date:BoundedVec<u8, ConstU32<32>>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance))]
	pub struct AlbumTracks<AccountId, Balance,AlbumId> {
		pub track_id:Option< AlbumId>,
		/// Token metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		/// Token owner
		pub owners: Vec<Owners<AccountId,Balance>>,
		///  Share Profits
		pub share_profits: Vec<ShareProfitsInfo<AccountId>>,
		pub price: Balance,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance))]
	pub struct Album<AccountId,Balance,AlbumId> {
			/// Token metadata
			pub metadata: BoundedVec<u8, ConstU32<32>>,
			/// NFT Issuer
			pub issuer: AccountId,
			/// Token owner
			pub owners: Vec<Owners<AccountId,Balance>>,
			pub tracks:Vec<AlbumTracks<AccountId,Balance,AlbumId>>,
			pub royalty: Balance,
			pub end_date:BoundedVec<u8, ConstU32<32>>,

	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
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

			type AlbumId: Member
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
		
		type TimeProvider: UnixTime;


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
		CreatedCollection { collection_id: T::CollectionId, issuer: T::AccountId },
		/// A nft NFT was minted.
		
		MintedNFT {
			collection_id: T::CollectionId,
			nft_id: T::NFTId,
			owner: T::AccountId,
			caller: T::AccountId,
		},
		MintedAlbum {
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			owner: T::AccountId,
			caller: T::AccountId,
		},
		/// An nft NFT was burned.
		BurnedNFT {
			collection_id: T::CollectionId,
			token_id: T::NFTId,
			owner: T::AccountId,
		},
		BurnedAlbum {
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			owner: T::AccountId,
		},
		/// An nft NFT was transferred.
		TransferredNFT {
			collection_id: T::CollectionId,
			token_id: T::NFTId,
			quantity: T::Quantity,
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		},
		/// An nft NFT was transferred.
		TransferredAlbum {
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		},
		SoldNFT {
			collection_id: T::CollectionId,
			token_id: T::NFTId,
			quantity: T::Quantity,
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		},
		SoldAlbum {
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			quantity: T::Quantity,
			from: T::AccountId,
			to: T::AccountId,
			price: BalanceOf<T>,
		},
		NFTSold {
			collection_id: T::CollectionId,
			token_id: T::NFTId,
			price: BalanceOf<T>,
			seller: T::AccountId,
			buyer: T::AccountId,
			royalty: BalanceOf<T>,
		},
		AlbumSold {
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			price: BalanceOf<T>,
			seller: T::AccountId,
			buyer: T::AccountId,
			royalty: BalanceOf<T>,
		},
		/// NFT info was updated
		UpdatedNFT { collection_id: T::CollectionId, nft_id: T::NFTId },
		UpdatedAlbum { collection_id: T::CollectionId, nft_id: T::AlbumId },
		UpdatedShareProfitList { collection_id: T::CollectionId, nft_id: T::NFTId },
		UpdatedShareProfitListAlbumtracks { collection_id: T::CollectionId, album_id: T::AlbumId },

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Collection not found
		CollectionNotFound,
		/// NFT not found
		NFTNotFound,
		AlbumNotFound,
		NoAvailableAlbumId,
		/// The operator is not the owner of the NFT and has no permission
		NoPermission,
		/// No available Collection ID
		NoAvailableCollectionId,
		/// No available NFT ID
		NoAvailableNFTId,
		/// Royalty rate great than RoyaltyRateLimit
		RoyaltyRateTooHigh,
		/// Quantity is invalid
		InvalidQuantity,
		/// Num overflow
		NumOverflow,
		CanNotReserveCurrency,
		CanNotTransferCurrency,
		ExpiredBuyAlbum,
		/// At least one consumer is remaining so the NFT cannot be burend.
		ConsumerRemaining,
		NotNFTOwner,
		NotAlbumOwner,
	}

	/// Store collection info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn collections)]
	pub type Collections<T: Config> =
		StorageMap<_, Twox64Concat, T::CollectionId, CollectionDetailsOf<T>>;

	/// Artist Collection
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn artists_collections)]
	pub type ArtistCollections<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<CollectionDetailsOf<T>>>;

	/// User Collection
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn user_collections)]
	pub type UserBuyNFTs<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<(T::CollectionId,T::NFTId)>>;

		#[pallet::storage]
		#[pallet::unbounded]
		#[pallet::getter(fn nft_likes)]
		pub type NftLikes<T: Config> = StorageMap<
			_, 
			Twox64Concat, 
			T::AccountId,  // The user ID
			Vec<(T::CollectionId, T::NFTId)>,  // The NFT IDs
			OptionQuery,
		>;

		#[pallet::storage]
		#[pallet::unbounded]
		#[pallet::getter(fn album_likes)]
		pub type AlbumLikes<T: Config> = StorageMap<
			_, 
			Twox64Concat, 
			T::AccountId,  // The user ID
			Vec<(T::CollectionId, T::AlbumId)>,  // The NFT IDs
			OptionQuery,
		>;

	/// User Albums
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn user_albums)]
	pub type UserBuyAlbums<T: Config> =
	StorageMap<_, Twox64Concat, T::AccountId, Vec<(T::CollectionId,T::AlbumId)>>;

	/// Store nft info.
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn nfts)]
	pub type NFTs<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::CollectionId,
		Twox64Concat,
		T::NFTId,
		NFTDetailsOf<T>,
		OptionQuery,
	>;

		/// Store nft info.
		#[pallet::storage]
		#[pallet::unbounded]
		#[pallet::getter(fn albums)]
		pub type Albums<T: Config> = StorageDoubleMap<
			_,
			Twox64Concat,
			T::CollectionId,
			Twox64Concat,
			T::AlbumId,
			AlbumDetailsOf<T>,
			OptionQuery,
		>;

	/// Next available collection ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextCollectionId<T: Config> = StorageValue<_, T::CollectionId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_nft_id)]
	pub type NextNFTId<T: Config> =
		StorageMap<_, Twox64Concat, T::CollectionId, T::NFTId, ValueQuery>;

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_album_id)]
	pub type NextAlbumId<T: Config> =
		StorageMap<_, Twox64Concat, T::CollectionId, T::AlbumId, ValueQuery>;

	/// Storage version of the pallet.
	///
	/// New networks start with last version.
	#[pallet::storage]
	pub type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn create_collection(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, ConstU32<32>>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			let collection_id = NextCollectionId::<T>::try_mutate(
				|id| -> Result<T::CollectionId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableCollectionId)?;
					Ok(current_id)
				},
			)?;

			let collection_details = Collection {
				collection_id: collection_id.clone(),
				metadata,
				issuer: issuer.clone(),
			};

			Collections::<T>::insert(collection_id, collection_details.clone());

			ArtistCollections::<T>::try_mutate(
				issuer.clone(),
				|collections_option| -> Result<(), DispatchError> {
					match collections_option {
						Some(collections) => collections.push(collection_details.clone()),
						None => *collections_option = Some(vec![collection_details.clone()]),
					}

					Ok(())
				},
			)?;

			Self::deposit_event(Event::CreatedCollection { collection_id, issuer });
			Ok(().into())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn mint_nft(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty:BalanceOf<T>,
			share_profits: Vec<ShareProfitsInfo<T::AccountId>>,
			price: BalanceOf<T>,
			end_date:BoundedVec<u8, ConstU32<32>>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			// Check that the collection exists
			let collection =
				Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;

			// Check that the issuer is the owner of the collection
			ensure!(collection.issuer == issuer, Error::<T>::NoPermission);

			// Generate the next NFT id for the collection
			let nft_id = NextNFTId::<T>::try_mutate(
				collection_id,
				|id| -> Result<T::NFTId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableNFTId)?;
					Ok(current_id)
				},
			)?;

			// Create the NFT instance
			let nft_details = NFT {
				metadata: metadata.clone(),
				issuer: issuer.clone(),
				royalty,
				owners: vec![],
				share_profits,
				price,
				end_date,
			};

			// Insert the NFT instance to the NFTs storage
			NFTs::<T>::insert(collection_id, nft_id, nft_details);
			UserBuyNFTs::<T>::try_mutate(issuer.clone(),|collections_option|-> Result<(),DispatchError> {
				match collections_option {
					Some(collection) => collection.push((collection_id,nft_id)),
					None => *collections_option = Some(vec![(collection_id,nft_id)])
				}
				Ok(())
			})?;
			// Emit the MintedNFT event
			Self::deposit_event(Event::MintedNFT {
				collection_id,
				nft_id: nft_id,
				owner: issuer.clone(),
				caller: issuer,
			});

			Ok(().into())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn like_item_nft(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			nft_id: T::NFTId,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			// Make sure the item exists
			ensure!(NFTs::<T>::contains_key(collection_id, nft_id), Error::<T>::NFTNotFound);

			NftLikes::<T>::try_mutate(user.clone(), |likes_option| -> Result<(), DispatchError> {
				match likes_option {
					Some(likes) => {
						
						if let Some(position) = likes.iter().position(|&i| i == (collection_id, nft_id)) {
							likes.remove(position);
						} else { 
							likes.push((collection_id, nft_id));
						}
					},
					None => { 
						*likes_option = Some(vec![(collection_id, nft_id)]);
					},
				}
				Ok(())
			})?;

			Ok(().into())
		}

		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn like_item_album(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
		) -> DispatchResult {
			let user = ensure_signed(origin)?;

			// Make sure the item exists
			ensure!(Albums::<T>::contains_key(collection_id, album_id), Error::<T>::NFTNotFound);

			AlbumLikes::<T>::try_mutate(user.clone(), |likes_option| -> Result<(), DispatchError> {
				match likes_option {
					Some(likes) => {
						
						if let Some(position) = likes.iter().position(|&i| i == (collection_id, album_id)) {
							likes.remove(position);
						} else { 
							likes.push((collection_id, album_id));
						}
					},
					None => { 
						*likes_option = Some(vec![(collection_id, album_id)]);
					},
				}
				Ok(())
			})?;

			Ok(().into())
		}


		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn mint_album(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			metadata: BoundedVec<u8, ConstU32<32>>,
			royalty:BalanceOf<T>,
			mut tracks: Vec<AlbumTracks<T::AccountId,BalanceOf<T>,T::AlbumId>>,
			end_date:BoundedVec<u8, ConstU32<32>>,
		) -> DispatchResult {
			let issuer = ensure_signed(origin)?;

			// Check that the collection exists
			let collection =
				Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;

			// Check that the issuer is the owner of the collection
			ensure!(collection.issuer == issuer, Error::<T>::NoPermission);

			// Generate the next Alnume id for the collection
			let album_id = NextAlbumId::<T>::try_mutate(
				collection_id,
				|id| -> Result<T::AlbumId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAlbumId)?;
					Ok(current_id)
				},
			)?;

			for track in &mut tracks {
				let track_id = NextAlbumId::<T>::try_mutate(
					collection_id,
					|id| -> Result<T::AlbumId, DispatchError> {
						let current_id = *id;
						*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableAlbumId)?;
						Ok(current_id)
					},
				)?;
				track.track_id = Some(track_id);
			}
			

			// Create the Alnume instance
			let album_details = Album {
				metadata: metadata.clone(),
				issuer: issuer.clone(),
				royalty,
				tracks,
				owners: vec![],
				end_date,
			};

			// Insert the Alnume instance to the Alnume storage
			Albums::<T>::insert(collection_id, album_id, album_details);

			// Emit the MintedAlnume event
			Self::deposit_event(Event::MintedAlbum {
				collection_id,
				album_id,
				owner: issuer.clone(),
				caller: issuer,
			});

			Ok(().into())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn buy_album(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
			album_id: T::AlbumId,
			total_supply:BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let buyer = ensure_signed(origin)?;
			let mut price = BalanceOf::<T>::default();
			// Retrieve the Album.
			let album = Albums::<T>::try_mutate_exists(
				collection_id.clone(),
				album_id,
				|album_option| -> Result<_, DispatchError> {
					let mut album = album_option.as_mut().ok_or(Error::<T>::AlbumNotFound)?;


					// let end_date_str = String::from_utf8(album.end_date.clone())
					// .map_err(|_| DispatchError::Other("Cannot decode end_date to utf8"))?;
				
					// let end_date: DateTime<Utc> = serde_json::from_str(&end_date_str)
					// 	.map_err(|_| DispatchError::Other("Cannot deserialize end_date from utf8"))?;
					
					// let now = chrono::Utc::now();
					// ensure!(
					// 	now <= end_date,
					// 	Error::<T>::ExpiredBuyAlbum
					// );

			         for track in &album.tracks {
						// Reserve the buyer's balance.
	
						let total_price =
						 track.price.clone() * total_supply.clone();
						
						T::Currency::reserve(&buyer, total_price.clone())
						.map_err(|_| Error::<T>::CanNotReserveCurrency)?;

						// Transfer funds to the share profits addresses.

						for info in &track.share_profits {
						// Convert BoundedVec<u8, ConstU32<32>> to &[u8] and then to [u8; 4]
						let slice: &[u8] = info.percentage.as_slice();
						if let Ok(array) = slice.try_into() as Result<[u8; 4], _> {
							let percentage: u32 = u32::from_be_bytes(array);
							price += track.price;
							// Calculate amount

						let total_price =
						 track.price.clone() * total_supply.clone();

							let amount = 
							track.price.clone() * percentage.into() /
							<<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance::from(100u32);

						let transfer_amount = amount * total_price;

							// Transfer
							T::Currency::transfer(
								&buyer.clone(),
								&info.owner_address,
								transfer_amount,
								ExistenceRequirement::AllowDeath,
							)
							.map_err(|_| Error::<T>::CanNotTransferCurrency)?;
						} else {
							// Handle error if the conversion fails
							// e.g. return an error, log a message, etc.
						}
						}
					 }
					
					 let ownres = Owners::<T::AccountId,BalanceOf<T>>{
						address: buyer.clone(),
						total_supply: total_supply.clone(),
					};
					
					// Update the NFT.
					album.owners.push(ownres);
					
				

					Ok(album.clone())
				},
			)?;

			UserBuyAlbums::<T>::try_mutate(buyer.clone(),|collections_option|-> Result<(),DispatchError> {
				match collections_option {
					Some(collection) => collection.push((collection_id,album_id)),
					None => *collections_option = Some(vec![(collection_id,album_id)])
				}
				Ok(())
			})?;

			Self::deposit_event(Event::TransferredAlbum { 
				collection_id: collection_id.clone(),
				 album_id: album_id.clone(), 
				 from:album.issuer.clone(),
				 to: buyer.clone(),
				  price: price });

			Ok(().into())
		}
	
		// #[pallet::call_index(9)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn sell_album(
		// 	origin: OriginFor<T>,
		// 	buyer: T::AccountId,
		// 	collection_id: T::CollectionId,
		// 	album_id: T::AlbumId,
		// 	price: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	let seller = ensure_signed(origin)?;
		
		// 	Albums::<T>::try_mutate_exists(
		// 		collection_id,
		// 		album_id,
		// 		|album_option| -> Result<_, DispatchError> {
		// 			let mut album = album_option.as_mut().ok_or(Error::<T>::AlbumNotFound)?;
		
		// 			// Ensure the seller is an owner of this Album.
		// 			let index = album.owners.iter().position(|x| *x == seller).ok_or(Error::<T>::NotAlbumOwner)?;
		
		// 			// Calculate the royalty.
		// 			let royalty_percentage: BalanceOf<T> = album.royalty.into();
		// 			let royalty_amount = (price.clone() * royalty_percentage) / 
		// 			<<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance::from(100u32);
		
		// 			// The remaining amount after subtracting the royalty.
		// 			let remaining_amount = price.clone() - royalty_amount;
		
		// 			// Transfer the royalty to the creator.
		// 			T::Currency::transfer(&buyer, &album.issuer, royalty_amount, ExistenceRequirement::KeepAlive)?;
		
		// 			// Transfer the remaining balance to the current owner (seller).
		// 			T::Currency::transfer(&buyer, &seller, remaining_amount, ExistenceRequirement::KeepAlive)?;
		
		// 			// Remove the seller from the owners.
		// 			album.owners.remove(index);
		
		// 			// Add the buyer to the owners.
		// 			album.owners.push(buyer.clone());
		
		// 			UserBuyAlbums::<T>::try_mutate(buyer.clone(),|collections_option|-> Result<(),DispatchError> {
		// 				match collections_option {
		// 					Some(collection) => collection.push((collection_id,album_id)),
		// 					None => *collections_option = Some(vec![(collection_id,album_id)])
		// 				}
		// 				Ok(())
		// 			})?;

		// 			UserBuyAlbums::<T>::try_mutate(seller.clone(), |collections_option| -> Result<(),DispatchError> {
		// 				match collections_option {
		// 					Some(collection) => {
		// 						// Retain only the tuples that do not match (collection_id,nft_id)
		// 						collection.retain(|&x| x != (collection_id, album_id));
		// 					},
		// 					None => return Err(Error::<T>::NotNFTOwner.into()), // Or whatever error is appropriate if there's no collection
		// 				}
		// 				Ok(())
		// 			})?;
					
		// 			Self::deposit_event(Event::AlbumSold {
		// 				collection_id,
		// 				album_id: album_id,
		// 				price: price.clone(),
		// 				seller: seller.clone(),
		// 				buyer: buyer.clone(),
		// 				royalty: album.royalty,
		// 			});
		
		// 			Ok(())
		// 		}
		// 	)
		// }

		// #[pallet::call_index(3)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn buy_nft(
		// 	origin: OriginFor<T>,
		// 	collection_id: T::CollectionId,
		// 	nft_id: T::NFTId,
		// ) -> DispatchResultWithPostInfo {
		// 	let buyer = ensure_signed(origin)?;

		// 	// Retrieve the NFT.
		// 	let nft = NFTs::<T>::try_mutate_exists(
		// 		collection_id,
		// 		nft_id,
		// 		|nft_option| -> Result<_, DispatchError> {
		// 			let mut nft = nft_option.as_mut().ok_or(Error::<T>::NFTNotFound)?;

		// 			// Check if there are any tokens left to buy.
		// 			ensure!(
		// 				nft.total_issuance  > nft.owners.len().try_into().unwrap(),  // hypothetical conversion function
		// 				Error::<T>::InvalidQuantity
		// 			);

		// 			let now = T::TimeProvider::now().as_secs();

		// 			// Compare with user provided time.
		// 			// ensure!(
		// 			// 	now >= nft.end_date,
		// 			// 	"Current time is less than the provided time"
		// 			// );

		// 			// Reserve the buyer's balance.
		// 			T::Currency::reserve(&buyer, nft.price.clone())
		// 				.map_err(|_| DispatchError::Other("Cannot reserve balance"))?;

		// 			// Transfer funds to the share profits addresses.
		// 			for info in &nft.share_profits {
		// 				// Convert BoundedVec<u8, ConstU32<32>> to &[u8] and then to [u8; 4]
		// 				let slice: &[u8] = info.percentage.as_slice();
		// 				if let Ok(array) = slice.try_into() as Result<[u8; 4], _> {
		// 					let percentage: u32 = u32::from_be_bytes(array);
					
		// 					// Calculate amount
		// 					let amount = 
		// 					nft.price.clone() * percentage.into() /
		// 					 <<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance::from(100u32);

		// 					// Transfer
		// 					T::Currency::transfer(
		// 						&buyer,
		// 						&info.owners,
		// 						amount,
		// 						ExistenceRequirement::AllowDeath,
		// 					)
		// 					.map_err(|_| DispatchError::Other("Cannot transfer funds"))?;
		// 				} else {
		// 					// Handle error if the conversion fails
		// 					// e.g. return an error, log a message, etc.
		// 				}
		// 			}

		// 			// Update the NFT.
		// 			nft.owners.push(buyer.clone());

				

		// 			Ok(nft.clone())
		// 		},
		// 	)?;

		// 	// Insert the NFT instance to the NFTs storage
		// 	UserBuyNFTs::<T>::try_mutate(buyer.clone(),|collections_option|-> Result<(),DispatchError> {
		// 		match collections_option {
		// 			Some(collection) => collection.push((collection_id,nft_id)),
		// 			None => *collections_option = Some(vec![(collection_id,nft_id)])
		// 		}
		// 		Ok(())
		// 	})?;
		// 	// Emit event
		// 	Self::deposit_event(Event::TransferredNFT {
		// 		collection_id,
		// 		token_id: nft_id,
		// 		quantity: One::one(),
		// 		from: buyer.clone(),
		// 		to: buyer,
		// 		price: nft.price,
		// 	});

		// 	Ok(().into())
		// }
	
		// #[pallet::call_index(4)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn burn_nft(
		// 	origin: OriginFor<T>,
		// 	collection_id: T::CollectionId,
		// 	nft_id: T::NFTId,
		// ) -> DispatchResult {
		// 	let burner = ensure_signed(origin)?;
		
		// 	// Check that the NFT exists
		// 	let nft =
		// 		NFTs::<T>::get(collection_id, nft_id).ok_or(Error::<T>::NFTNotFound)?;
		
		// 	// Check that the burner is one of the owners of the NFT
		// 	ensure!(nft.owners.len() == 0, Error::<T>::NoPermission);
		
		// 	// Remove NFT details
		// 	NFTs::<T>::remove(collection_id, nft_id);
		
		// 	Self::deposit_event(Event::BurnedNFT {
		// 		collection_id,
		// 		token_id: nft_id,
		// 		owner: burner,
		// 	});
		
		// 	Ok(().into())
		// }

		// #[pallet::call_index(11)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn burn_album(
		// 	origin: OriginFor<T>,
		// 	collection_id: T::CollectionId,
		// 	album_id: T::AlbumId,
		// ) -> DispatchResult {
		// 	let burner = ensure_signed(origin)?;
		
		// 	// Check that the Album exists
		// 	let album =
		// 		Albums::<T>::get(collection_id, album_id).ok_or(Error::<T>::NFTNotFound)?;
		
		// 	// Check that the burner is one of the owners of the NFT
		// 	ensure!(album.owners.len() == 0, Error::<T>::NoPermission);
		
		// 	// Remove Album details
		// 	Albums::<T>::remove(collection_id, album_id);
		
		// 	Self::deposit_event(Event::BurnedAlbum {
		// 		collection_id,
		// 		album_id: album_id,
		// 		owner: burner,
		// 	});
		
		// 	Ok(().into())
		// }
	
		// #[pallet::call_index(5)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn update_share_profit(
		// 	origin: OriginFor<T>,
		// 	collection_id: T::CollectionId,
		// 	nft_id: T::NFTId,
		// 	updated_share_profits: Vec<ShareProfitsInfo<T::AccountId>>,
		// ) -> DispatchResult {
		// 	let issuer = ensure_signed(origin)?;

		// 	// Check that the collection exists
		// 	let _ = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;

		// 	// Check that the NFT exists
		// 	let mut nft = NFTs::<T>::get(collection_id, nft_id).ok_or(Error::<T>::NFTNotFound)?;

		// 	// Check that the issuer is the owner of the NFT
		// 	ensure!(nft.issuer == issuer, Error::<T>::NoPermission);

		// 	// Update the share profit details
		// 	nft.share_profits = updated_share_profits;

		// 	// Store the updated NFT details
		// 	NFTs::<T>::insert(collection_id, nft_id, nft);

		// 	// Emit the UpdatedNFT event
		// 	Self::deposit_event(Event::UpdatedShareProfitList {
		// 		collection_id,
		// 		nft_id,
		// 	});

		// 	Ok(().into())
		// }

		// #[pallet::call_index(10)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn update_share_profit_album(
		// 	origin: OriginFor<T>,
		// 	collection_id: T::CollectionId,
		// 	album_id: T::AlbumId,
		// 	track_id:T::AlbumId,
		// 	updated_share_profits: Vec<ShareProfitsInfo<T::AccountId>>,
		// ) -> DispatchResult {
		// 	let issuer = ensure_signed(origin)?;
		
		// 	// Check that the collection exists
		// 	let _ = Collections::<T>::get(collection_id).ok_or(Error::<T>::CollectionNotFound)?;
		
		// 	// Check that the Album exists and get a mutable reference
		// 	let mut album = Albums::<T>::get(collection_id, album_id).ok_or(Error::<T>::AlbumNotFound)?;
		
		// 	// Check that the issuer is the owner of the NFT
		// 	ensure!(album.issuer == issuer, Error::<T>::NoPermission);
		
		// 	// Update the share profits of the track with the specified ID
		// 	album.tracks.iter_mut()
		// 		.filter(|track| track.track_id == Some(track_id))
		// 		.for_each(|track| track.share_profits = updated_share_profits.clone());
		
		// 	// Store the updated album
		// 	Albums::<T>::insert(collection_id, album_id, album);
		
		// 	// Emit the UpdatedNFT event
		// 	Self::deposit_event(Event::UpdatedShareProfitListAlbumtracks {
		// 		collection_id,
		// 		album_id,
		// 	});
		
		// 	Ok(().into())
		// }
		
		// #[pallet::call_index(6)]
		// #[pallet::weight(T::WeightInfo::do_something())]
		// pub fn sell_nft(
		// 	origin: OriginFor<T>,
		// 	buyer: T::AccountId,
		// 	collection_id: T::CollectionId,
		// 	nft_id: T::NFTId,
		// 	price: BalanceOf<T>,
		// ) -> DispatchResult {
		// 	let seller = ensure_signed(origin)?;
		
		// 	NFTs::<T>::try_mutate_exists(
		// 		collection_id,
		// 		nft_id,
		// 		|nft_option| -> Result<_, DispatchError> {
		// 			let mut nft = nft_option.as_mut().ok_or(Error::<T>::NFTNotFound)?;
		
		// 			// Ensure the seller is an owner of this NFT.
		// 			let index = nft.owners.iter().position(|x| *x == seller).ok_or(Error::<T>::NotNFTOwner)?;
		
		// 			// Calculate the royalty.
		// 			let royalty_percentage: BalanceOf<T> = nft.royalty.into();
		// 			let royalty_amount = (price.clone() * royalty_percentage) / 
		// 			<<T as pallet::Config>::Currency as frame_support::traits::Currency<<T as frame_system::Config>::AccountId>>::Balance::from(100u32);
		
		// 			// The remaining amount after subtracting the royalty.
		// 			let remaining_amount = price.clone() - royalty_amount;
		
		// 			// Transfer the royalty to the creator.
		// 			T::Currency::transfer(&buyer, &nft.issuer, royalty_amount, ExistenceRequirement::KeepAlive)?;
		
		// 			// Transfer the remaining balance to the current owner (seller).
		// 			T::Currency::transfer(&buyer, &seller, remaining_amount, ExistenceRequirement::KeepAlive)?;
		
		// 			// Remove the seller from the owners.
		// 			nft.owners.remove(index);
		
		// 			// Add the buyer to the owners.
		// 			nft.owners.push(buyer.clone());
		
		// 			UserBuyNFTs::<T>::try_mutate(buyer.clone(),|collections_option|-> Result<(),DispatchError> {
		// 				match collections_option {
		// 					Some(collection) => collection.push((collection_id,nft_id)),
		// 					None => *collections_option = Some(vec![(collection_id,nft_id)])
		// 				}
		// 				Ok(())
		// 			})?;

		// 			UserBuyNFTs::<T>::try_mutate(seller.clone(), |collections_option| -> Result<(),DispatchError> {
		// 				match collections_option {
		// 					Some(collection) => {
		// 						// Retain only the tuples that do not match (collection_id,nft_id)
		// 						collection.retain(|&x| x != (collection_id, nft_id));
		// 					},
		// 					None => return Err(Error::<T>::NotNFTOwner.into()), // Or whatever error is appropriate if there's no collection
		// 				}
		// 				Ok(())
		// 			})?;
		// 			Self::deposit_event(Event::NFTSold {
		// 				collection_id,
		// 				token_id: nft_id,
		// 				price: price.clone(),
		// 				seller: seller.clone(),
		// 				buyer: buyer.clone(),
		// 				royalty: nft.royalty,
		// 			});
		
		// 			Ok(())
		// 		}
		// 	)
		// }
		
	}
}
