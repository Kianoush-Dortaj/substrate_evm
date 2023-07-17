pub mod Types {
	use super::super::structs::NFTStructs; // Import `NFTStructs` from the structs module
	use crate::pallet::Config; // Import `Config` from the pallet module
	use frame_support::traits::Currency;
	use frame_system::Config as SystemConfig; // Import `SystemConfig`

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	pub type AccountOf<T> = <T as SystemConfig>::AccountId;

	pub type HashId<T> = <T as frame_system::Config>::Hash;

	pub type MarketPlace<T> = NFTStructs::Marketplace<AccountOf<T>, BalanceOf<T>, HashId<T>>;
}
