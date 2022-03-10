use borsh::BorshDeserialize;

use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program::{invoke, invoke_signed};
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

use crate::{
    require, Multisig, MultisigError, MultisigInstruction, Transaction, TransactionAccount,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = MultisigInstruction::try_from_slice(instruction_data).unwrap();

        match instruction {
            MultisigInstruction::CreateMultisig { owners, threshold } => {
                msg!("Instruction: Create Multisig");
                Self::process_create_multisig(program_id, accounts, owners, threshold)?;
            }
            MultisigInstruction::CreateTransaction { pid, accs, data } => {
                msg!("Instruction: Create Transaction");
                Self::process_create_transaction(program_id, accounts, pid, accs, data)?;
            }
            MultisigInstruction::Approve => {
                msg!("Instruction: Approve");
                Self::process_approve(program_id, accounts)?;
            }
            MultisigInstruction::ExecuteTransaction => {
                msg!("Instruction: Execute Transaction");
                Self::process_execute_transaction(program_id, accounts)?;
            }
        };

        Ok(())
    }

    fn process_create_multisig(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owners: Vec<Pubkey>,
        threshold: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let funder_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_info)?;

        if !multisig_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        assert_unique_owners(&owners)?;

        require!(
            threshold > 0 && threshold <= owners.len() as u64,
            MultisigError::InvalidThreshold
        );

        require!(!owners.is_empty(), MultisigError::InvalidOwnersLen);

        invoke(
            &system_instruction::create_account(
                funder_account_info.key,
                multisig_account_info.key,
                1.max(rent.minimum_balance(Multisig::LEN)),
                Multisig::LEN as u64,
                program_id,
            ),
            &[
                funder_account_info.clone(),
                multisig_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;

        let multisig = Multisig {
            is_initialized: true,
            owners,
            threshold,
        };

        Multisig::pack(multisig, &mut multisig_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_create_transaction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let funder_account_info = next_account_info(account_info_iter)?;
        let proposer_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_info)?;

        if !proposer_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let owner_index = multisig_account_data
            .owners
            .iter()
            .position(|a| a == proposer_account_info.key)
            .ok_or(MultisigError::InvalidOwner)?;

        invoke(
            &system_instruction::create_account(
                funder_account_info.key,
                transaction_account_info.key,
                1.max(rent.minimum_balance(Transaction::LEN)),
                Transaction::LEN as u64,
                program_id,
            ),
            &[
                funder_account_info.clone(),
                transaction_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;

        let mut signers = Vec::new();
        signers.resize(multisig_account_data.owners.len(), false);
        signers[owner_index] = true;

        let tx = Transaction {
            is_initialized: true,
            multisig: *multisig_account_info.key,
            program_id: pid,
            accounts: accs,
            did_execute: false,
            data,
            signers,
        };

        Transaction::pack(tx, &mut transaction_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_approve(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let proposer_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;

        if !proposer_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;
        let mut transaction_account_data =
            Transaction::unpack(&transaction_account_info.data.borrow())?;

        if transaction_account_data.multisig != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let owner_index = multisig_account_data
            .owners
            .iter()
            .position(|a| a == proposer_account_info.key)
            .ok_or(MultisigError::InvalidOwner)?;

        transaction_account_data.signers[owner_index] = true;

        Transaction::pack(
            transaction_account_data,
            &mut transaction_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_execute_transaction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let multisig_signer_account_info = next_account_info(account_info_iter)?;

        let multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;
        let mut transaction_account_data =
            Transaction::unpack(&transaction_account_info.data.borrow())?;

        if transaction_account_data.multisig != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let (multisig_signer_account, multisig_signer_nonce) =
            Pubkey::find_program_address(&[&multisig_account_info.key.to_bytes()], program_id);

        if multisig_signer_account != *multisig_signer_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let multisig_account_signer_seeds: &[&[_]] = &[
            &multisig_account_info.key.to_bytes(),
            &[multisig_signer_nonce],
        ];

        // Has this been executed already?
        if transaction_account_data.did_execute {
            return Err(MultisigError::AlreadyExecuted.into());
        }

        // Do we have enough signers.
        let sig_count = transaction_account_data
            .signers
            .iter()
            .filter(|&did_sign| *did_sign)
            .count() as u64;
        if sig_count < multisig_account_data.threshold {
            return Err(MultisigError::NotEnoughSigners.into());
        }

        // Execute the transaction signed by the multisig.
        let mut ix: Instruction = (&transaction_account_data).into();
        ix.accounts = ix
            .accounts
            .iter()
            .map(|acc| {
                let mut acc = acc.clone();
                if &acc.pubkey == multisig_signer_account_info.key {
                    acc.is_signer = true;
                }
                acc
            })
            .collect();

        let accounts = account_info_iter.map(|x| x.clone()).collect::<Vec<_>>();

        invoke_signed(&ix, &accounts, &[multisig_account_signer_seeds])?;

        // Burn the transaction to ensure one time use.
        transaction_account_data.did_execute = true;

        Transaction::pack(
            transaction_account_data,
            &mut transaction_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }
}

fn assert_unique_owners(owners: &[Pubkey]) -> Result<(), ProgramError> {
    for (i, owner) in owners.iter().enumerate() {
        require!(
            !owners.iter().skip(i + 1).any(|item| item == owner),
            MultisigError::UniqueOwners
        )
    }
    Ok(())
}
