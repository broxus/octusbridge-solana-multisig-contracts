use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{system_program, sysvar};

use crate::{id, TransactionAccount};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MultisigInstruction {
    /// Initializes a new multisig account with a set of owners and a threshold
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Funder account
    ///   1. [WRITE, SIGNER]    Multisig account
    ///   2. []                 System program
    ///   3. []                 The rent sysvar
    CreateMultisig { owners: Vec<Pubkey>, threshold: u64 },
    /// Creates a new transaction account, automatically signed by the creator,
    /// which must be one of the owners of the multisig
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Funder account
    ///   1. [WRITE, SIGNER]    Proposer account
    ///   2. [WRITE, SIGNER]    Transaction account
    ///   3. []                 Multisig account
    ///   4. []                 System program
    ///   5. []                 The rent sysvar
    CreateTransaction {
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    },
    /// Approves a transaction on behalf of an owner of the multisig
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Proposer account
    ///   1. [WRITE]            Transaction account
    ///   2. []                 Multisig account
    Approve,
    /// Execute transaction
    ///
    /// # Account references
    ///   0. [WRITE]            Transaction account
    ///   1. []                 Multisig account
    ///   2. []                 Multisig Signer account
    ///   ...                   Set instruction accounts
    ExecuteTransaction,
    /// Execute transaction
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Funder account
    ///   1. [WRITE, SIGNER]    Multisig account
    ///   2. []                 System program
    ///   3. []                 The rent sysvar
    CreateMultisigSigner,
}

pub fn create_multisig(
    funder_pubkey: &Pubkey,
    multisig_pubkey: &Pubkey,
    owners: Vec<Pubkey>,
    threshold: u64,
) -> Instruction {
    let data = MultisigInstruction::CreateMultisig { owners, threshold }
        .try_to_vec()
        .expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funder_pubkey, true),
            AccountMeta::new(*multisig_pubkey, true),
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
    transaction_pubkey: &Pubkey,
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

    let data = MultisigInstruction::CreateTransaction {
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
            AccountMeta::new(*transaction_pubkey, true),
            AccountMeta::new_readonly(*multisig_pubkey, false),
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
    let multisig_signer_pubkey = get_multisig_signer_address(&multisig_pubkey);

    let mut accounts = vec![
        AccountMeta::new(*transaction_pubkey, false),
        AccountMeta::new_readonly(*multisig_pubkey, false),
        AccountMeta::new_readonly(multisig_signer_pubkey, false),
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

pub fn create_multisig_signer(funder_pubkey: &Pubkey, multisig_pubkey: &Pubkey) -> Instruction {
    let multisig_signer_pubkey = get_multisig_signer_address(&multisig_pubkey);

    let data = MultisigInstruction::CreateMultisigSigner
        .try_to_vec()
        .expect("pack");

    Instruction {
        program_id: id(),
        accounts: vec![
            AccountMeta::new(*funder_pubkey, true),
            AccountMeta::new(multisig_signer_pubkey, false),
            AccountMeta::new_readonly(*multisig_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data,
    }
}

pub fn get_multisig_signer_address(multisig: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[&multisig.to_bytes()], &id()).0
}
