//! SimplEltoo
//!
//! Eltoo payment channel implementation with SimplycityHL.
//!
//! Eltoo, also called LN Symmetry, is a proposed protocol that allows for
//! securing lightning transactions without the current penalty mechanism,
//! since publishing an old channel state does is harmless and does not incur
//! in loss of funds via the penalty transaction. It also allows LN nodes to
//! only store the latest channel state.
//!
//! Using SimplicityHL support on Liquid, we can leverage OP_CTV to implement
//! Eltoo.

use std::str::FromStr;

use elements::bitcoin::Amount;
use elements::hashes::sha256::{self, Midstate};
use elements::schnorr::Keypair;
use elements::secp256k1_zkp::{SECP256K1, SecretKey, XOnlyPublicKey};
use elements::{Address, OutPoint, Txid};
use tracing::{error, info};

use crate::sign::{sign_setup_withdrawal_transaction, verify_setup_withdrawal_transaction};
use crate::transaction::{build_setup_address, build_setup_withdrawal_transaction};

mod error;
mod esplora;
mod faucet;
mod script;
mod sign;
mod transaction;

// L-BTC Testnet [`AssetId`] midstate.
pub(crate) const LBTC_TEST_MIDSTATE: Midstate = sha256::Midstate([
    0x14, 0x4c, 0x65, 0x43, 0x44, 0xaa, 0x71, 0x6d, 0x6f, 0x3a, 0xbc, 0xc1, 0xca, 0x90, 0xe5, 0x64,
    0x1e, 0x4e, 0x2a, 0x7f, 0x63, 0x3b, 0xc0, 0x9f, 0xe3, 0xba, 0xf6, 0x45, 0x85, 0x81, 0x9a, 0x49,
]);

/// The default fee value, in sats.
pub(crate) const FEE_AMOUNT: u64 = 69;

#[allow(unused)]
/// The CSV of 10 blocks, from the Eltoo paper.
pub(crate) const CSV_DELAY: u32 = 10;

/// Alice's hardcoded master key.
pub(crate) const ALICE_MASTER_KEY: &str =
    "39eefd3d3d0082cb2f4a61f41fd394be96151da6fc432fd48bf7419056fb8f2e";
/// Bob's hardcoded master key.
pub(crate) const BOB_MASTER_KEY: &str =
    "39eefd3d3d0082cb2f4a61f41fd394be96151da6fc432fd48bf7419056fb8f2e";

/// Alice's resolution address.
pub(crate) const ALICE_RESOLUTION_ADDRESS: &str =
    "tex1p53ct8hcvnr7zznfjawxwetycthxyv6c06vh4dk2zymc3c3laps5q94kptw";

fn main() {
    tracing_subscriber::fmt().init();

    // Create Alice's Settlement keys.
    let alice_update_sk = SecretKey::from_str(ALICE_MASTER_KEY).unwrap();
    let alice_update_pk =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(SECP256K1, &alice_update_sk)).0;
    info!(
        "Created Alice's SK and derived PK:\n SK={:?}\n PK={:?}",
        alice_update_sk, alice_update_pk
    );
    // Create Bob's Settlement keys.
    let bob_update_sk = SecretKey::from_str(BOB_MASTER_KEY).unwrap();
    let bob_update_pk =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(SECP256K1, &bob_update_sk)).0;
    info!(
        "Created Bob's SK and derived PK:\n SK={:?}\n PK={:?}",
        bob_update_sk, bob_update_pk
    );

    // Create the 2-of-2 setup address between Alice and Bob.
    let ab_multisig_address = build_setup_address(alice_update_pk, bob_update_pk, false);
    info!(
        "Created 2-of-2 setup multisig address between Alice and Bob: {}",
        ab_multisig_address
    );

    // Create a transaction that spends from the multisig and pays out to Alice's resolution
    // address.
    //
    // Here we fabricate a fictional prevout for convenience.
    // That would be the [`OutPoint`] that funded the `ab_multisig_address`.
    let funding_amount = Amount::from_sat(2140);
    let prevout = OutPoint {
        txid: Txid::from_str("000000000000000000001cd7e92aaf365e841cdd39f19360139b7baef188992f")
            .unwrap(),
        vout: 0,
    };
    let alice_resolution_address = Address::from_str(ALICE_RESOLUTION_ADDRESS).unwrap();
    let unsigned_setup_withdrawal_transaction = build_setup_withdrawal_transaction(
        prevout,
        funding_amount,
        alice_resolution_address,
        false,
    );
    info!(
        "Created unsigned `Setup Withdrawal Transaction` from the 2-of-2 multisig back to Alice's resolution address"
    );

    // Both Alice and Bob sign the withdrawal transaction,
    // BEFORE Alice actually broadcasts the funding to the address.
    // This way, Alice's coins can be recovered if Bob becomes uncooperative.
    let signed_setup_withdrawal_transaction = sign_setup_withdrawal_transaction(
        &unsigned_setup_withdrawal_transaction,
        &alice_update_sk,
        &bob_update_sk,
    );
    info!("Alice and Bob signed the `Setup Withdrawal Transaction`");

    // Verify that the signature is good.
    info!("Verifying `Setup Withdrawal Transaction` signaures...");
    match verify_setup_withdrawal_transaction(
        &signed_setup_withdrawal_transaction,
        &alice_update_pk,
        &bob_update_pk,
    ) {
        Ok(()) => {
            info!("Good signatures for the `Setup Withdrawal Transaction`!")
        }
        Err(e) => {
            error!(
                "Bad signatures for the `Setup Withdrawal Transaction`: {}",
                e
            );
        }
    }
}
