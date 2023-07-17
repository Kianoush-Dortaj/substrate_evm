pub mod Types {
	use crate::NFTStructs::{Collection}; // use structs // Import `NFTStructs` from the structs module
	use crate::pallet::Config; // Import `Config` from the pallet module
	use frame_support::traits::Currency;
	use frame_system::Config as SystemConfig; // Import `SystemConfig`

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	pub type HashId<T> = <T as frame_system::Config>::Hash;

	pub type CollectionDetailsOf<T> = Collection<<T as SystemConfig>::AccountId, HashId<T>>;

	pub type AccountOf<T> = <T as SystemConfig>::AccountId;
}
