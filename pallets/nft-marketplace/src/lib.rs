#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::UnixTime, PalletId};
//use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::{RuntimeDebug, traits::{AtLeast32BitUnsigned, AccountIdConversion}};
use pallet_traits::{FTTransfer, NFTForMarketplace};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_system::ensure_signed;
	use frame_system::pallet_prelude::OriginFor;
	use super::*;

	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct SellItem<AccountId> {
		pub seller: AccountId,
		pub current_price: u64,
		pub current_winner: AccountId,
		pub start_time: u64,
		pub end_time: u64,
	}

	#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct RentItem<AccountId> {
		pub leaser: AccountId,
		pub renter: AccountId,
		pub collateral: u64,
		pub price: u64,
		pub start_time: u64,
		pub duration: u64,
		pub is_renting: bool,
	}

	pub type SellItemOf<T> = SellItem<<T as frame_system::Config>::AccountId>;
	pub type RentItemOf<T> = RentItem<<T as frame_system::Config>::AccountId>;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type MaxBidDuration: Get<u64>;

		#[pallet::constant]
		type MinBidDuration: Get<u64>;

		#[pallet::constant]
		type MinBasePrice: Get<u64>;

		#[pallet::constant]
		type MaxBasePrice: Get<u64>;

		#[pallet::constant]
		type MinRentPrice: Get<u64>;

		#[pallet::constant]
		type MaxRentPrice: Get<u64>;

		#[pallet::constant]
		type MinCollateral: Get<u64>;

		#[pallet::constant]
		type MaxCollateral: Get<u64>;

		#[pallet::constant]
		type MinRentDuration: Get<u64>;

		#[pallet::constant]
		type MaxRentDuration: Get<u64>;
		/// The class ID type
		type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		/// The token ID type
		type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type NFT: NFTForMarketplace<Self::AccountId, Self::ClassId, Self::TokenId>;

		type FT: FTTransfer<Self::AccountId>;

		type UnixTime: UnixTime;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn sell_item)]
	pub type SellItems<T: Config> = StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, SellItemOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn rent_item)]
	pub type RentItems<T: Config> = StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, RentItemOf<T>>;


	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		Opened(T::AccountId, T::ClassId, T::TokenId, u64),
		Bid(T::AccountId, T::ClassId, T::TokenId, u64),
		Canceled(T::AccountId, T::ClassId, T::TokenId),
		Closed(T::AccountId, T::ClassId, T::TokenId),

		Offered(T::AccountId, T::ClassId, T::TokenId),
		RentCanceled(T::AccountId, T::ClassId, T::TokenId),
		Rented(T::AccountId, T::ClassId, T::TokenId),
		Repaid(T::AccountId, T::ClassId, T::TokenId),
		Liquidated(T::AccountId, T::ClassId, T::TokenId, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Errors should have helpful documentation associated with them.
		InvalidBidTime,
		InvalidBasePrice,
		SellItemNotExist,
		OnlyOwnerCanSellNFT,
		OnlyOwnerCanCancel,
		CannotCancel,
		OwnerCannotBid,
		NotInBidDuration,
		TooLowBidPrice,
		OnlyWinnerCanClaim,
		SellIsNotEnded,

		InvalidRentPrice,
		InvalidRentDuration,
		InvalidRentCollateral,
		OnlyOwnerCanOffer,
		RentItemNotExist,
		ItemIsRenting,
		ItemIsNotRenting,
		RenterMustNotBeLeaser,
		OnlyRenterCanRepay,
		RentIsExpired,
		OnlyLeaserCanLiquidate,
		RentIsNotExpired,
		OnlyOwnerCanCancelRent,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(100_000)]
		pub fn open_bid(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
			base_price: u64,
			delay_duration: u64,
			bid_duration: u64,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			Self::do_open(
				&seller,
				class_id,
				token_id,
				base_price,
				delay_duration,
				bid_duration,
			)
		}

		#[pallet::weight(100_000)]
		pub fn cancel_bid(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			Self::do_cancel(&seller, class_id, token_id)
		}

		#[pallet::weight(100_000)]
		pub fn bid(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
			bid_price: u64,
		) -> DispatchResult {
			let bidder = ensure_signed(origin)?;
			Self::do_bid(
				&bidder,
				class_id,
				token_id,
				bid_price,
			)
		}

		#[pallet::weight(100_000)]
		pub fn claim_nft(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let winner = ensure_signed(origin)?;
			Self::do_claim(&winner, class_id, token_id)
		}

		#[pallet::weight(100_000)]
		pub fn offer_rent(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
			duration: u64,
			collateral: u64,
			price: u64,
		) -> DispatchResult {
			let leaser = ensure_signed(origin)?;
			Self::do_offer(&leaser, class_id, token_id, duration, collateral, price)
		}

		#[pallet::weight(100_000)]
		pub fn rent(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let renter = ensure_signed(origin)?;
			Self::do_rent(&renter, class_id, token_id)
		}

		#[pallet::weight(100_000)]
		pub fn cancel_rent(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let leaser = ensure_signed(origin)?;
			Self::do_cancel_rent(&leaser, class_id, token_id)
		}

		#[pallet::weight(100_000)]
		pub fn repay_rent(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let renter = ensure_signed(origin)?;
			Self::do_repay(&renter, class_id, token_id)
		}

		#[pallet::weight(100_000)]
		pub fn liquidate_rent(
			origin: OriginFor<T>,
			class_id: T::ClassId,
			token_id: T::TokenId,
		) -> DispatchResult {
			let leaser = ensure_signed(origin)?;
			Self::do_liquidate(&leaser, class_id, token_id)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn pallet_account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}
	pub fn do_open(
		seller: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
		base_price: u64,
		delay_duration: u64,
		bid_duration: u64,
	) -> DispatchResult {
		let min_base_price = T::MinBasePrice::get();
		let max_base_price = T::MaxBasePrice::get();
		let min_bid_duration = T::MinBidDuration::get();
		let max_bid_duration = T::MaxBidDuration::get();

		ensure!(
			(base_price >= min_base_price && base_price <= max_base_price),
			Error::<T>::InvalidBasePrice
		);
		let now = T::UnixTime::now().as_secs();
		let start_time = now + delay_duration;
		let end_time = start_time + bid_duration;
		ensure!(
			(start_time >= now && start_time < end_time && end_time - start_time >= min_bid_duration && end_time - start_time <= max_bid_duration),
			Error::<T>::InvalidBidTime
		);
		ensure!(T::NFT::is_owner_of(seller, class_id, token_id), Error::<T>::OnlyOwnerCanSellNFT);
		T::NFT::transfer(seller, &Self::pallet_account_id(), class_id, token_id)?;
		let sell_item: SellItemOf<T> = SellItem {
			seller: seller.clone(),
			current_price: base_price,
			current_winner: seller.clone(),
			start_time,
			end_time,
		};
		SellItems::<T>::insert(class_id, token_id, sell_item);
		Self::deposit_event(Event::<T>::Opened(seller.clone(), class_id, token_id, base_price));
		Ok(())
	}

	pub fn do_cancel(
		seller: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		if let Some(sell_item) = Self::sell_item(class_id, token_id) {
			ensure!(*seller == sell_item.seller, Error::<T>::OnlyOwnerCanCancel);
			ensure!(*seller == sell_item.current_winner, Error::<T>::CannotCancel);
			T::NFT::transfer(&Self::pallet_account_id(), seller, class_id, token_id)?;
			SellItems::<T>::remove(class_id, token_id);
		} else {
			return Err(Error::<T>::SellItemNotExist.into());
		}
		Self::deposit_event(Event::<T>::Canceled(seller.clone(), class_id, token_id));
		Ok(())
	}

	pub fn do_bid(
		bidder: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
		bid_price: u64,
	) -> DispatchResult {
		SellItems::<T>::try_mutate(class_id, token_id, |opt| -> DispatchResult{
			let mut sell_item = opt.as_mut().ok_or(Error::<T>::SellItemNotExist)?;
			ensure!(*bidder != sell_item.seller, Error::<T>::OwnerCannotBid);
			let now = T::UnixTime::now().as_secs();
			ensure!((sell_item.start_time <= now && sell_item.end_time >= now), Error::<T>::NotInBidDuration);
			ensure!(bid_price > sell_item.current_price, Error::<T>::TooLowBidPrice);
			T::FT::transfer(bidder, &sell_item.seller, bid_price - sell_item.current_price)?;
			if *bidder != sell_item.current_winner {
				T::FT::transfer(bidder, &sell_item.current_winner, sell_item.current_price)?;
			}
			sell_item.current_winner = bidder.clone();
			sell_item.current_price = bid_price;
			Ok(())
		})?;
		Self::deposit_event(Event::<T>::Bid(bidder.clone(), class_id, token_id, bid_price));
		Ok(())
	}

	pub fn do_claim(
		winner: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		if let Some(sell_item) = Self::sell_item(class_id, token_id) {
			ensure!(*winner == sell_item.current_winner, Error::<T>::OnlyWinnerCanClaim);
			let now = T::UnixTime::now().as_secs();
			ensure!(now > sell_item.end_time, Error::<T>::SellIsNotEnded);
			T::NFT::transfer(&Self::pallet_account_id(), winner, class_id, token_id)?;
			SellItems::<T>::remove(class_id, token_id);
		} else {
			return Err(Error::<T>::SellItemNotExist.into());
		}
		Self::deposit_event(Event::<T>::Closed(winner.clone(), class_id, token_id));
		Ok(())
	}

}

impl<T: Config> Pallet<T> {
	pub fn do_offer(
		leaser: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
		duration: u64,
		collateral: u64,
		price: u64,
	) -> DispatchResult {
		ensure!(
			(price >= T::MinRentPrice::get() && price <= T::MaxRentPrice::get()), Error::<T>::InvalidRentPrice
		);
		ensure!(
			(duration >= T::MinRentDuration::get() && duration <= T::MaxRentDuration::get()), Error::<T>::InvalidRentDuration
		);
		ensure!(
			(collateral >= T::MinCollateral::get() && collateral <= T::MaxCollateral::get()), Error::<T>::InvalidRentCollateral
		);
		ensure!(
			T::NFT::is_owner_of(leaser, class_id, token_id), Error::<T>::OnlyOwnerCanOffer
		);
		T::NFT::transfer(leaser, &Self::pallet_account_id(), class_id, token_id)?;
		let rent_item: RentItemOf<T> = RentItem {
			leaser: leaser.clone(),
			renter: leaser.clone(),
			collateral,
			price,
			start_time: 0,
			duration,
			is_renting: false,
		};
		RentItems::<T>::insert(class_id, token_id, rent_item);
		Self::deposit_event(Event::<T>::Offered(leaser.clone(), class_id, token_id));
		Ok(())
	}

	pub fn do_cancel_rent(
		leaser: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		if let Some(rent_item) = Self::rent_item(class_id, token_id) {
			ensure!(!rent_item.is_renting, Error::<T>::ItemIsRenting);
			ensure!(rent_item.leaser == *leaser, Error::<T>::OnlyOwnerCanCancelRent);
			T::NFT::transfer(&Self::pallet_account_id(), leaser, class_id, token_id)?;
			RentItems::<T>::remove(class_id, token_id);
			Self::deposit_event(Event::<T>::RentCanceled(leaser.clone(), class_id, token_id));
		} else {
			return Err(Error::<T>::RentItemNotExist.into());
		}
		Ok(())
	}

	pub fn do_rent(
		renter: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		RentItems::<T>::try_mutate(class_id, token_id, |opt| -> DispatchResult{
			let rent_item = opt.as_mut().ok_or(Error::<T>::RentItemNotExist)?;
			ensure!(!rent_item.is_renting, Error::<T>::ItemIsRenting);
			ensure!(rent_item.leaser != *renter, Error::<T>::RenterMustNotBeLeaser);
			T::FT::transfer(renter, &Self::pallet_account_id(), rent_item.collateral)?;
			T::FT::transfer(renter, &rent_item.leaser, rent_item.price)?;
			T::NFT::transfer(&Self::pallet_account_id(), renter, class_id, token_id)?;
			rent_item.renter = renter.clone();
			rent_item.start_time = T::UnixTime::now().as_secs();
			rent_item.is_renting = true;
			Ok(())
		})?;
		Ok(())
	}

	pub fn do_repay(
		renter: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		if let Some(rent_item) = Self::rent_item(class_id, token_id) {
			ensure!(rent_item.is_renting, Error::<T>::ItemIsNotRenting);
			ensure!(rent_item.renter == *renter, Error::<T>::OnlyRenterCanRepay);
			ensure!(rent_item.start_time + rent_item.duration >= T::UnixTime::now().as_secs(), Error::<T>::RentIsExpired);
			T::NFT::transfer(renter, &rent_item.leaser, class_id, token_id)?;
			T::FT::transfer(&Self::pallet_account_id(), renter, rent_item.collateral)?;
			RentItems::<T>::remove(class_id, token_id);
		} else {
			return Err(Error::<T>::RentItemNotExist.into());
		}
		Self::deposit_event(Event::<T>::Repaid(renter.clone(), class_id, token_id));
		Ok(())
	}

	pub fn do_liquidate(
		leaser: &T::AccountId,
		class_id: T::ClassId,
		token_id: T::TokenId,
	) -> DispatchResult {
		if let Some(rent_item) = Self::rent_item(class_id, token_id) {
			ensure!(rent_item.is_renting, Error::<T>::ItemIsNotRenting);
			ensure!(rent_item.leaser == *leaser, Error::<T>::OnlyLeaserCanLiquidate);
			ensure!(rent_item.start_time + rent_item.duration < T::UnixTime::now().as_secs(), Error::<T>::RentIsNotExpired);
			T::FT::transfer(&Self::pallet_account_id(), leaser, rent_item.collateral)?;
			RentItems::<T>::remove(class_id, token_id);
			Self::deposit_event(Event::<T>::Liquidated(leaser.clone(), class_id, token_id, rent_item.renter));
		} else {
			return Err(Error::<T>::RentItemNotExist.into());
		}

		Ok(())
	}
}
