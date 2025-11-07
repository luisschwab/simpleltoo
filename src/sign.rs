//! Signature and Keys

use elements::EcdsaSighashType;
use elements::Transaction;
use elements::TxInWitness;
use elements::confidential;
use elements::hashes::{Hash, sha256};
use elements::schnorr::Keypair;
use elements::secp256k1_zkp::{
    Message, Parity, PublicKey, SECP256K1, SecretKey, XOnlyPublicKey, ecdsa,
};
use elements::sighash::SighashCache;

use crate::FEE_AMOUNT;
use crate::script::build_setup_address_program;

/// Sign the `Setup Withdrawal Transaction`.
///
/// Alice has Bob sign the transaction that refunds Alice in case
/// Bob becomes uncooperative before actually broadcasting the 2-of-2
/// funding transaction, so that she is assured to not have her BTC locked
/// on the multisig forever.
pub(crate) fn sign_setup_withdrawal_transaction(
    transaction: &Transaction,
    alice_update_sk: &SecretKey,
    bob_update_sk: &SecretKey,
) -> Transaction {
    // Re-derive [`XOnlyPubkey`]s from [`SecretKey`]s.
    let alice_update_pk =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(SECP256K1, alice_update_sk)).0;
    let bob_update_pk =
        XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(SECP256K1, bob_update_sk)).0;

    // Re-build the program.
    let witness_program = build_setup_address_program(&alice_update_pk, &bob_update_pk);

    // Re-derive the output value from the transaction and the fee.
    let output_value = match transaction.output[0].value {
        confidential::Value::Explicit(v) => v,
        _ => panic!("Expected explicit value"),
    };
    let value = output_value + FEE_AMOUNT;

    // Hash the unsigned transaction and wrap in in a [`Message`] for signing.
    let sighash = SighashCache::new(&transaction.clone()).segwitv0_sighash(
        0,
        &witness_program,
        confidential::Value::Explicit(value),
        EcdsaSighashType::All,
    );
    let message = Message::from_digest_slice(&sighash[..]).unwrap();

    // Alice and Bob sign the message. In a production implementation,
    // they would sign separately, and only exchange signatures.
    let alice_sig = SECP256K1.sign_ecdsa(&message, alice_update_sk);
    let bob_sig = SECP256K1.sign_ecdsa(&message, bob_update_sk);

    // Push the signatures into the witness.
    //
    // <alice_sig>
    // <bob_sig>
    // <witness_program>
    let mut signed_transaction = transaction.clone();
    signed_transaction.input[0].witness = TxInWitness {
        amount_rangeproof: None,
        inflation_keys_rangeproof: None,
        script_witness: vec![
            [
                &alice_sig.serialize_der()[..],
                &[EcdsaSighashType::All as u8],
            ]
            .concat(),
            [&bob_sig.serialize_der()[..], &[EcdsaSighashType::All as u8]].concat(),
            witness_program.to_bytes(),
        ],
        pegin_witness: vec![],
    };

    signed_transaction
}

/// Verify the `Setup Withdrawal Transaction` signatures.
///
/// This checks that both Alice and Bob have correctly signed the transaction.
pub(crate) fn verify_setup_withdrawal_transaction(
    transaction: &Transaction,
    alice_update_pk: &XOnlyPublicKey,
    bob_update_pk: &XOnlyPublicKey,
) -> Result<(), String> {
    // Re-build the witness program
    let witness_program = build_setup_address_program(alice_update_pk, bob_update_pk);

    // Re-derive the output value from the transaction and the fee
    let output_value = match transaction.output[0].value {
        confidential::Value::Explicit(v) => v,
        _ => return Err("Expected explicit value".to_string()),
    };
    let value = output_value + FEE_AMOUNT;

    // Calculate the sighash
    let sighash = SighashCache::new(transaction).segwitv0_sighash(
        0,
        &witness_program,
        confidential::Value::Explicit(value),
        EcdsaSighashType::All,
    );

    let message = Message::from_digest_slice(&sighash[..])
        .map_err(|e| format!("Failed to create message: {}", e))?;

    // Extract signatures from witness
    let witness = &transaction.input[0].witness.script_witness;
    if witness.len() != 3 {
        return Err(format!(
            "Expected 3 witness elements, got {}",
            witness.len()
        ));
    }

    let alice_sig_bytes = &witness[0];
    let bob_sig_bytes = &witness[1];

    // Parse signatures (remove the sighash type byte at the end)
    if alice_sig_bytes.is_empty() || bob_sig_bytes.is_empty() {
        return Err("Empty signature found".to_string());
    }

    let alice_sig = ecdsa::Signature::from_der(&alice_sig_bytes[..alice_sig_bytes.len() - 1])
        .map_err(|e| format!("Failed to parse Alice's signature: {}", e))?;
    let bob_sig = ecdsa::Signature::from_der(&bob_sig_bytes[..bob_sig_bytes.len() - 1])
        .map_err(|e| format!("Failed to parse Bob's signature: {}", e))?;

    // Convert XOnlyPublicKey to PublicKey for ECDSA verification
    // We use even parity since the witness script doesn't encode parity
    let alice_pubkey = PublicKey::from_x_only_public_key(*alice_update_pk, Parity::Even);
    let bob_pubkey = PublicKey::from_x_only_public_key(*bob_update_pk, Parity::Even);

    // Verify signatures
    SECP256K1
        .verify_ecdsa(&message, &alice_sig, &alice_pubkey)
        .map_err(|e| format!("Alice's signature verification failed: {}", e))?;

    SECP256K1
        .verify_ecdsa(&message, &bob_sig, &bob_pubkey)
        .map_err(|e| format!("Bob's signature verification failed: {}", e))?;

    Ok(())
}

/// Derive a Settlement [`SecretKey`] from an Update [`SecretKey`] and the state index.
///
/// Settlement Key := SHA256(update_key || state_idx)
pub(crate) fn derive_settlement_key(sk_update: &SecretKey, state_idx: u64) -> SecretKey {
    let mut sk_settlement_seed = Vec::with_capacity(32 + 8);

    sk_settlement_seed.extend_from_slice(&sk_update.secret_bytes());
    sk_settlement_seed.extend_from_slice(&state_idx.to_be_bytes());

    let sk_settlement_digest = sha256::Hash::hash(&sk_settlement_seed);

    SecretKey::from_slice(sk_settlement_digest.as_ref()).unwrap_or_else(|_| {
        let mut attempt = sk_settlement_seed;
        loop {
            attempt.extend_from_slice("eltoo".as_bytes());
            let attempt_digest = sha256::Hash::hash(&attempt);

            if let Ok(key) = SecretKey::from_slice(attempt_digest.as_ref()) {
                break key;
            }
        }
    })
}

/// Add the `0x` prefix to a [`XOnlyPublicKey`].
pub(crate) fn prefix_pubkey(pubkey: XOnlyPublicKey) -> String {
    format!("0x{}", pubkey)
}
