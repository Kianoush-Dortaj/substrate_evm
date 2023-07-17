use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub mod NFTStructs {
	use super::*;

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
