use crate::{curves::CurveType, mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use frame_system::{ensure_signed, RawOrigin};

#[test]
fn correct_error_for_unsigned_origin_while_creating_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::none(),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn correct_error_for_insufficient_balance_to_reserve_while_creating_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::signed(1),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			Error::<Test>::InsufficientBalanceToReserve,
		);
	});
}

#[test]
fn correct_error_for_asset_already_existing_while_creating_asset() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::signed(2),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			Error::<Test>::AssetAlreadyExists,
		);
	});
}

#[test]
fn it_works_for_create_asset_with_correct_parameters() {
	let mut extrinsic = ExtBuilder::default().with_balances(vec![(1, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
	});
}

#[test]
fn correct_error_for_unsigned_origin_while_minting_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::create_asset(
				Origin::none(),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn correct_error_for_asset_does_not_exist_while_minting_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::mint_asset(Origin::signed(1), 10, 10000,),
			Error::<Test>::AssetDoesNotExist,
		);
	});
}

#[test]
fn correct_error_for_invalid_minter_while_minting_asset() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::mint_asset(Origin::signed(2), 10, 10000,),
			Error::<Test>::InvalidMinter,
		);
	});
}

#[test]
fn it_works_for_mint_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_ok!(PalletYodaBondingCurve::mint_asset(Origin::signed(1), 10, 10000,),);
	});
}

#[test]
fn correct_error_for_unsigned_origin_while_selling_asset() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::sell_asset(
				Origin::none(),
				10,
				ensure_signed(Origin::signed(2)).unwrap(),
				5000,
			),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn it_works_for_sell_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 2000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_ok!(PalletYodaBondingCurve::sell_asset(
			Origin::signed(1),
			10,
			ensure_signed(Origin::signed(2)).unwrap(),
			1000,
		),);
	});
}

#[test]
fn correct_error_while_buying_an_asset_that_does_not_exist() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_noop!(
			PalletYodaBondingCurve::buy_asset(Origin::signed(1), 30, 3000,),
			Error::<Test>::AssetDoesNotExist,
		);
	});
}

#[test]
fn correct_error_while_buying_an_asset_whose_amount_is_greater_than_the_current_token_supply() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_noop!(
			PalletYodaBondingCurve::buy_asset(Origin::signed(2), 10, 3000,),
			Error::<Test>::MintAmountGreaterThanMaxSupply,
		);
	});
}

#[test]
fn it_works_for_buying_as_asset_with_correct_parameters() {
	let mut extrinsic =
		ExtBuilder::default().with_balances(vec![(1, 1000000), (2, 1000000)]).build();
	extrinsic.execute_with(|| {
		assert_ok!(PalletYodaBondingCurve::create_asset(
			Origin::signed(1),
			10,
			10000,
			CurveType::Linear,
			2000,
			b"batman".to_vec(),
			b"bat".to_vec(),
			10
		));
		assert_ok!(PalletYodaBondingCurve::buy_asset(Origin::signed(2), 10, 500,));
	});
}
