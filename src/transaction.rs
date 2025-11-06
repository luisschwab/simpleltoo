//! Transaction PSBTs
//!
//! 1. Trigger
//! 2. Settlement
//! 3. Fund Multisig
//! 4. Floating (aka Commitment) + Settlement for each

use elements::{
    Sequence, Transaction, TxIn, TxOut,
    bitcoin::{Amount, OutPoint},
    pset::PartiallySignedTransaction as Pset,
};

pub(crate) fn build_funding_transaction() -> Pset {
    unimplemented!()
}

pub(crate) fn build_trigger_transaction() -> Pset {
    unimplemented!()
}

/// Build the transaction that represents the
pub(crate) fn build_floating_transaction() -> Pset {
    unimplemented!()
}

/// Build the transaction that settles the channel's balance at a given state.
pub(crate) fn build_settlement_transaction() -> Pset {
    unimplemented!()
}
