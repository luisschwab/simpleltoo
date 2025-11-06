use std::fmt::format;

use elements::hashes::{Hash, sha256};
use lwk_wollet::secp256k1::SecretKey;
use lwk_wollet::secp256k1::{self, XOnlyPublicKey};
use simplicityhl::{Arguments, CompiledProgram};

use crate::error::Error;

const TEMPLATE_PATH: &str = "scripts/eltoo_commitment_template.simf";

/// Placeholder identifiers in the template file
const SETTLEMENT_KEY_A_PLACEHOLDER: &str = "__SETTLEMENT_KEY_A__";
const SETTLEMENT_KEY_B_PLACEHOLDER: &str = "__SETTLEMENT_KEY_B__";
const STATE_NUMBER_PLACEHOLDER: &str = "__NEXT_STATE_NUMBER__";

pub(crate) fn create_new_commitment_script(
    secp: &secp256k1::Secp256k1<secp256k1::All>,
    settlement_key_a: SecretKey,
    settlement_key_b: SecretKey,
    next_state: u64,
) -> Result<CompiledProgram, Error> {
    let prog_path = std::path::Path::new(TEMPLATE_PATH);
    let mut template = std::fs::read_to_string(prog_path)?;

    let pub_settlement_key_a = settlement_key_a.x_only_public_key(secp);
    let pub_settlement_key_b = settlement_key_b.x_only_public_key(secp);

    // Replace the placeholder values
    let prog_text = populate_template(
        &template,
        pub_settlement_key_a.0,
        pub_settlement_key_b.0,
        next_state,
    );

    let compiled = CompiledProgram::new(prog_text, Arguments::default(), false)
        .map_err(simplicityhl::error::Error::CannotCompile)?;

    Ok(compiled)
}

fn populate_template(
    template: &str,
    pub_key_a: XOnlyPublicKey,
    pub_key_b: XOnlyPublicKey,
    state: u64,
) -> String {
    template
        .replace(SETTLEMENT_KEY_A_PLACEHOLDER, &format_pubkey(pub_key_a))
        .replace(SETTLEMENT_KEY_B_PLACEHOLDER, &format_pubkey(pub_key_b))
        .replace(STATE_NUMBER_PLACEHOLDER, &state.to_string())
}

fn format_pubkey(pubkey: XOnlyPublicKey) -> String {
    format!("0x{}", pubkey)
}

pub(crate) fn derive_settlement_key(update_key: &SecretKey, state: u64) -> [u8; 32] {
    use sha256::Hash;

    let mut key_material = Vec::with_capacity(32 + 8);
    key_material.extend_from_slice(&update_key.secret_bytes());
    key_material.extend_from_slice(&state.to_be_bytes());

    *Hash::hash(&key_material).as_ref()
}

#[cfg(test)]
mod tests {
    use elements::bitcoin::secp256k1::SecretKey;
    use lwk_wollet::secp256k1;
    use simplicityhl::CompiledProgram;

    use crate::compile::{create_new_commitment_script, derive_settlement_key};

    #[test]
    fn test_create_new_commitment_script() {
        let secp = secp256k1::Secp256k1::new();
        let update_key_a = SecretKey::from_slice(&[0xcd; 32]).unwrap();
        let update_key_b = SecretKey::from_slice(&[0xee; 32]).unwrap();
        let next_state = 1;

        let settlement_key_a = derive_settlement_key(&update_key_a, next_state);
        let settlement_key_b = derive_settlement_key(&update_key_b, next_state);
        let compiled =
            create_new_commitment_script(&secp, update_key_a, update_key_b, next_state).unwrap();
    }
}
