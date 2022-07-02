#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{ensure, pallet_prelude::*, traits::Get, BoundedVec, Parameter};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero},
    ArithmeticError, DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;
use pallet_traits::NFTForMarketplace;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

/// Class info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct ClassInfo<TokenId, AccountId, Data, ClassMetadataOf> {
    /// Class metadata
    pub metadata: ClassMetadataOf,
    /// Total issuance for the class
    pub total_issuance: TokenId,
    /// Class owner
    pub owner: AccountId,
    /// Class Properties
    pub data: Data,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<AccountId, Data, TokenMetadataOf> {
    /// Token metadata
    pub metadata: TokenMetadataOf,
    /// Token owner
    pub owner: AccountId,
    /// Token Properties
    pub data: Data,
}

pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use frame_system::ensure_signed;
    use frame_system::pallet_prelude::OriginFor;
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The class ID type
        type ClassId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        /// The token ID type
        type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        /// The class properties type
        type ClassData: Parameter + Member + MaybeSerializeDeserialize;
        /// The token properties type
        type TokenData: Parameter + Member + MaybeSerializeDeserialize;
        /// The maximum size of a class's metadata
        #[pallet::constant]
        type MaxClassMetadata: Get<u32>;
        /// The maximum size of a token's metadata
        #[pallet::constant]
        type MaxTokenMetadata: Get<u32>;
    }

    pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;
    pub type TokenMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxTokenMetadata>;
    pub type ClassInfoOf<T> = ClassInfo<
        <T as Config>::TokenId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::ClassData,
        ClassMetadataOf<T>,
    >;
    pub type TokenInfoOf<T> =
    TokenInfo<<T as frame_system::Config>::AccountId, <T as Config>::TokenData, TokenMetadataOf<T>>;

    pub type GenesisTokenData<T> = (
        <T as frame_system::Config>::AccountId, // Token owner
        Vec<u8>,                                // Token metadata
        <T as Config>::TokenData,
    );
    pub type GenesisClassData<T> = (
        <T as frame_system::Config>::AccountId, // Token class owner
        Vec<u8>,                                // Token class metadata
        <T as Config>::ClassData,
        Vec<GenesisTokenData<T>>, // Vector of tokens belonging to this class
    );

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub classes: Vec<GenesisClassData<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig {
                classes: vec![]
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.classes.iter().for_each(|class| {
                let class_id = Pallet::<T>::do_create_class(&class.0, class.1.to_vec(), class.2.clone())
                    .expect("Create class cannot fail while building genesis");
                for (account_id, token_metadata, token_data) in &class.3 {
                    Pallet::<T>::mint(&account_id, class_id, token_metadata.to_vec(), token_data.clone())
                        .expect("Token mint cannot fail during genesis");
                }
            });
        }
    }

    /// Error for non-fungible-token module.
    #[pallet::error]
    pub enum Error<T> {
        /// No available class ID
        NoAvailableClassId,
        /// No available token ID
        NoAvailableTokenId,
        /// Token(ClassId, TokenId) not found
        TokenNotFound,
        /// Class not found
        ClassNotFound,
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Can not destroy class
        /// Total issuance is not 0
        CannotDestroyClass,
        /// Failed because the Maximum amount of metadata was exceeded
        MaxMetadataExceeded,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClassCreated(T::ClassId, T::AccountId),
        ClassDestroyed(T::ClassId, T::AccountId),
        TokenMinted(T::ClassId, T::TokenId, T::AccountId),
        TokenBurned(T::ClassId, T::TokenId, T::AccountId),
        TokenTransfer(T::ClassId, T::TokenId, T::AccountId, T::AccountId),
    }

    /// Next available class ID.
    #[pallet::storage]
    #[pallet::getter(fn next_class_id)]
    pub type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

    /// Next available token ID.
    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    pub type NextTokenId<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, T::TokenId, ValueQuery>;

    /// Store class info.
    ///
    /// Returns `None` if class info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn classes)]
    pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::ClassId, ClassInfoOf<T>>;

    /// Store token info.
    ///
    /// Returns `None` if token info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub type Tokens<T: Config> = StorageDoubleMap<_, Twox64Concat, T::ClassId, Twox64Concat, T::TokenId, TokenInfoOf<T>>;

    /// Token existence check by owner and class ID.
    #[pallet::storage]
    #[pallet::getter(fn token_by_owner)]
    pub type TokenByOwner<T: Config> = StorageNMap<_, (
        NMapKey<Twox64Concat, T::AccountId>,
        NMapKey<Twox64Concat, T::ClassId>,
        NMapKey<Twox64Concat, T::TokenId>
    ), (), ValueQuery>;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn create_class(
            origin: OriginFor<T>,
            metadata: Vec<u8>,
            data: T::ClassData
        ) -> DispatchResult{
            let creator = ensure_signed(origin)?;

            Self::do_create_class(&creator, metadata, data)?;
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn mint_token(
            origin: OriginFor<T>,
            owner: T::AccountId,
            class_id: T::ClassId,
            metadata: Vec<u8>,
            data: T::TokenData
        ) -> DispatchResult{
            let creator = ensure_signed(origin)?;
            if let Some(class) = Self::classes(class_id){
                ensure!(
                    creator == class.owner,
                    Error::<T>::NoPermission
                );
            }
            else{
                return Err(Error::<T>::ClassNotFound.into());
            }
            Self::mint(&owner, class_id, metadata, data)?;
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn transfer_token(
            origin: OriginFor<T>,
            dst: T::AccountId,
            class_id: T::ClassId,
            token_id: T::TokenId
        ) -> DispatchResult{
            let src = ensure_signed(origin)?;
            Self::transfer(&src, &dst, class_id, token_id)?;
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn burn_token(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            token_id: T::TokenId
        ) -> DispatchResult{
            let owner = ensure_signed(origin)?;
            Self::burn(&owner, class_id, token_id)?;
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn destroy_class(
            origin: OriginFor<T>,
            class_id: T::ClassId
        ) -> DispatchResult{
            let owner = ensure_signed(origin)?;
            Self::do_destroy_class(&owner, class_id)?;
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_create_class(
        creator: &T::AccountId,
        metadata: Vec<u8>,
        data: T::ClassData,
    ) -> Result<T::ClassId, DispatchError> {
        let bounded_metadata: ClassMetadataOf<T> = metadata.try_into().map_err(|_err| Error::<T>::MaxMetadataExceeded)?;
        let class_id = NextClassId::<T>::try_mutate(|next_id| -> Result<T::ClassId, DispatchError>{
            let id = *next_id;
            *next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
            Ok(id)
        })?;
        let new_class: ClassInfoOf<T> = ClassInfo {
            data: data,
            metadata: bounded_metadata,
            owner: creator.clone(),
            total_issuance: Default::default(),
        };
        Classes::<T>::insert(class_id, new_class);
        Self::deposit_event(Event::<T>::ClassCreated(class_id, creator.clone()));
        Ok(class_id)
    }

    pub fn mint(
        owner: &T::AccountId,
        class_id: T::ClassId,
        metadata: Vec<u8>,
        data: T::TokenData,
    ) -> Result<T::TokenId, DispatchError> {
        let bounded_metadata: TokenMetadataOf<T> = metadata.try_into().map_err(|_err| Error::<T>::MaxMetadataExceeded)?;
        let token_id = NextTokenId::<T>::try_mutate(class_id, |next_id| -> Result<T::TokenId, DispatchError>{
            let token_id = *next_id;
            *next_id = next_id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableTokenId)?;


            let new_token: TokenInfoOf<T> = TokenInfo {
                metadata: bounded_metadata,
                data: data,
                owner: owner.clone(),
            };
            Classes::<T>::try_mutate(class_id, |opt| -> DispatchResult{
                let class = opt.as_mut().ok_or(Error::<T>::ClassNotFound)?;
                class.total_issuance = class.total_issuance.checked_add(&One::one()).ok_or(ArithmeticError::Overflow)?;
                Ok(())
            })?;
            Tokens::<T>::insert(class_id, token_id, new_token);
            TokenByOwner::<T>::insert((owner.clone(), class_id, token_id), ());

            Ok(token_id)
        })?;
        Self::deposit_event(Event::<T>::TokenMinted(class_id, token_id, owner.clone()));
        Ok(token_id)
    }

    pub fn transfer(
        src: &T::AccountId,
        dst: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
    ) -> DispatchResult {
        Tokens::<T>::try_mutate(class_id, token_id, |opt| -> DispatchResult{
            let token = opt.as_mut().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(token.owner == *src, Error::<T>::NoPermission);
            token.owner = dst.clone();
            TokenByOwner::<T>::insert((dst.clone(), class_id, token_id), ());
            TokenByOwner::<T>::remove((src.clone(), class_id, token_id));
            Ok(())
        })?;
        Self::deposit_event(Event::<T>::TokenTransfer(class_id, token_id, src.clone(), dst.clone()));
        Ok(())
    }

    pub fn burn(
        owner: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
    ) -> DispatchResult {
        if let Some(token) = Self::tokens(class_id, token_id) {
            ensure!(
                token.owner == *owner,
                Error::<T>::NoPermission
            );
            Tokens::<T>::remove(class_id, token_id);
            TokenByOwner::<T>::remove((owner.clone(), class_id, token_id));
            Classes::<T>::try_mutate(class_id, |opt| -> DispatchResult{
                let class = opt.as_mut().ok_or(Error::<T>::ClassNotFound)?;
                class.total_issuance = class.total_issuance.checked_sub(&One::one()).ok_or(ArithmeticError::Underflow)?;
                Ok(())
            })?;
        } else {
            return Err(Error::<T>::TokenNotFound.into());
        }
        Self::deposit_event(Event::<T>::TokenBurned(class_id, token_id, owner.clone()));
        Ok(())
    }

    pub fn do_destroy_class(
        owner: &T::AccountId,
        class_id: T::ClassId,
    ) -> DispatchResult {
        if let Some(class) = Self::classes(class_id) {
            ensure!(
            *owner == class.owner,
            Error::<T>::NoPermission
            );
            ensure!(
                class.total_issuance == Zero::zero(),
                Error::<T>::CannotDestroyClass
            );
        } else {
            return Err(Error::<T>::ClassNotFound.into());
        }
        Classes::<T>::remove(class_id);
        NextTokenId::<T>::remove(class_id);
        Self::deposit_event(Event::<T>::ClassDestroyed(class_id, owner.clone()));
        Ok(())
    }

    pub fn is_owner_of(
        owner: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
    ) -> bool {
        TokenByOwner::<T>::contains_key((owner.clone(), class_id, token_id))
    }
}

impl<T: Config> NFTForMarketplace<T::AccountId, T::ClassId, T::TokenId> for Pallet<T>{
	fn transfer(src: &T::AccountId, dst: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) -> DispatchResult {
		Self::transfer(src, dst, class_id, token_id)
	}

	fn is_owner_of(owner: &T::AccountId, class_id: T::ClassId, token_id: T::TokenId) -> bool {
		Self::is_owner_of(owner, class_id, token_id)
	}
}
