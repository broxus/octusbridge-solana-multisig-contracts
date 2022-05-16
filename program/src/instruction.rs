use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::TransactionAccount;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MultisigInstruction {
    /// Initializes a new multisig account with a set of owners and a threshold
    ///
    /// # Account references
    /// ...
    CreateMultisig {
        seed: u128,
        owners: Vec<Pubkey>,
        threshold: u64,
    },

    /// Upgrade multisig account with a new set of owners and a threshold
    ///
    /// # Account references
    /// ...
    UpgradeMultisig { owners: Vec<Pubkey>, threshold: u64 },

    /// Creates a new transaction account, automatically signed by the creator,
    /// which must be one of the owners of the multisig
    ///
    /// # Account references
    /// ...
    CreateTransaction {
        seed: u128,
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    },

    /// Approves a transaction on behalf of an owner of the multisig
    ///
    /// # Account references
    /// ...
    Approve,

    /// Execute transaction
    ///
    /// # Account references
    /// ...
    ExecuteTransaction,

    /// Delete pending transaction
    ///
    /// # Account references
    /// ...
    DeletePendingTransactions { pending_transactions: Vec<Pubkey> },
}
