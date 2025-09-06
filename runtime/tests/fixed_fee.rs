use frame_support::{assert_ok, dispatch::GetDispatchInfo};
use sp_runtime::traits::Dispatchable;
use pallet_transaction_payment::OnChargeTransaction as OnChargeTx;
use sp_keyring::Sr25519Keyring;
use sp_runtime::BuildStorage;
use sp_io::TestExternalities;

use solochain_template_runtime as runtime;

#[test]
fn fixed_fee_is_charged_per_signed_extrinsic() {
    type Runtime = runtime::Runtime;

    let alice = Sr25519Keyring::Alice.to_account_id();
    let bob = Sr25519Keyring::Bob.to_account_id();

    // initial balances
    let initial_alice: runtime::Balance = 1_000 * runtime::UNIT;
    let initial_bob: runtime::Balance = runtime::EXISTENTIAL_DEPOSIT;

    // build genesis storage
    let mut t = frame_system::GenesisConfig::<Runtime>::default()
        .build_storage()
        .unwrap();
    pallet_balances::GenesisConfig::<Runtime> {
        balances: vec![(alice.clone(), initial_alice), (bob.clone(), initial_bob)],
        dev_accounts: None,
    }
    .assimilate_storage(&mut t)
    .unwrap();

    let mut ext = TestExternalities::new(t);
    ext.execute_with(|| {
        // transfer 100 UNIT
        let amount1: runtime::Balance = 100 * runtime::UNIT;
        let call1 = runtime::RuntimeCall::Balances(runtime::BalancesCall::transfer_allow_death {
            dest: bob.clone().into(),
            value: amount1,
        });
        let info1 = call1.get_dispatch_info();

        let alice_before = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        let bob_before = pallet_balances::Pallet::<Runtime>::free_balance(&bob);

        // pre-charge fixed fee and execute
        let liq1 = <runtime::OnChargeTransaction as OnChargeTx<runtime::Runtime>>::withdraw_fee(&alice, &call1, &info1, 0u128.into(), 0u128.into())
            .expect("withdrawal should succeed");
        let post1 = call1
            .dispatch(runtime::RuntimeOrigin::signed(alice.clone()))
            .expect("dispatch should succeed");
        assert_ok!(<runtime::OnChargeTransaction as OnChargeTx<runtime::Runtime>>::correct_and_deposit_fee(
            &alice,
            &info1,
            &post1,
            0u128.into(),
            0u128.into(),
            liq1,
        ));

        let alice_after_1 = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        let bob_after_1 = pallet_balances::Pallet::<Runtime>::free_balance(&bob);

        assert_eq!(
            alice_after_1,
            alice_before - amount1 - runtime::FIXED_FEE
        );
        assert_eq!(bob_after_1, bob_before + amount1);

        // second extrinsic, transfer 2 UNIT
        let amount2: runtime::Balance = 2 * runtime::UNIT;
        let call2 = runtime::RuntimeCall::Balances(runtime::BalancesCall::transfer_allow_death {
            dest: bob.clone().into(),
            value: amount2,
        });
        let info2 = call2.get_dispatch_info();

        let liq2 = <runtime::OnChargeTransaction as OnChargeTx<runtime::Runtime>>::withdraw_fee(&alice, &call2, &info2, 0u128.into(), 0u128.into())
            .expect("withdrawal should succeed");
        let post2 = call2
            .dispatch(runtime::RuntimeOrigin::signed(alice.clone()))
            .expect("dispatch should succeed");
        assert_ok!(<runtime::OnChargeTransaction as OnChargeTx<runtime::Runtime>>::correct_and_deposit_fee(
            &alice,
            &info2,
            &post2,
            0u128.into(),
            0u128.into(),
            liq2,
        ));

        let alice_after_2 = pallet_balances::Pallet::<Runtime>::free_balance(&alice);
        let bob_after_2 = pallet_balances::Pallet::<Runtime>::free_balance(&bob);

        assert_eq!(
            alice_after_2,
            alice_after_1 - amount2 - runtime::FIXED_FEE
        );
        assert_eq!(bob_after_2, bob_after_1 + amount2);
    });
}
