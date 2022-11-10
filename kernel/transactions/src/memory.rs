/* Define operations over kernel memory - persisted in RAM between yields */

use host::path::RefPath;
use host::rollup_core::RawRollupCore;
use serde::{ Deserialize, Serialize };

use alloc::collections::BTreeMap;
use crypto::hash::Layer2Tz4Hash;

/* need load_memory to use in lib.rs */

const MEMORY_PATH: RefPath = RefPath::assert_from(b"/tx/memory/");

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
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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