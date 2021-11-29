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

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(TemplateModule::something(), Some(42));
// 	});
// }
