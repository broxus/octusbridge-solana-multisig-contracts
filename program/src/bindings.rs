use borsh::BorshSerialize;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use crate::*;

pub fn create_multisig(
    funder_pubkey: &Pubkey,
    seed: u128,
    owners: Vec<Pubkey>,
    threshold: u64,
) -> Instruction {
    let multisig_pubkey = get_multisig_address(seed);

    let data = MultisigInstruction::CreateMultisig {
        seed,
        owners,
        threshold,
    }
    .try_to_vec()
    .expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funder_pubkey, true),
            AccountMeta::new(multisig_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    }
}

pub fn create_transaction(
    funder_pubkey: &Pubkey,
    proposer_pubkey: &Pubkey,
    multisig_pubkey: &Pubkey,
    seed: u128,
    ix: Instruction,
) -> Instruction {
    let mut accounts = ix
        .accounts
        .into_iter()
        .map(|acc| TransactionAccount {
            pubkey: acc.pubkey,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect::<Vec<_>>();
    accounts.push(TransactionAccount {
        pubkey: ix.program_id,
        is_signer: false,
        is_writable: false,
    });

    let transaction_pubkey = get_transaction_address(seed);

    let data = MultisigInstruction::CreateTransaction {
        seed,
        pid: ix.program_id,
        accs: accounts,
        data: ix.data,
    }
    .try_to_vec()
    .expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funder_pubkey, true),
            AccountMeta::new(*proposer_pubkey, true),
            AccountMeta::new(*multisig_pubkey, false),
            AccountMeta::new(transaction_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    }
}

pub fn approve(
    proposer_pubkey: &Pubkey,
    multisig_pubkey: &Pubkey,
    transaction_pubkey: &Pubkey,
) -> Instruction {
    let data = MultisigInstruction::Approve.try_to_vec().expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*proposer_pubkey, true),
            AccountMeta::new(*transaction_pubkey, false),
            AccountMeta::new_readonly(*multisig_pubkey, false),
        ],
        data,
    }
}

pub fn execute_transaction(
    multisig_pubkey: &Pubkey,
    transaction_pubkey: &Pubkey,
    accs: Vec<TransactionAccount>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*multisig_pubkey, false),
        AccountMeta::new(*transaction_pubkey, false),
    ];

    for account in accs {
        let account_meta = match account.is_writable {
            true => AccountMeta::new(account.pubkey, false),
            false => AccountMeta::new_readonly(account.pubkey, false),
        };
        accounts.push(account_meta);
    }

    let data = MultisigInstruction::ExecuteTransaction
        .try_to_vec()
        .expect("pack");

    Instruction {
        program_id: id(),
        accounts,
        data,
    }
}

pub fn delete_pending_transaction(
    multisig_pubkey: &Pubkey,
    pending_transaction: Pubkey,
) -> Instruction {
    let data = MultisigInstruction::DeletePendingTransaction {
        pending_transaction,
    }
    .try_to_vec()
    .expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![AccountMeta::new(*multisig_pubkey, true)],
        data,
    }
}

pub fn get_multisig_address(seed: u128) -> Pubkey {
    Pubkey::find_program_address(&[br"multisig", &seed.to_le_bytes()], &id()).0
}

pub fn get_transaction_address(seed: u128) -> Pubkey {
    Pubkey::find_program_address(&[br"transaction", &seed.to_le_bytes()], &id()).0
}
