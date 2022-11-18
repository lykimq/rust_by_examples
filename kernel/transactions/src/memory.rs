/* Define operations over kernel memory - persisted in RAM between yields */

use host::path::RefPath;
use host::rollup_core::RawRollupCore;
use alloc::collections::BTreeMap;
use crypto::hash::Layer2Tz4Hash;
use crate::{ encoding::{ string_ticket::{ StringTicketHash } } };

use thiserror::Error;

/* need load_memory to use in lib.rs */

const MEMORY_PATH: RefPath = RefPath::assert_from(b"/tx/memory/");

#[derive(Default, Debug)]
/* Memory contents: ticket defintions and the account balance sheet */
pub struct Memory {
    // add only account
    accounts: Accounts,
}

impl Memory {
    // Load memory from the durable store.
    pub fn load_memory<Host: RawRollupCore>(host: &Host) -> Self {
        host::runtime
            ::load_value_sized(host, &MEMORY_PATH)
            .map(|mem| {
                serde_json::from_slice(mem.as_slice()).expect("Could not deserialize memory")
            })
            .unwrap_or_default()
    }

    // deal with accounts
    pub fn accounts(&self) -> &Accounts {
        &self.accounts
    }
}

// Accounts balance sheet
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Accounts(BTreeMap<Layer2Tz4Hash, Account>);

impl Accounts {
    // Get a mutable reference to account
    pub fn account_of_mut(&mut self, address: &Layer2Tz4Hash) -> Option<&mut Account> {
        self.0.get_mut(address)
    }

    // Add a new account at address
    pub fn add_account(
        &mut self,
        address: Layer2Tz4Hash,
        account: Account
    ) -> Result<(), AccountError> {
        if self.0.contains_key(&address) {
            return Err(AccountError::AddressOccupied(address));
        }
        self.0.insert(address, account);
        Ok(())
    }
}

// Define AccountError
#[derive(Error, Debug)]
pub enum AccountError {
    // Address already taken by previous account
    #[error("Could not add new account due to previous account at address {0}")] AddressOccupied(
        Layer2Tz4Hash,
    ),
}

/* Account only content counter */
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Account {
    balance: BTreeMap<StringTicketHash, u64>,
    counter: i64,
}

impl Account {
    // Increments the operation counter of the account
    pub fn increment_counter(&mut self) {
        self.counter += 1;
    }

    // The current value of the account's operation counter
    pub fn counter(&self) -> i64 {
        self.counter
    }

    // Add ticket

    pub fn add_ticket(&mut self, hash: StringTicketHash, amount: u64) -> Result<(), AccountError> {
        if let Some(ticket_balance) = self.balance.get_mut(&hash) {
            *ticket_balance = ticket_balance
                .checked_add(amount)
                .ok_or(AccountError::BalanceOverflow(*ticket_balance, amount))?;
        } else {
            self.balance.insert(hash, amount);
        }
        Ok(())
    }
}