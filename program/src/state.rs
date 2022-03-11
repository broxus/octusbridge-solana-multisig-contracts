use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

use multisig_derive::MultisigPack;

/// Maximum number of pending transactions
pub const MAX_TRANSACTIONS: usize = 10;

#[derive(Debug, BorshSerialize, BorshDeserialize, MultisigPack)]
#[multisig_pack(length = 500)]
pub struct Multisig {
    pub is_initialized: bool,
    // Set of custodians
    pub owners: Vec<Pubkey>,
    // Required number of signers
    pub threshold: u64,
    // Set of pending transactions
    pub pending_transactions: Vec<Pubkey>,
}

impl Sealed for Multisig {}

impl IsInitialized for Multisig {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize, MultisigPack)]
#[multisig_pack(length = 5000)]
pub struct Transaction {
    pub is_initialized: bool,
    // The multisig account this transaction belongs to.
    pub multisig: Pubkey,
    // Target program to execute against.
    pub program_id: Pubkey,
    // Accounts required for the transaction.
    pub accounts: Vec<TransactionAccount>,
    // Instruction data for the transaction.
    pub data: Vec<u8>,
    // signers[index] is true if multisig.owners[index] signed the transaction.
    pub signers: Vec<bool>,
    // Boolean ensuring one time execution.
    pub did_execute: bool,
}

impl Sealed for Transaction {}

impl IsInitialized for Transaction {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl From<&Transaction> for Instruction {
    fn from(tx: &Transaction) -> Instruction {
        Instruction {
            program_id: tx.program_id,
            accounts: tx.accounts.iter().map(Into::into).collect(),
            data: tx.data.clone(),
        }
    }
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct TransactionAccount {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&TransactionAccount> for AccountMeta {
    fn from(account: &TransactionAccount) -> AccountMeta {
        match account.is_writable {
            false => AccountMeta::new_readonly(account.pubkey, account.is_signer),
            true => AccountMeta::new(account.pubkey, account.is_signer),
        }
    }
}