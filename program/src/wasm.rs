use std::str::FromStr;

use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::{bpf_loader_upgradeable, system_program, sysvar};

use crate::*;

#[wasm_bindgen(js_name = "createMultisig")]
pub fn create_multisig_ix(
    funder_pubkey: String,
    seed: String,
    owners: JsValue,
    threshold: u64,
) -> Result<JsValue, JsValue> {
    let funder_pubkey = Pubkey::from_str(funder_pubkey.as_str()).handle_error()?;

    let owners: Vec<String> = owners.into_serde().handle_error()?;
    let owners = owners
        .into_iter()
        .map(|x| Pubkey::from_str(x.as_str()).unwrap())
        .collect();

    let seed = uuid::Uuid::from_str(&seed).handle_error()?.as_u128();
    let multisig_pubkey = get_multisig_address(seed);

    let data = MultisigInstruction::CreateMultisig {
        seed,
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

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "addOwner")]
pub fn add_owner_ix(multisig_pubkey: String, owner: String) -> Result<JsValue, JsValue> {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;
    let owner = Pubkey::from_str(owner.as_str()).handle_error()?;

    let data = MultisigInstruction::AddOwner { owner }
        .try_to_vec()
        .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![AccountMeta::new(multisig_pubkey, true)],
        data,
    };

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "deleteOwner")]
pub fn delete_owner_ix(multisig_pubkey: String, owner: String) -> Result<JsValue, JsValue> {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;
    let owner = Pubkey::from_str(owner.as_str()).handle_error()?;

    let data = MultisigInstruction::DeleteOwner { owner }
        .try_to_vec()
        .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![AccountMeta::new(multisig_pubkey, true)],
        data,
    };

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "updateThreshold")]
pub fn update_threshold_ix(multisig_pubkey: String, threshold: u64) -> Result<JsValue, JsValue> {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;

    let data = MultisigInstruction::UpdateThreshold { threshold }
        .try_to_vec()
        .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![AccountMeta::new(multisig_pubkey, true)],
        data,
    };

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "createTransaction")]
pub fn create_transaction_ix(
    funder_pubkey: String,
    proposer_pubkey: String,
    multisig_pubkey: String,
    seed: String,
    instruction: JsValue,
) -> Result<JsValue, JsValue> {
    let funder_pubkey = Pubkey::from_str(funder_pubkey.as_str()).handle_error()?;
    let proposer_pubkey = Pubkey::from_str(proposer_pubkey.as_str()).handle_error()?;
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;

    let upgrade_ix: Instruction = instruction.into_serde().handle_error()?;

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

    let seed = uuid::Uuid::from_str(&seed).handle_error()?.as_u128();
    let transaction_pubkey = get_transaction_address(seed);

    let data = MultisigInstruction::CreateTransaction {
        seed,
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

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "setProgramAuthority")]
pub fn set_program_authority_ix(
    current_authority_pubkey: String,
    new_authority_pubkey: String,
    program_address: String,
) -> Result<JsValue, JsValue> {
    let current_authority_pubkey =
        Pubkey::from_str(current_authority_pubkey.as_str()).handle_error()?;
    let new_authority_pubkey = Pubkey::from_str(new_authority_pubkey.as_str()).handle_error()?;
    let program_address = Pubkey::from_str(program_address.as_str()).handle_error()?;

    let ix = bpf_loader_upgradeable::set_upgrade_authority(
        &program_address,
        &current_authority_pubkey,
        Some(&new_authority_pubkey),
    );

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "upgradeProgram")]
pub fn upgrade_program_ix(
    program_pubkey: String,
    buffer_pubkey: String,
    authority_pubkey: String,
    spill_pubkey: String,
) -> Result<JsValue, JsValue> {
    let program_pubkey = Pubkey::from_str(program_pubkey.as_str()).handle_error()?;
    let buffer_pubkey = Pubkey::from_str(buffer_pubkey.as_str()).handle_error()?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey.as_str()).handle_error()?;
    let spill_pubkey = Pubkey::from_str(spill_pubkey.as_str()).handle_error()?;

    let ix = bpf_loader_upgradeable::upgrade(
        &program_pubkey,
        &buffer_pubkey,
        &authority_pubkey,
        &spill_pubkey,
    );

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "closeProgram")]
pub fn close_program_ix(
    program_pubkey: String,
    authority_pubkey: String,
    recipient_pubkey: String,
) -> Result<JsValue, JsValue> {
    let program_pubkey = Pubkey::from_str(program_pubkey.as_str()).handle_error()?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey.as_str()).handle_error()?;
    let recipient_pubkey = Pubkey::from_str(recipient_pubkey.as_str()).handle_error()?;

    let close_pubkey =
        Pubkey::find_program_address(&[program_pubkey.as_ref()], &bpf_loader_upgradeable::id()).0;

    let ix = bpf_loader_upgradeable::close_any(
        &close_pubkey,
        &recipient_pubkey,
        Some(&authority_pubkey),
        Some(&program_pubkey),
    );

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "closeBuffer")]
pub fn close_buffer_ix(
    buffer_pubkey: String,
    authority_pubkey: String,
    recipient_pubkey: String,
) -> Result<JsValue, JsValue> {
    let buffer_pubkey = Pubkey::from_str(buffer_pubkey.as_str()).handle_error()?;
    let authority_pubkey = Pubkey::from_str(authority_pubkey.as_str()).handle_error()?;
    let recipient_pubkey = Pubkey::from_str(recipient_pubkey.as_str()).handle_error()?;

    let ix = bpf_loader_upgradeable::close_any(
        &buffer_pubkey,
        &recipient_pubkey,
        Some(&authority_pubkey),
        None,
    );

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "approve")]
pub fn approve_ix(
    proposer_pubkey: String,
    multisig_pubkey: String,
    transaction_pubkey: String,
) -> Result<JsValue, JsValue> {
    let proposer_pubkey = Pubkey::from_str(proposer_pubkey.as_str()).handle_error()?;
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;
    let transaction_pubkey = Pubkey::from_str(transaction_pubkey.as_str()).handle_error()?;

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

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "deletePendingTransactions")]
pub fn delete_pending_transaction_ix(
    multisig_pubkey: String,
    pending_transaction: String,
) -> Result<JsValue, JsValue> {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;
    let pending_transaction = Pubkey::from_str(pending_transaction.as_str()).handle_error()?;

    let data = MultisigInstruction::DeletePendingTransaction {
        pending_transaction,
    }
    .try_to_vec()
    .expect("pack");

    let ix = Instruction {
        program_id: id(),
        accounts: vec![AccountMeta::new(multisig_pubkey, true)],
        data,
    };

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "execute")]
pub fn execute_ix(
    multisig_pubkey: String,
    transaction_pubkey: String,
    transaction_data: Vec<u8>,
) -> Result<JsValue, JsValue> {
    let multisig_pubkey = Pubkey::from_str(multisig_pubkey.as_str()).handle_error()?;
    let transaction_pubkey = Pubkey::from_str(transaction_pubkey.as_str()).handle_error()?;

    let transaction_data = Transaction::unpack_from_slice(&transaction_data).handle_error()?;

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

    return JsValue::from_serde(&ix).handle_error();
}

#[wasm_bindgen(js_name = "unpackMultisig")]
pub fn unpack_multisig(data: Vec<u8>) -> Result<JsValue, JsValue> {
    let multisig = Multisig::unpack(&data).handle_error()?;

    let msig = WasmMultisigMeta {
        threshold: multisig.threshold,
        owners: multisig.owners,
        pending_transactions: multisig.pending_transactions,
    };

    return JsValue::from_serde(&msig).handle_error();
}

#[wasm_bindgen(js_name = "unpackTransaction")]
pub fn unpack_transaction(data: Vec<u8>) -> Result<JsValue, JsValue> {
    let transaction = Transaction::unpack_from_slice(&data).handle_error()?;

    let tx = WasmTransactionMeta {
        multisig: transaction.multisig,
        program_id: transaction.program_id,
        signers: transaction.signers,
        accounts: transaction.accounts,
        did_execute: transaction.did_execute,
        data: transaction.data,
    };

    return JsValue::from_serde(&tx).handle_error();
}

#[derive(Serialize, Deserialize)]
pub struct WasmMultisigMeta {
    pub threshold: u64,
    pub owners: Vec<Pubkey>,
    pub pending_transactions: Vec<Pubkey>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmTransactionMeta {
    pub multisig: Pubkey,
    pub program_id: Pubkey,
    pub signers: Vec<bool>,
    pub accounts: Vec<TransactionAccount>,
    pub did_execute: bool,
    pub data: Vec<u8>,
}

impl<T, E> HandleError for Result<T, E>
where
    E: ToString,
{
    type Output = T;

    fn handle_error(self) -> Result<Self::Output, JsValue> {
        self.map_err(|e| {
            let error = e.to_string();
            js_sys::Error::new(&error).unchecked_into()
        })
    }
}

pub trait HandleError {
    type Output;

    fn handle_error(self) -> Result<Self::Output, JsValue>;
}
