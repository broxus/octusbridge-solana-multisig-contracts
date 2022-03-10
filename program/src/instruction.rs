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
    CreateMultisig {
        owners: Vec<Pubkey>,
        threshold: u64,
    },
    /// Creates a new transaction account, automatically signed by the creator,
    /// which must be one of the owners of the multisig
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Proposer account
    ///   1. [WRITE, SIGNER]    Transaction account
    ///   2. []                 Multisig account
    ///   3. []                 System program
    ///   4. []                 The rent sysvar
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
    ExecuteTransaction,
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
    pid: Pubkey,
    accs: Vec<TransactionAccount>,
    data: Vec<u8>,
) -> Instruction {
    let data = MultisigInstruction::CreateTransaction { pid, accs, data }
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
