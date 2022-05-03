use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::TransactionAccount;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MultisigInstruction {
    /// Initializes a new multisig account with a set of owners and a threshold
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Funder account
    ///   1. [WRITE]            Multisig account
    ///   2. []                 System program
    ///   3. []                 The rent sysvar
    CreateMultisig {
        name: String,
        owners: Vec<Pubkey>,
        threshold: u64,
    },
    /// Upgrade multisig account with a new set of owners and a threshold
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Multisig account
    UpgradeMultisig { owners: Vec<Pubkey>, threshold: u64 },
    /// Creates a new transaction account, automatically signed by the creator,
    /// which must be one of the owners of the multisig
    ///
    /// # Account references
    ///   0. [WRITE, SIGNER]    Funder account
    ///   1. [WRITE, SIGNER]    Proposer account
    ///   2. [WRITE]            Multisig account
    ///   3. [WRITE]            Transaction account
    ///   4. []                 System program
    ///   5. []                 The rent sysvar
    CreateTransaction {
        name: String,
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
    ///   1. [WRITE]            Multisig account
    ///   ...                   Set instruction accounts
    ExecuteTransaction,
}
