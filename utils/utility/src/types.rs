pub mod Types {

	use crate::pallet::Config;
	use frame_support::traits::Currency;
	use frame_system::Config as SystemConfig;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	pub type AccountOf<T> = <T as SystemConfig>::AccountId;

	pub type HashId<T> = <T as frame_system::Config>::Hash;
}
