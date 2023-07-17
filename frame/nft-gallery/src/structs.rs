use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32, BoundedVec};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

pub mod MarketPlaceStructs {
	use super::*;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(AccountId, Balance, Hash))]
	pub struct Marketplace<AccountId, Balance, Hash> {
		pub metadata: BoundedVec<u8, ConstU32<32>>,
		pub owner: AccountId,
		pub issuer: AccountId,
		pub fee: Balance,
		pub hash_id: Hash,
		pub export_fee:Balance,
		pub import_fee:Balance,
	}
}
