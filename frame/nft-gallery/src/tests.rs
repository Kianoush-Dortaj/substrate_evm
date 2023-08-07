#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, Error};
    use frame_support::{assert_noop, assert_ok, traits::Currency};
    use sp_core::H256;
    use sp_runtime::AccountId32;
	use frame_system::Origin;
	use frame_support::pallet_prelude::ConstU32;
	use frame_support::traits::tokens::Balance;
	use frame_system::Pallet;
   use frame_support::BoundedVec;
   
    #[test]
    fn test_create_market_place_works() {
        new_test_ext().execute_with(|| {
            let origin = Origin::signed(AccountId32::from([0u8; 32]));
            let metadata: BoundedVec<u8, ConstU32<32>> = BoundedVec::from(vec![0u8; 32]);
            let fee: Balance = 100;
            let export_fee: Balance = 10;
            let import_fee: Balance = 10;
            let max_royalty: u64 = 5;
            let royalty_fee: u64 = 3;

            assert_ok!(Pallet::<Test>::create_market_place(
                origin,
                metadata,
                fee,
                export_fee,
                import_fee,
                max_royalty,
                royalty_fee
            ));

            let owner = AccountId32::from([0u8; 32]);
            let hash_id = <Test as frame_system::Config>::Hashing::hash_of(&metadata);
            let market = MarketplaceStorage::<Test>::get(owner, hash_id);

            assert_eq!(market.unwrap().fee, fee);
            assert_eq!(market.unwrap().export_fee, export_fee);
            assert_eq!(market.unwrap().import_fee, import_fee);
            assert_eq!(market.unwrap().max_royalty, max_royalty);
            assert_eq!(market.unwrap().royalty_fee, royalty_fee);
        });
    }

    #[test]
    fn test_create_market_place_fails_when_fee_is_too_high() {
        new_test_ext().execute_with(|| {
            let origin = Origin::signed(AccountId32::from([0u8; 32]));
            let metadata: BoundedVec<u8, ConstU32<32>> = BoundedVec::from(vec![0u8; 32]);
            let fee: Balance = u64::MAX;
            let export_fee: Balance = 10;
            let import_fee: Balance = 10;
            let max_royalty: u64 = 5;
            let royalty_fee: u64 = 3;

            assert_noop!(
                Pallet::<Test>::create_market_place(
                    origin,
                    metadata,
                    fee,
                    export_fee,
                    import_fee,
                    max_royalty,
                    royalty_fee
                ),
                Error::<Test>::InsufficientBalance
            );
        });
    }
}
