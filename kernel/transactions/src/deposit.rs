use crypto::hash::Layer2Tz4Hash;
use host::rollup_core::RawRollupCore;
use thiserror::Error;
use debug::debug_msg;

use crate::{
    encoding::string_ticket::{ StringTicket, TicketHashError },
    memory::{ Account, AccountError, Memory },
};

// Deposit tickets into the kernel state

/// Errors that may occur when depositing a ticket into an account.
#[derive(Error, Debug)]
pub enum DepositError {
    /// Issue occurred while handling depositee account.
    #[error("{0}")]
    AccountError(#[from] AccountError),

    /// Issue occurred hashing ticket.
    #[error("Error hashing ticket contents: {0}")]
    TicketHash(#[from] TicketHashError),
}

pub fn deposit_ticket<Host: RawRollupCore>(
    memory: &mut Memory,
    account_address: Layer2Tz4Hash,
    ticket: StringTicket
) -> Result<(), DepositError> {
    let ticket_amount = ticket.amount();
    let id_proof = ticket.identify_trustless()?;

    debug_msg!(
        Host,
        "Depositing {:#?} with identity {:?} into account {:?}",
        id_proof.ticket(),
        id_proof.identify(),
        &account_address
    );

    let result: Result<Option<Account>, AccountError> = memory
        .accounts_mut()
        .account_of_mut(&account_address)
        .map_or_else(
            || {
                let mut account = Account::default();
                account.add_ticket(id_proof.identify().clone(), ticket_amount)?;
                Ok(Some(account))
            },
            |account| {
                account.add_ticket(id_proof.identify().clone(), ticket_amount)?;
                Ok(None)
            }
        );

    match result? {
        Some(new_account) => memory.accounts_mut().add_account(account_address, new_account)?,
        None => (),
    }

    // update global ticket table
    memory.add_ticket(id_proof);
    Ok(())
}