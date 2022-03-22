use std::str::FromStr;

use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{bpf_loader_upgradeable, system_program, sysvar};

use wasm_bindgen::prelude::*;

use crate::{
    get_multisig_address, get_transaction_address, id, state, MultisigInstruction, Transaction,
    TransactionAccount,
};

#[wasm_bindgen(js_name = "createMultisig")]
pub fn create_multisig_ix(
    funder_pubkey: String,
    name: String,
    owners: JsValue,
    threshold: u64,
) -> JsValue {
    let funder_pubkey = Pubkey::from_str(funder_pubkey.as_str()).unwrap();

    let owners: Vec<String> = owners.into_serde().unwrap();
    let owners = owners
        .into_iter()
        .map(|x| Pubkey::from_str(x.as_str()).unwrap())
        .collect();

    let multisig_pubkey = get_multisig_address(&name);

    let data = MultisigInstruction::CreateMultisig {
        name,
        owners,
        threshold,
    }
    .try_to_vec()
    .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(funder_pubkey, true),
            AccountMeta::new(multisig_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    };

    return JsValue::from_serde(&ix).unwrap();
}

#[wasm_bindgen(js_name = "createTransaction")]
pub fn create_transaction_ix(
    funder_pubkey: String,
    proposer_pubkey: String,
    multisig_pubkey: String,
    name: String,
    instruction: JsValue,
) -> JsValue {
    let funder_pubkey = Pubkey::from_str(funder_pubkey.as_str()).unwrap();
    let proposer_pubkey = Pubkey::from_str(proposer_pubkey.as_str()).unwrap();
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).unwrap();

    let upgrade_ix: Instruction = instruction.into_serde().unwrap();

    let mut accounts = upgrade_ix
        .accounts
        .into_iter()
        .map(|acc| TransactionAccount {
            pubkey: acc.pubkey,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect::<Vec<_>>();
    accounts.push(TransactionAccount {
        pubkey: upgrade_ix.program_id,
        is_signer: false,
        is_writable: false,
    });

    let transaction_pubkey = get_transaction_address(&name);

    let data = MultisigInstruction::CreateTransaction {
        name,
        pid: upgrade_ix.program_id,
        accs: accounts,
        data: upgrade_ix.data,
    }
    .try_to_vec()
    .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(funder_pubkey, true),
            AccountMeta::new(proposer_pubkey, true),
            AccountMeta::new(multisig_pubkey, false),
            AccountMeta::new(transaction_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    };

    return JsValue::from_serde(&ix).unwrap();
}

#[wasm_bindgen(js_name = "upgrade")]
pub fn upgrade_ix(
    program_pubkey: String,
    buffer_pubkey: String,
    authority_pubkey: String,
    spill_pubkey: String,
) -> JsValue {
    let program_pubkey = Pubkey::from_str(program_pubkey.as_str()).unwrap();
    let buffer_pubkey = Pubkey::from_str(buffer_pubkey.as_str()).unwrap();
    let authority_pubkey = Pubkey::from_str(authority_pubkey.as_str()).unwrap();
    let spill_pubkey = Pubkey::from_str(spill_pubkey.as_str()).unwrap();

    let ix = bpf_loader_upgradeable::upgrade(
        &program_pubkey,
        &buffer_pubkey,
        &authority_pubkey,
        &spill_pubkey,
    );

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

#[wasm_bindgen(js_name = "execute")]
pub fn execute_ix(
    multisig_pubkey: String,
    transaction_pubkey: String,
    transaction_data: Vec<u8>,
) -> JsValue {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).unwrap();
    let transaction_pubkey = Pubkey::from_str(transaction_pubkey.as_str()).unwrap();

    let transaction_data = Transaction::unpack(&transaction_data).unwrap();

    let mut accounts = vec![
        AccountMeta::new(multisig_pubkey, false),
        AccountMeta::new(transaction_pubkey, false),
    ];

    for account in transaction_data.accounts {
        let account_meta = match account.is_writable {
            true => AccountMeta::new(account.pubkey, false),
            false => AccountMeta::new_readonly(account.pubkey, false),
        };
        accounts.push(account_meta);
    }

    let data = MultisigInstruction::ExecuteTransaction
        .try_to_vec()
        .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts,
        data,
    };

    return JsValue::from_serde(&ix).unwrap();
}

#[wasm_bindgen(js_name = "unpackMultisig")]
pub fn unpack_multisig(data: Vec<u8>) -> JsValue {
    let multisig = state::Multisig::unpack(&data).unwrap();

    let msig = MultisigMeta {
        threshold: multisig.threshold,
        owners: multisig.owners,
        pending_transactions: multisig.pending_transactions,
    };

    return JsValue::from_serde(&msig).unwrap();
}

#[wasm_bindgen(js_name = "unpackTransaction")]
pub fn unpack_transaction(data: Vec<u8>) -> JsValue {
    let transaction = state::Transaction::unpack(&data).unwrap();

    let tx = TransactionMeta {
        multisig: transaction.multisig,
        program_id: transaction.program_id,
        signers: transaction.signers,
        did_execute: transaction.did_execute,
    };

    return JsValue::from_serde(&tx).unwrap();
}

#[derive(Serialize, Deserialize)]
pub struct MultisigMeta {
    pub threshold: u64,
    pub owners: Vec<Pubkey>,
    pub pending_transactions: Vec<Pubkey>,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionMeta {
    pub multisig: Pubkey,
    pub program_id: Pubkey,
    pub signers: Vec<bool>,
    pub did_execute: bool,
}
