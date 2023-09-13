pub mod Utility {
	use crate::{
		types::Types::{BalanceOf, Config, SahreProfitDetailsOf},
		AccountOf,
	};
	use frame_support::{
		dispatch::DispatchError,
		inherent::Vec,
		sp_runtime::traits::SaturatedConversion,
		traits::{Currency, ExistenceRequirement},
	};
	use sp_runtime::traits::UniqueSaturatedFrom;
	/// This function calculates the amount to be transferred based on a given price, total supply,
	/// and percentage.
	///
	/// # Arguments
	///
	/// * `price` - The price at which the NFT is being sold.
	/// * `total_supply` - The total supply of the NFTs for sale.
	/// * `percentage` - The profit percentage for the transaction.
	///
	/// # Returns
	///
	/// * `BalanceOf<T>` - The result of the function, indicating the amount to be transferred.
	///
	/// # Errors
	///
	/// This function will panic with `Balance should be convertible to u64` if the price cannot be
	/// converted to a `u64`.
	///
	/// # Example
	///
	/// Consider an NFT sale where the price is 100, total supply is 10, and the profit percentage
	/// is 20%. The amount to transfer would be calculated as follows: `(100 * 10 * 20) / 100`,
	/// resulting in 200. Thus, this function will return `200` for these input parameters.
	///
	/// # Usage
	///
	/// This function is typically used in the context of transferring profits from NFT sales.
	/// The calculated amount is used in calls to the `Currency::transfer` function to distribute
	/// profits among stakeholders.
	pub fn calc_transfer_amount_with_percentage<T: Config>(
		price: BalanceOf<T>,
		total_supply: u64,
		percentage: u64,
	) -> BalanceOf<T> {
		let price_as_u64: u64 = TryInto::<u64>::try_into(price)
			.ok()
			.expect("Balance should be convertible to u64; qed");

		let amount_to_transfer_as_u64 = price_as_u64 * total_supply * percentage / 100;

		let amount_to_transfer: BalanceOf<T> =
			UniqueSaturatedFrom::unique_saturated_from(amount_to_transfer_as_u64);
		amount_to_transfer
	}
	/// This function performs the transfer of NFT share profit to multiple stakeholders.
	///
	/// # Arguments
	///
	/// * `buyer` - The account identifier of the user buying the NFT.
	/// * `share_profit_address` - A vector containing the details of profit shares, including the
	///   address of each stakeholder and their respective share percentage.
	/// * `price` - The price at which the NFT is being sold.
	/// * `total_supply` - The total supply of the NFTs for sale.
	///
	/// # Returns
	///
	/// * `DispatchResult` - The result of the function, indicating success or failure.
	///
	/// # Errors
	///
	/// This function can return `DispatchError` when `Currency::transfer` fails for
	/// any of the stakeholders in `share_profit_address`.
	///
	/// # Events
	///
	/// This function does not emit any events.
	///
	/// # Transactional
	///
	/// This function is not flagged as `transactional`. If it fails, only the changes made
	/// within the failing transaction will be rolled back.
	///
	/// # Example
	///
	/// If an NFT is being sold at a price of 100, and there are two stakeholders each with a
	/// 50% profit share, this function will transfer 50 units of currency to each stakeholder.
	///
	/// # Usage
	///
	/// This function is typically used after an NFT sale to distribute profits among stakeholders
	/// based on their share percentages.
	pub fn do_transfer_nft_share_profit<T: Config>(
		buyer: &AccountOf<T>,
		share_profit_address: &Vec<SahreProfitDetailsOf<T>>,
		price: &BalanceOf<T>,
		total_supply: u64,
	) -> Result<(), DispatchError> {
		for info in share_profit_address {
			let amount_to_transfer =
				calc_transfer_amount_with_percentage::<T>(*price, total_supply, info.percentage);

			// Use the transfer function
			T::Currency::transfer(
				&buyer,
				&info.owner_address,
				amount_to_transfer,
				ExistenceRequirement::AllowDeath,
			)?;
		}

		Ok(())
	}

	pub fn calc_royalty_and_fee<T: Config>(
		royalty: u64,
		price: &BalanceOf<T>,
		royalty_fee: u64,
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		// Calculate royalty amount
		let royalty_percentage: BalanceOf<T> = royalty.saturated_into::<BalanceOf<T>>();

		let royalty_amount = 
        (*price * royalty_percentage) / BalanceOf::<T>::from(100u32);

		// Calculate royalty fee
		let fee_percentage: BalanceOf<T> = royalty_fee.saturated_into::<BalanceOf<T>>();
		let fee_amount = 
        (royalty_amount * fee_percentage) / BalanceOf::<T>::from(100u32);

		Ok((royalty_amount, fee_amount))
	}
}