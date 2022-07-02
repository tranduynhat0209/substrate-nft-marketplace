#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime::DispatchResult;

pub trait NFTForMarketplace<AccountId, ClassId, TokenId>{

	fn transfer(
		src: &AccountId,
		dst: &AccountId,
		class_id: ClassId,
		token_id: TokenId,
	) -> DispatchResult;


	fn is_owner_of(
		owner: &AccountId,
		class_id: ClassId,
		token_id: TokenId,
	) -> bool;
}

pub trait FTTransfer<AccountId>{
	fn transfer(
		src: &AccountId,
		dst: &AccountId,
		amount: u64
	) -> DispatchResult;
}
