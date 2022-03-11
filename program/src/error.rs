use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum MultisigError {
    #[error("The given owner is not part of this multisig.")]
    InvalidOwner,
    #[error("Owners length must be non zero.")]
    InvalidOwnersLen,
    #[error("Not enough owners signed this transaction.")]
    NotEnoughSigners,
    #[error("Cannot delete a transaction that has been signed by an owner.")]
    TransactionAlreadySigned,
    #[error("Overflow when adding.")]
    Overflow,
    #[error("Cannot delete a transaction the owner did not create.")]
    UnableToDelete,
    #[error("The given transaction has already been executed.")]
    AlreadyExecuted,
    #[error("Threshold must be less than or equal to the number of owners.")]
    InvalidThreshold,
    #[error("Owners must be unique")]
    UniqueOwners,
    #[error("Pending transaction limit exceeded")]
    PendingTransactionLimit,
    #[error("Invalid transaction")]
    InvalidTransaction,
}

impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
