#![allow(unused)]

use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use elements::opcodes::all::{OP_CHECKSIG, OP_CHECKSIGVERIFY};
use elements::script::Builder;
use elements::secp256k1_zkp::{self, SecretKey, XOnlyPublicKey};
use elements::{
    secp256k1_zkp::SECP256K1,
    {Address, AddressParams, Script},
};
use hal_simplicity::hal_simplicity::{Program, elements_address};
use simplicityhl::{Arguments, CompiledProgram, simplicity::jet};

use crate::error::Error;
use crate::sign::{derive_settlement_key, prefix_pubkey};

const TEMPLATE_PATH: &str = "scripts/eltoo_commitment_template.simf";
const FUNDING_TRANSACTION_TEMPLATE_PATH: &str = "scripts/funding_transaction.simf";

/// Placeholder identifiers in the template file.
const SETTLEMENT_KEY_A_PLACEHOLDER: &str = "__SETTLEMENT_KEY_A__";
const SETTLEMENT_KEY_B_PLACEHOLDER: &str = "__SETTLEMENT_KEY_B__";
const STATE_NUMBER_PLACEHOLDER: &str = "__NEXT_STATE_NUMBER__";

/// Build the Witness Program for the 2-of2 setup address between Alice and Bob.
pub(crate) fn build_setup_address_program(
    alice_update_pk: &XOnlyPublicKey,
    bob_update_pk: &XOnlyPublicKey,
) -> Script {
    Builder::new()
        .push_slice(&alice_update_pk.serialize())
        .push_opcode(OP_CHECKSIGVERIFY)
        .push_slice(&bob_update_pk.serialize())
        .push_opcode(OP_CHECKSIG)
        .into_script()
}

/// Build the `ELTOO` commitment script from both parties settlement [`SecretKey`]s.
pub(crate) fn build_new_commitment_script(
    settlement_key_a: SecretKey,
    settlement_key_b: SecretKey,
    next_state: u64,
) -> Result<CompiledProgram, Error> {
    let prog_path = std::path::Path::new(TEMPLATE_PATH);
    let template = std::fs::read_to_string(prog_path)?;

    let pub_settlement_key_a = settlement_key_a.x_only_public_key(SECP256K1);
    let pub_settlement_key_b = settlement_key_b.x_only_public_key(SECP256K1);

    // Replace the placeholders with actual values.
    let prog_text = populate_template(
        &template,
        pub_settlement_key_a.0,
        pub_settlement_key_b.0,
        next_state,
    );

    // Compile the `SimplicityHL` program from it's string.
    let compiled = CompiledProgram::new(prog_text, Arguments::default(), false)
        .map_err(simplicityhl::error::Error::CannotCompile)?;

    Ok(compiled)
}

pub(crate) fn build_funding_transaction(
    key1: SecretKey,
    key2: SecretKey,
) -> Result<CompiledProgram, Error> {
    let prog_path = std::path::Path::new(FUNDING_TRANSACTION_TEMPLATE_PATH);
    let prog_text = std::fs::read_to_string(prog_path).unwrap();
    let compiled = CompiledProgram::new(prog_text, Arguments::default(), false)
        .map_err(simplicityhl::error::Error::CannotCompile)?;

    Ok(compiled)
}

/// Derive an Elements [`Address`] from a `SimplicityHL` program.
pub(crate) fn derive_address(program: &CompiledProgram, is_mainnet: bool) -> Address {
    let commited = program.commit();

    let script_bytes = Script::from(commited.to_vec_without_witness());
    let script_base64 = Base64Display::new(&script_bytes.to_bytes(), &STANDARD).to_string();
    let program = Program::<jet::Elements>::from_str(&script_base64, None).unwrap();

    if is_mainnet {
        elements_address(program.cmr(), &AddressParams::LIQUID)
    } else {
        elements_address(program.cmr(), &AddressParams::LIQUID_TESTNET)
    }
}

/// Populate the ELTOO commitment template with both parties pubkeys and the state index.
fn populate_template(
    template: &str,
    pub_key_a: XOnlyPublicKey,
    pub_key_b: XOnlyPublicKey,
    state: u64,
) -> String {
    template
        .replace(SETTLEMENT_KEY_A_PLACEHOLDER, &prefix_pubkey(pub_key_a))
        .replace(SETTLEMENT_KEY_B_PLACEHOLDER, &prefix_pubkey(pub_key_b))
        .replace(STATE_NUMBER_PLACEHOLDER, &state.to_string())
}

#[cfg(test)]
mod tests {
    use elements::bitcoin::secp256k1::SecretKey;
    use lwk_wollet::secp256k1;
    use simplicityhl::CompiledProgram;

    use super::*;

    #[test]
    fn test_create_new_commitment_script() {
        let update_key_a = SecretKey::from_slice(&[0xcd; 32]).unwrap();
        let update_key_b = SecretKey::from_slice(&[0xee; 32]).unwrap();
        let next_state = 1;

        let settlement_key_a = derive_settlement_key(&update_key_a, next_state);
        let settlement_key_b = derive_settlement_key(&update_key_b, next_state);
        let compiled =
            build_new_commitment_script(settlement_key_a, settlement_key_b, next_state).unwrap();
    }
}
