use elements::{
    Address, AddressParams, AssetId, LockTime, OutPoint, Transaction, TxIn, TxOut, TxOutWitness,
    bitcoin::Amount, confidential, confidential::Asset, schnorr::XOnlyPublicKey,
};

use crate::script::build_setup_address_program;
use crate::{FEE_AMOUNT, LBTC_TEST_MIDSTATE};

/// Build a `Setup Address`. It is a 2-of-2 multisig created between
/// both parties Settlement (A_s, B_s) or Update (A_u, B_u) keys
/// (here we will use the Update Keys, as they are invariant).
///
/// The channel's initial balance is reflected in the transaction
/// that funds this address.
pub(crate) fn build_setup_address(
    alice_update_pk: XOnlyPublicKey,
    bob_update_pk: XOnlyPublicKey,
    is_mainnet: bool,
) -> Address {
    // The setup script is just a 2-of-2 between Alice and Bob.
    let setup_witness_program = build_setup_address_program(&alice_update_pk, &bob_update_pk);

    match is_mainnet {
        true => Address::p2wsh(&setup_witness_program, None, &AddressParams::LIQUID),
        false => Address::p2wsh(&setup_witness_program, None, &AddressParams::LIQUID_TESTNET),
    }
}

/// Build the unsigned `Setup Withdrawal Transaction`. This transaction spends
/// from the initial 2-of-2 and pays out back to Alice, and is signed
/// by Bob **before** Alice funds the 2-of2 and broadcasts it. This
/// guarantees that Alice can recover her funds if Bob becomes uncooperative.
///
/// The `funding_prevout` and `amount` comes from the unbroadcast funding transaction to the 2-of-2.
pub(crate) fn build_setup_withdrawal_transaction(
    funding_prevout: OutPoint,
    funding_amount: Amount,
    alice_resolution_address: Address,
    is_mainnet: bool,
) -> Transaction {
    let asset = match is_mainnet {
        true => Asset::Explicit(AssetId::LIQUID_BTC),
        false => Asset::Explicit(AssetId::from_inner(LBTC_TEST_MIDSTATE)),
    };

    let funding_amount = funding_amount.to_sat();

    Transaction {
        version: 2,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: funding_prevout,
            ..Default::default()
        }],
        output: vec![TxOut {
            asset,
            value: confidential::Value::Explicit(funding_amount - FEE_AMOUNT),
            nonce: confidential::Nonce::Null,
            script_pubkey: alice_resolution_address.script_pubkey(),
            witness: TxOutWitness {
                surjection_proof: None,
                rangeproof: None,
            },
        }],
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use elements::secp256k1_zkp::{SECP256K1, SecretKey};
    use tracing::info;

    use crate::{ALICE_MASTER_KEY, BOB_MASTER_KEY};

    use super::*;

    #[test]
    fn setup_address() {
        tracing_subscriber::fmt().init();

        let alice_settlement_sk = SecretKey::from_str(ALICE_MASTER_KEY).unwrap();
        let alice_settlement_pk = alice_settlement_sk.x_only_public_key(SECP256K1).0;

        let bob_settlement_sk = SecretKey::from_str(BOB_MASTER_KEY).unwrap();
        let bob_settlement_pk = bob_settlement_sk.x_only_public_key(SECP256K1).0;

        let setup_address = build_setup_address(alice_settlement_pk, bob_settlement_pk, false);
        info!(
            "Built address {} from A_s = {} and B_s = {}",
            setup_address, alice_settlement_pk, bob_settlement_pk
        );
    }
}
