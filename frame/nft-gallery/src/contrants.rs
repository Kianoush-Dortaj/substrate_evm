pub mod currency {
	use node_primitives::Balance;

	pub const DOLLARS: Balance = 1_000_000_000_000;
	pub const CENTS: Balance = DOLLARS / 100;
	pub const MILLICENTS: Balance = CENTS / 1_000;

	pub const FILE_STORAGE_FIRST_YEAR_MINE: Balance = 50_000_000 * DOLLARS;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
	}
}