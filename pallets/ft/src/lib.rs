#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_traits::{FTTransfer};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn balance)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Balances<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply)]
	pub type TotalSupply<T> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allowance)]
	pub type Allowances<T: Config> = StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, T::AccountId, u64, ValueQuery>;
	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Transferred(T::AccountId, T::AccountId, u64),
		Approved(T::AccountId, T::AccountId, u64),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.
		Overflow,

		InsufficientBalance,

		InsufficientAllowance
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub balances: Vec<(T::AccountId, u64)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				balances: vec![]
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.balances.iter().for_each(|balance| {
				Pallet::<T>::mint(&balance.0, balance.1).expect("Mint cannot fail during genesis");
			});
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000 + T::DbWeight::get().writes(2))]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
			Self::sub_balance(&from, amount)?;
			Self::add_balance(&to, amount)?;

			Self::deposit_event(Event::Transferred(from.clone(), to.clone(), amount));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn approve(origin: OriginFor<T>, spender: T::AccountId, amount: u64) -> DispatchResult{
			let owner = ensure_signed(origin)?;
			Allowances::<T>::insert(owner.clone(), spender.clone(), amount);
			Self::deposit_event(Event::Approved(owner.clone(), spender.clone(), amount));
			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(3))]
		pub fn transfer_from(origin: OriginFor<T>, from: T::AccountId, to: T::AccountId, amount: u64) -> DispatchResult{
			let signer = ensure_signed(origin)?;
			Self::sub_allowance(&from, &signer, amount)?;
			Self::sub_balance(&from, amount)?;
			Self::add_balance(&to, amount)?;
			Self::deposit_event(Event::Transferred(from.clone(), to.clone(), amount));
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn add_balance(
		who: &T::AccountId,
		amount: u64
	) -> DispatchResult {
		Balances::<T>::try_mutate(who, |balance| -> DispatchResult{
			*balance = balance.checked_add(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;
		Ok(())
	}

	pub fn sub_balance(
		who: &T::AccountId,
		amount: u64
	) -> DispatchResult {
		Balances::<T>::try_mutate(who, |balance| -> DispatchResult{
			*balance = balance.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
			Ok(())
		})?;
		Ok(())
	}

	pub fn add_supply(
		amount: u64
	) -> DispatchResult {
		TotalSupply::<T>::try_mutate(|total_supply| -> DispatchResult{
			*total_supply = total_supply.checked_add(amount).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;
		Ok(())
	}

	pub fn sub_supply(
		amount: u64
	) -> DispatchResult {
		TotalSupply::<T>::try_mutate(|total_supply| -> DispatchResult{
			*total_supply = total_supply.checked_sub(amount).ok_or(Error::<T>::InsufficientBalance)?;
			Ok(())
		})?;
		Ok(())
	}

	pub fn mint(
		who: &T::AccountId,
		amount: u64
	) -> DispatchResult {
		Self::add_supply(amount)?;
		Self::add_balance(who, amount)?;
		Ok(())
	}

	pub fn burn(
		who: &T::AccountId,
		amount: u64
	) -> DispatchResult {
		Self::sub_supply(amount)?;
		Self::sub_balance(who, amount)?;
		Ok(())
	}

	pub fn sub_allowance(
		owner: &T::AccountId,
		spender: &T::AccountId,
		amount: u64
	) -> DispatchResult {
		Allowances::<T>::try_mutate(owner, spender, |allowance| -> DispatchResult{
			*allowance = allowance.checked_sub(amount).ok_or(Error::<T>::InsufficientAllowance)?;
			Ok(())
		})?;
		Ok(())
	}
}

impl <T: Config> FTTransfer<T::AccountId> for Pallet<T>{
	fn transfer(src: &T::AccountId, dst: &T::AccountId, amount: u64) -> DispatchResult {
		Self::sub_balance(src, amount)?;
		Self::add_balance(dst, amount)?;
		Ok(())
	}
}
