use crate::{mock::*, curves::CurveType, Error};
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
fn create_asset() {
    let mut extrinsic = ExtBuilder::default().with_balances(
        vec![
            (1, 1000000),
        ]
    ).build();
    extrinsic.execute_with(|| {
		assert_ok!(
			PalletYodaBondingCurve::create_asset(
				Origin::signed(1),
				10,
				10000,
				CurveType::Linear,
				2000,
				b"batman".to_vec(),
				b"bat".to_vec(),
				10
			)
		);
	});
}

#[test]
fn correct_error_for_asset_already_existing_while_creating_asset() {
    let mut extrinsic = ExtBuilder::default().with_balances(
        vec![
            (1, 1000000),
            (2, 1000000),
        ]
    ).build();
    extrinsic.execute_with(|| {
        assert_ok!(
            PalletYodaBondingCurve::create_asset(
                Origin::signed(1),
                10,
                10000,
                CurveType::Linear,
                2000,
                b"batman".to_vec(),
                b"bat".to_vec(),
                10
            )
        );
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


// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }
