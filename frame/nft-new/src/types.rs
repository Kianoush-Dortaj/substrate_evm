pub mod Types {

	pub use crate::{
		pallet::Config,
		structs::NFTStructs::{Collection, ShareProfitsInfo, NFT,ConfigMarketPlace},
	};
	use frame_support::traits::Currency;
	use frame_system::Config as SystemConfig;

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;

	pub type HashId<T> = <T as frame_system::Config>::Hash;

	pub type ConfigMarketPlaceDetailsOf = ConfigMarketPlace;

	pub type CollectionDetailsOf<T> = Collection<<T as SystemConfig>::AccountId, HashId<T>>;

	pub type SahreProfitDetailsOf<T> = ShareProfitsInfo<<T as SystemConfig>::AccountId>;

	pub type NFTDetailsOf<T> = NFT<<T as SystemConfig>::AccountId, BalanceOf<T>>;

	pub type AccountOf<T> = <T as SystemConfig>::AccountId;
}
