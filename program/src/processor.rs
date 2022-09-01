use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, system_instruction};

use crate::{
    require, Multisig, MultisigError, MultisigInstruction, Transaction, TransactionAccount,
    MAX_SIGNERS, MAX_TRANSACTIONS, MIN_SIGNERS,
};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = MultisigInstruction::try_from_slice(instruction_data)?;

        match instruction {
            MultisigInstruction::CreateMultisig {
                seed,
                owners,
                threshold,
            } => {
                msg!("Instruction: Create Multisig");
                Self::process_create_multisig(program_id, accounts, seed, owners, threshold)?;
            }
            MultisigInstruction::CreateTransaction {
                seed,
                pid,
                accs,
                data,
            } => {
                msg!("Instruction: Create Transaction");
                Self::process_create_transaction(program_id, accounts, seed, pid, accs, data)?;
            }
            MultisigInstruction::Approve => {
                msg!("Instruction: Approve");
                Self::process_approve(program_id, accounts)?;
            }
            MultisigInstruction::ExecuteTransaction => {
                msg!("Instruction: Execute Transaction");
                Self::process_execute_transaction(program_id, accounts)?;
            }
            MultisigInstruction::AddOwner { owner } => {
                msg!("Instruction: Add Owner");
                Self::process_add_owner(program_id, accounts, owner)?;
            }
            MultisigInstruction::DeleteOwner { owner } => {
                msg!("Instruction: Delete Owner");
                Self::process_delete_owner(program_id, accounts, owner)?;
            }
            MultisigInstruction::UpdateThreshold { threshold } => {
                msg!("Instruction: Update Threshold");
                Self::process_update_threshold(program_id, accounts, threshold)?;
            }
            MultisigInstruction::DeletePendingTransaction {
                pending_transaction,
            } => {
                msg!("Instruction: Delete Pending Transactions");
                Self::process_delete_pending_transaction(
                    program_id,
                    accounts,
                    pending_transaction,
                )?;
            }
        };

        Ok(())
    }

    fn process_create_multisig(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seed: u128,
        owners: Vec<Pubkey>,
        threshold: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let funder_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_info)?;

        let (multisig_account, multisig_nonce) =
            Pubkey::find_program_address(&[br"multisig", &seed.to_le_bytes()], program_id);

        if multisig_account != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let multisig_account_signer_seeds: &[&[_]] =
            &[br"multisig", &seed.to_le_bytes(), &[multisig_nonce]];

        assert_unique_owners(&owners)?;

        require!(
            threshold <= owners.len() as u64
                && threshold <= MAX_SIGNERS as u64
                && threshold >= MIN_SIGNERS as u64,
            MultisigError::InvalidThreshold
        );

        require!(!owners.is_empty(), MultisigError::InvalidOwnersLen);

        invoke_signed(
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
            &[multisig_account_signer_seeds],
        )?;

        let multisig = Multisig {
            is_initialized: true,
            owners,
            threshold,
            pending_transactions: vec![],
            seed,
        };

        Multisig::pack(multisig, &mut multisig_account_info.data.borrow_mut())?;

        Ok(())
    }

    fn process_add_owner(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;

        if !multisig_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let (multisig_account, _nonce) = Pubkey::find_program_address(
            &[br"multisig", &multisig_account_data.seed.to_le_bytes()],
            program_id,
        );

        if multisig_account != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        require!(
            multisig_account_data.owners.len() + 1 <= MAX_SIGNERS,
            MultisigError::OwnersOverflow
        );

        if let None = multisig_account_data
            .owners
            .iter()
            .position(|a| *a == owner)
        {
            multisig_account_data.owners.push(owner);
        }

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_delete_owner(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        owner: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;

        if !multisig_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let (multisig_account, _nonce) = Pubkey::find_program_address(
            &[br"multisig", &multisig_account_data.seed.to_le_bytes()],
            program_id,
        );

        if multisig_account != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        require!(
            multisig_account_data.owners.len() - 1 >= MIN_SIGNERS
                && multisig_account_data.owners.len() - 1
                    >= multisig_account_data.threshold as usize,
            MultisigError::OwnersLackOff
        );

        multisig_account_data.owners.retain(|x| *x != owner);

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_update_threshold(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        threshold: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;

        if !multisig_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let (multisig_account, _nonce) = Pubkey::find_program_address(
            &[br"multisig", &multisig_account_data.seed.to_le_bytes()],
            program_id,
        );

        if multisig_account != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        require!(
            multisig_account_data.owners.len() >= threshold as usize,
            MultisigError::InvalidThreshold
        );

        multisig_account_data.threshold = threshold;

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_create_transaction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seed: u128,
        pid: Pubkey,
        accs: Vec<TransactionAccount>,
        data: Vec<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let funder_account_info = next_account_info(account_info_iter)?;
        let proposer_account_info = next_account_info(account_info_iter)?;
        let multisig_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;
        let system_program_info = next_account_info(account_info_iter)?;
        let rent_sysvar_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_sysvar_info)?;

        if !proposer_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        require!(
            multisig_account_data.pending_transactions.len() <= MAX_TRANSACTIONS,
            MultisigError::InvalidThreshold
        );

        if multisig_account_data.pending_transactions.len() + 1 == MAX_TRANSACTIONS {
            if *program_id != pid {
                return Err(MultisigError::InvalidLastTransaction.into());
            }

            let instruction = MultisigInstruction::try_from_slice(&data)?;
            if !matches!(
                instruction,
                MultisigInstruction::DeletePendingTransaction {
                    pending_transaction: _
                }
            ) {
                return Err(MultisigError::InvalidLastTransaction.into());
            }
        }

        let owner_index = multisig_account_data
            .owners
            .iter()
            .position(|a| a == proposer_account_info.key)
            .ok_or(MultisigError::InvalidOwner)?;

        let (transaction_account, transaction_nonce) =
            Pubkey::find_program_address(&[br"transaction", &seed.to_le_bytes()], program_id);

        if transaction_account != *transaction_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let transaction_account_signer_seeds: &[&[_]] =
            &[br"transaction", &seed.to_le_bytes(), &[transaction_nonce]];

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

        let data_len = tx.try_to_vec()?.len();

        invoke_signed(
            &system_instruction::create_account(
                funder_account_info.key,
                transaction_account_info.key,
                1.max(rent.minimum_balance(data_len)),
                data_len as u64,
                program_id,
            ),
            &[
                funder_account_info.clone(),
                transaction_account_info.clone(),
                system_program_info.clone(),
            ],
            &[transaction_account_signer_seeds],
        )?;

        tx.pack_into_slice(&mut transaction_account_info.data.borrow_mut());

        // Add transaction to pending list
        multisig_account_data
            .pending_transactions
            .push(*transaction_account_info.key);

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

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
            Transaction::unpack_from_slice(&transaction_account_info.data.borrow())?;

        if transaction_account_data.multisig != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let owner_index = multisig_account_data
            .owners
            .iter()
            .position(|value| value == proposer_account_info.key)
            .ok_or(MultisigError::InvalidOwner)?;

        transaction_account_data.signers[owner_index] = true;

        transaction_account_data.pack_into_slice(&mut transaction_account_info.data.borrow_mut());

        Ok(())
    }

    fn process_execute_transaction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;
        let transaction_account_info = next_account_info(account_info_iter)?;

        let multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let (_account, multisig_nonce) = Pubkey::find_program_address(
            &[br"multisig", &multisig_account_data.seed.to_le_bytes()],
            program_id,
        );

        let multisig_account_seeds: &[&[_]] = &[
            br"multisig",
            &multisig_account_data.seed.to_le_bytes(),
            &[multisig_nonce],
        ];

        let mut transaction_account_data =
            Transaction::unpack_from_slice(&transaction_account_info.data.borrow())?;

        if transaction_account_data.multisig != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

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
                if &acc.pubkey == multisig_account_info.key {
                    acc.is_signer = true;
                }
                acc
            })
            .collect();

        let accounts = account_info_iter.cloned().collect::<Vec<_>>();

        invoke_signed(&ix, &accounts, &[multisig_account_seeds])?;

        // Burn the transaction to ensure one time use.
        transaction_account_data.did_execute = true;

        transaction_account_data.pack_into_slice(&mut transaction_account_info.data.borrow_mut());

        // Remove transaction from pending list
        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let transaction_index = multisig_account_data
            .pending_transactions
            .iter()
            .position(|value| value == transaction_account_info.key)
            .ok_or(MultisigError::InvalidTransaction)?;

        multisig_account_data
            .pending_transactions
            .remove(transaction_index);

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
        )?;

        Ok(())
    }

    fn process_delete_pending_transaction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        pending_transaction: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let multisig_account_info = next_account_info(account_info_iter)?;

        if !multisig_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut multisig_account_data = Multisig::unpack(&multisig_account_info.data.borrow())?;

        let (multisig_account, _nonce) = Pubkey::find_program_address(
            &[br"multisig", &multisig_account_data.seed.to_le_bytes()],
            program_id,
        );

        if multisig_account != *multisig_account_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        multisig_account_data
            .pending_transactions
            .retain(|x| *x != pending_transaction);

        Multisig::pack(
            multisig_account_data,
            &mut multisig_account_info.data.borrow_mut(),
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
