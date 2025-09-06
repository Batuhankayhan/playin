use core::marker::PhantomData;

use frame_support::traits::fungible::{Balanced, Credit, Inspect};
use frame_support::traits::tokens::{Fortitude, Precision, Preservation, WithdrawConsequence};
use frame_support::weights::{Weight, WeightToFee};
use frame_support::unsigned::TransactionValidityError;
use sp_runtime::traits::{DispatchInfoOf, PostDispatchInfoOf, Zero, SaturatedConversion};
use sp_runtime::transaction_validity::InvalidTransaction;

pub struct FixedFeeCharger<T>(PhantomData<T>);

impl<T> pallet_transaction_payment::OnChargeTransaction<T> for FixedFeeCharger<T>
where
    T: pallet_transaction_payment::Config,
    crate::Balances: Balanced<<T as frame_system::Config>::AccountId>,
{
    type Balance = crate::Balance;
    type LiquidityInfo = Option<Credit<<T as frame_system::Config>::AccountId, crate::Balances>>;

    fn withdraw_fee(
        who: &T::AccountId,
        _call: &T::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<Self::LiquidityInfo, TransactionValidityError> {
        let fee: <crate::Balances as Inspect<<T as frame_system::Config>::AccountId>>::Balance =
            crate::FIXED_FEE.saturated_into();
        if fee.is_zero() {
            return Ok(None);
        }
        match <crate::Balances as Balanced<<T as frame_system::Config>::AccountId>>::withdraw(
            who,
            fee,
            Precision::Exact,
            Preservation::Preserve,
            Fortitude::Polite,
        ) {
            Ok(imbalance) => Ok(Some(imbalance)),
            Err(_) => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn can_withdraw_fee(
        who: &T::AccountId,
        _call: &T::RuntimeCall,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _fee: Self::Balance,
        _tip: Self::Balance,
    ) -> Result<(), TransactionValidityError> {
        let fee: <crate::Balances as Inspect<<T as frame_system::Config>::AccountId>>::Balance =
            crate::FIXED_FEE.saturated_into();
        if fee.is_zero() {
            return Ok(());
        }
        match <crate::Balances as Inspect<<T as frame_system::Config>::AccountId>>::can_withdraw(who, fee) {
            WithdrawConsequence::Success => Ok(()),
            _ => Err(InvalidTransaction::Payment.into()),
        }
    }

    fn correct_and_deposit_fee(
        _who: &T::AccountId,
        _dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
        _post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        _corrected_fee: Self::Balance,
        _tip: Self::Balance,
        already_withdrawn: Self::LiquidityInfo,
    ) -> Result<(), TransactionValidityError> {
        if let Some(paid) = already_withdrawn {
            drop(paid);
        }
        Ok(())
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn endow_account(who: &T::AccountId, amount: Self::Balance) {
        let _ = <crate::Balances as Balanced<_>>::deposit(who, amount, Precision::BestEffort);
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn minimum_balance() -> Self::Balance {
        <crate::Balances as Inspect<<T as frame_system::Config>::AccountId>>::minimum_balance()
    }
}

pub struct ZeroToFee<B>(PhantomData<B>);

impl<B> WeightToFee for ZeroToFee<B>
where
    B: frame_support::traits::tokens::Balance,
{
    type Balance = B;
    fn weight_to_fee(_: &Weight) -> Self::Balance {
        Zero::zero()
    }
}
