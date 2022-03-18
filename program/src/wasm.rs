use std::str::FromStr;

use borsh::BorshSerialize;

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use wasm_bindgen::prelude::*;

use crate::{id, MultisigInstruction};

#[wasm_bindgen(js_name = "createMultisig")]
pub fn create_multisig_ix(
    funder_pubkey: String,
    multisig_pubkey: String,
    owners: JsValue,
    threshold: u64,
) -> JsValue {
    let funder_pubkey = Pubkey::from_str(funder_pubkey.as_str()).unwrap();
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).unwrap();

    let owners: Vec<String> = owners.into_serde().unwrap();
    let owners = owners
        .into_iter()
        .map(|x| Pubkey::from_str(x.as_str()).unwrap())
        .collect();

    let data = MultisigInstruction::CreateMultisig { owners, threshold }
        .try_to_vec()
        .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(funder_pubkey, true),
            AccountMeta::new(multisig_pubkey, true),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    };

    return JsValue::from_serde(&ix).unwrap();
}

#[wasm_bindgen(js_name = "approve")]
pub fn approve_ix(
    proposer_pubkey: String,
    multisig_pubkey: String,
    transaction_pubkey: String,
) -> JsValue {
    let proposer_pubkey = Pubkey::from_str(proposer_pubkey.as_str()).unwrap();
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).unwrap();
    let transaction_pubkey = Pubkey::from_str(transaction_pubkey.as_str()).unwrap();

    let data = MultisigInstruction::Approve.try_to_vec().expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(proposer_pubkey, true),
            AccountMeta::new(transaction_pubkey, false),
            AccountMeta::new_readonly(multisig_pubkey, false),
        ],
        data,
    };

    return JsValue::from_serde(&ix).unwrap();
}