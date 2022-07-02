#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn create_class_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NFT::do_create_class(&ALICE, vec![1], ()));
        assert_ok!(NFT::create_class(Origin::signed(BOB), vec![2], ()));
    });
}

#[test]
fn create_class_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        NextClassId::<Runtime>::mutate(|id| *id = <Runtime as Config>::ClassId::max_value());
        assert_noop!(
           NFT::do_create_class(&ALICE, vec![1], ()),
           Error::<Runtime>::NoAvailableClassId
       );
    });
}

#[test]
fn should_build_genesis_nfts() {
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_eq!(NextClassId::<Runtime>::get(), 3);
        assert_eq!(NextTokenId::<Runtime>::get(0), 3);
        assert_eq!(NextTokenId::<Runtime>::get(1), 3);
        assert_eq!(NextTokenId::<Runtime>::get(2), 0);
        assert_eq!(NFT::is_owner_of(&ALICE, 0, 0), true);
        assert_eq!(NFT::is_owner_of(&BOB, 0, 1), true);
        assert_eq!(NFT::is_owner_of(&PETER, 0, 2), true);
        assert_eq!(NFT::is_owner_of(&ONLY, 1, 0), true);
        assert_eq!(NFT::is_owner_of(&ALICE, 1, 1), true);
        assert_eq!(NFT::is_owner_of(&PETER, 1, 2), true);
        assert_eq!(NFT::is_owner_of(&ONLY, 2, 0), false);
    });
}

#[test]
fn mint_token_should_work(){
    ExtBuilder::default().build_with_genesis(
        vec![
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_ok!(NFT::mint_token(Origin::signed(ONLY), ALICE, 0, vec![0], ()));
        assert_eq!(NFT::is_owner_of(&ALICE, 0, 0), true);
    });
}

#[test]
fn mint_token_should_fail(){
    ExtBuilder::default().build_with_genesis(
        vec![
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_noop!(NFT::mint_token(Origin::signed(ALICE), ALICE, 0, vec![0], ()), Error::<Runtime>::NoPermission);
    });
}

#[test]
fn should_burn_token_work() {
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_ok!(NFT::burn_token(Origin::signed(ALICE), 0, 0));
        assert_eq!(NFT::is_owner_of(&ALICE, 0, 0), false);
    });
}

#[test]
fn should_burn_token_fail() {
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_noop!(
            NFT::burn_token(Origin::signed(BOB), 0, 0),
            Error::<Runtime>::NoPermission
        );
    });
}

#[test]
fn should_transfer_token_work() {
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_ok!(NFT::transfer_token(Origin::signed(ALICE), ONLY, 0, 0));
        assert_eq!(NFT::is_owner_of(&ALICE, 0, 0), false);
        assert_eq!(NFT::is_owner_of(&ONLY, 0, 0), true);
    });
}

#[test]
fn should_transfer_token_fail() {
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_noop!(
            NFT::transfer_token(Origin::signed(BOB), ONLY, 0, 0),
            Error::<Runtime>::NoPermission
        );
    });
}

#[test]
fn should_destroy_class_work(){
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_ok!(NFT::burn_token(Origin::signed(ALICE), 0, 0));
        assert_ok!(NFT::burn_token(Origin::signed(BOB), 0, 1));
        assert_ok!(NFT::burn_token(Origin::signed(PETER), 0, 2));
        assert_ok!(NFT::destroy_class(Origin::signed(ALICE), 0));
    });
}

#[test]
fn should_destroy_class_fail(){
    ExtBuilder::default().build_with_genesis(
        vec![
            (ALICE, vec![1], (), vec![
                (ALICE, vec![1], ()),
                (BOB, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (BOB, vec![2], (), vec![
                (ONLY, vec![1], ()),
                (ALICE, vec![2], ()),
                (PETER, vec![3], ()),
            ]),
            (ONLY, vec![3], (), vec![]),
        ]
    ).execute_with(|| {
        assert_noop!(NFT::destroy_class(Origin::signed(ALICE), 0), Error::<Runtime>::CannotDestroyClass);
        assert_ok!(NFT::burn_token(Origin::signed(ALICE), 0, 0));
        assert_ok!(NFT::burn_token(Origin::signed(BOB), 0, 1));
        assert_noop!(NFT::destroy_class(Origin::signed(ALICE), 0), Error::<Runtime>::CannotDestroyClass);
        assert_ok!(NFT::burn_token(Origin::signed(PETER), 0, 2));
        assert_noop!(NFT::destroy_class(Origin::signed(BOB), 0), Error::<Runtime>::NoPermission);
    });
}