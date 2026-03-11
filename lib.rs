#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// pallet_assets runtime calls
pub mod assets;

/// Errors
pub mod errors;

#[ink::contract]
mod xaver {

    use ink::prelude::vec::Vec;

    use crate::errors::{Error, RuntimeError, ContractError};
    use crate::assets::{AssetsCall, RuntimeCall};

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Xaver setup successful
        XaverSetupSuccess,
        /// Bank close successful
        XaverCloseSuccess,
        /// Bank open successful
        XaverOpenSuccess,
        /// Staking successful
        StakingSuccess,
        /// Unstaking successful
        UnstakingSuccess,
        /// Staking interest success
        StakingInterestSuccess,        
    }    

    /// Xaver transaction status
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum XaverTransactionStatus {
        EmitSuccess(Success),
        EmitError(Error),
    }    

    /// Xaver events
    #[ink(event)]
    pub struct XaverEvent {
        #[ink(topic)]
        operator: AccountId,
        status: BankTransactionStatus,
    }     

    /// Xaver staker
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Staker {
        /// Account address
        pub account: AccountId,
        /// Accumulated income
        pub accumulated_income: u128,
        /// Cessation block
        pub cessation_block: u128,
        /// Status (0-Frozen, 1-Liquid)
        pub status: u8,
    }        

    /// Xaver storage
    #[ink(storage)]
    pub struct Xaver {
        /// Xaver asset
        pub asset_id: u128,
        /// Owner
        pub owner: AccountId,
        /// Operator
        pub operator: AccountId,
        /// Price (e.g., $10, $100, $200, $1,000)
        pub price: u16,
        /// Share percentage (0.1%, 1%, 2%, 10%)
        pub share: u16,
        /// Maximum stakers of the xaver node
        pub maximum_stakers: u16,
        /// Stakers
        pub stakers: Vec<Staker>,
        /// Status (0-Open, 1-Close)
        pub status: u8,
    }

    impl Xaver {

        /// Create new bank
        #[ink(constructor)]
        pub fn new(asset_id: u128, 
            maximum_stakers: u16) -> Self {

            let caller: ink::primitives::AccountId = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                owner: caller,
                operator: caller,
                price: 0u16,
                share: 0u16,
                maximum_stakers: maximum_stakers,
                stakers: Vec::new(),
                status: 0u8,
            }
        }

        /// Default setup
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128, 0u16)
        }

        /// Setup xaver
        #[ink(message)]
        pub fn setup(&mut self,
            asset_id: u128,
            operator: AccountId,
            price: u16,
            share: u16,
            maximum_stakers: u16) -> Result<(), Error> {
            
            // Setup can only be done by the owner
            let caller = self.env().caller();
            if self.env().caller() != self.owner {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // The setup will delete all existing stakers - Very Important!
            self.asset_id = asset_id;
            self.operator = operator;
            self.price = price;
            self.share = share;
            self.maximum_stakers = maximum_stakers;
            self.stakers =  Vec::new();
            self.status = 0;

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::XaverSetupSuccess),
            });

            Ok(())
        }

        /// Get xaver information
        #[ink(message)]
        pub fn get(&self) -> (u128, AccountId, AccountId, u16, u16, u16, u8) {
            (
                self.asset_id,
                self.owner,
                self.operator,
                self.price,
                self.share,
                self.maximum_stakers,
                self.status,
            )
        }

        /// Close xaver
        #[ink(message)]
        pub fn close(&mut self) -> Result<(), Error> {

            // Closing the can only be done by the operator
            let caller = self.env().caller();
            if self.env().caller() != self.operator {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will close xaver staking
            self.status = 1;

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::XaverCloseSuccess),
            });

            Ok(())
        }

        /// Open xaver
        #[ink(message)]
        pub fn open(&mut self) -> Result<(), Error> {

            // Opening the can only be done by the operator
            let caller = self.env().caller();
            if self.env().caller() != self.operator {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // This will open the xaver
            self.status = 0;

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::XaverOpenSuccess),
            });

            Ok(())
        }        

        /// Stake to xaver
        #[ink(message)]
        pub fn stake(&mut self,
            account: AccountId) -> Result<(), Error> {

            // Staking can only be done by the operator once the transfer of the 
            // asset is verified through the tx-hash.
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search if the account exist already, if it does in just add to the
            // ledger the amount deposited, if not then create the new account.
            // 1. Update a balance
            let mut account_found = false;
            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    
                    ledger.balance = ledger
                        .balance
                        .checked_add(amount)
                        .ok_or(Error::AccountBalanceOverflow)?; 

                    account_found = true;
                    break;
                }
            }
            // 2. Create a new account if the account does not exist
            if !account_found {
                if self.ledgers.len() as u16 >= self.maximum_accounts {
                    self.env().emit_event(BankingEvent {
                        operator: caller,
                        status: BankTransactionStatus::EmitError(Error::BankAccountMaxOut),
                    });
                    return Ok(());
                }
                let new_ledger = Ledger {
                    account,
                    balance: amount,
                    status: 1, // 1 = Liquid
                };
                self.ledgers.push(new_ledger);
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountDepositSuccess),
            });

            Ok(())
        }

        /// Unstake from xaver
        #[ink(message)]
        pub fn unstake(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), ContractError> {

            // Withdraw can only be done by the manager once the balance of the account
            // is sufficient for withdrawal
            let caller = self.env().caller();
            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search if the account exist already, if it does, check if the balance is
            // sufficient, if so, deduct the ledger, if not raise a balance insufficient
            // error.
            let mut account_found = false;
            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    account_found = true;

                    // Check if balance is sufficient
                    if ledger.balance < amount {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountBalanceInsufficient),
                        });
                        return Ok(());
                    }

                    // Deduct the amount
                    ledger.balance -= amount;

                    // Transfer the asset to the account
                    self.env()
                        .call_runtime(&RuntimeCall::Assets(AssetsCall::Transfer {
                            id: self.asset_id,
                            target: account.into(),
                            amount: amount,
                        }))
                        .map_err(|_| RuntimeError::CallRuntimeFailed)?;

                    break;
                }
            }

            if !account_found {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                });
                return Ok(());
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountWithdrawalSuccess),
            });

            Ok(())
        }

        /// Credit to staker and increment accumulated income
        #[ink(message)]
        pub fn income(&mut self,
            account: AccountId,
            amount: u128) -> Result<(), Error> {
            
            // Credit is adding to the balance of an account, this is done only
            // by the manager.
            let caller = self.env().caller();

            if self.env().caller() != self.manager {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the bank is open
            if self.status != 0 {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::BankIsClose),
                });
                return Ok(());
            }

            // Search for the caller account in the ledger, if found, add to the balance
            // the given amount.
            let mut account_found = false;

            for ledger in self.ledgers.iter_mut() {
                if ledger.account == account {
                    account_found = true;

                    // Check if account is liquid
                    if ledger.status != 1 {
                        self.env().emit_event(BankingEvent {
                            operator: caller,
                            status: BankTransactionStatus::EmitError(Error::AccountFrozen),
                        });
                        return Ok(());
                    }

                    // Add the amount to the balance safely
                    match ledger.balance.checked_add(amount) {
                        Some(new_balance) => ledger.balance = new_balance,
                        None => {
                            self.env().emit_event(BankingEvent {
                                operator: caller,
                                status: BankTransactionStatus::EmitError(Error::AccountBalanceOverflow),
                            });
                            return Ok(());
                        }
                    }

                    break;
                }
            }

            if !account_found {
                self.env().emit_event(BankingEvent {
                    operator: caller,
                    status: BankTransactionStatus::EmitError(Error::AccountNotFound),
                });
                return Ok(());
            }

            self.env().emit_event(BankingEvent {
                operator: caller,
                status: BankTransactionStatus::EmitSuccess(Success::AccountCreditSuccess),
            });

            Ok(())
        }

        /// Get staker information
        #[ink(message)]
        pub fn get_staker(&self,
            account: AccountId) ->  Option<Ledger> {

            for ledger in self.ledgers.iter() {
                if ledger.account == account {
                    return Some(ledger.clone()); 
                }
            }

            None
        }

    }

    /// Unit tests
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let Bank = Bank::default();
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = BankRef::default();

            // When
            let contract_account_id = client
                .instantiate("bank", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = BankRef::new(false);
            let contract_account_id = client
                .instantiate("bank", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<BankRef>(contract_account_id.clone())
                .call(|bank| bank.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
