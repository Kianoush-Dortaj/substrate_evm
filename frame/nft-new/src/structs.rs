use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{inherent::Vec, traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub mod NFTStructs {
	use super::*;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId))]
	pub struct Owners<AccountId> {
		/// Token metadata
		pub total_supply: u64, // change this according to your needs
		/// Token owner
		pub address: AccountId,
	}

	/// Class info
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance))]
	pub struct ConfigMarketPlace {
		pub royalty_fee: u64,
		pub max_allow_royalty_percent: u64,
	}

	impl Default for ConfigMarketPlace {
		fn default() -> Self {
			Self { royalty_fee: 10, max_allow_royalty_percent: 10 }
		}
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId))]
	pub struct ShareProfitsInfo<AccountId> {
		/// Token metadata
		pub percentage: u64,
		/// Token owner
		pub owner_address: AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance))]
	pub struct NFT<AccountId, Balance> {
		/// Token metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		/// NFT Issuer
		pub issuer: AccountId,
		/// Token owner
		pub owners: Option<Vec<Owners<AccountId>>>,
		///  Share Profits
		pub share_profits: Vec<ShareProfitsInfo<AccountId>>,
		pub price: Balance,
		pub royalty: u64,
		pub end_date: u64,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(Hash))]
	pub struct Album<Hash> {
		/// Token metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		pub nfts: Vec<Hash>,
	}

	/// Class info
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, CollectionId))]
	pub struct Collection<AccountId, CollectionId> {
		pub collection_id: CollectionId,
		/// Class metadata
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		/// Class owner
		pub issuer: AccountId,
	}

	impl<AccountId, CollectionId> Default for Collection<AccountId, CollectionId>
	where
		AccountId: Default,
		CollectionId: Default,
	{
		fn default() -> Self {
			Self {
				collection_id: Default::default(),
				metadata: BoundedVec::default(),
				issuer: Default::default(),
			}
		}
	}
}
