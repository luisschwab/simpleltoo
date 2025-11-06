// TODO(@luisschwab): remove this
#![allow(unused)]

use std::str::FromStr;

use elements::{
    AssetId, OutPoint, Script, Transaction, TxOut, TxOutWitness, Txid,
    bitcoin::bip32::Xpriv,
    confidential,
    hashes::sha256,
    pset::{Input, Output, PartiallySignedTransaction as Pset},
};
use lwk_signer::SwSigner;
use lwk_wollet::{elements_miniscript::psbt::PsbtExt, secp256k1::SecretKey};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod compile;
mod error;
mod esplora;
mod faucet;
mod sign;
mod transaction;

pub const LIQUID_TESTNET_BTC: AssetId = AssetId::from_inner(sha256::Midstate([
    0x38, 0xfc, 0xa2, 0xd9, 0x39, 0x69, 0x60, 0x61, 0xa8, 0xf7, 0x6d, 0x4e, 0x6b, 0x5e, 0xec, 0xd5,
    0x4e, 0x3b, 0x42, 0x21, 0xc8, 0x46, 0xf2, 0x4a, 0x6b, 0x27, 0x9e, 0x79, 0x95, 0x28, 0x50, 0xa5,
]));

const UPDATE_KEY_A: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
];

const UPDATE_KEY_B: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2,
];

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let update_key_a = SecretKey::from_slice(&UPDATE_KEY_A).unwrap();
    let update_key_b = SecretKey::from_slice(&UPDATE_KEY_B).unwrap();

    let funding_script = compile::build_funding_transaction(update_key_a, update_key_b).unwrap();
    let address = compile::derive_address(&funding_script, false);
    info!("Funding address: {}", address);

    //let txid = faucet::get_coins(&address).await.unwrap();
    let txid =
        Txid::from_str("8f918dce5094f7ff8da0e57c4d1abc6909b658419ee75e46aafbd83500ba90ac").unwrap();

    info!("Txid: {:?}", txid);
    let state = 0;
    let settlement_key_a = compile::derive_settlement_key(&update_key_a, state);
    let settlement_key_b = compile::derive_settlement_key(&update_key_b, state);
    let trigger_transaction_program =
        compile::build_new_commitment_script(settlement_key_a, settlement_key_b, state + 1)
            .unwrap();
    let address_trigger = compile::derive_address(&trigger_transaction_program, false);
    let mut pset = Pset::new_v2();
    let input = Input::from_prevout(OutPoint::new(txid, 0));
    let txout = TxOut {
        asset: confidential::Asset::Explicit(LIQUID_TESTNET_BTC),
        value: confidential::Value::Explicit(100_000),
        nonce: confidential::Nonce::Null,
        script_pubkey: address_trigger.script_pubkey(),
        witness: TxOutWitness::empty(),
    };
    let output = Output::from_txout(txout);
    pset.add_input(input);
    pset.add_output(output);

    // info!("Pset: {:?}", pset);
    // info!("Pset sanity check: {:?}", pset.sanity_check());
    let transaction = pset.extract_tx().unwrap();

    info!("Transaction: {:?}", transaction);
}
