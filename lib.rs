#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// pallet_assets runtime calls
pub mod assets;

/// Errors
pub mod errors;

#[ink::contract]
mod xaver {

    use ink::prelude::vec::Vec;
    use crate::errors::{Error, ContractError};

    /// Success Messages
    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Success {
        /// Xaver setup successful
        XaverSetupSuccess,
        /// Xaver close successful
        XaverCloseSuccess,
        /// Xaver open successful
        XaverOpenSuccess,
        /// Staking successful
        StakingSuccess,
        /// Unstaking successful
        UnstakingSuccess,
        /// Staking interest success
        StakingInterestSuccess,
        /// Income credited success        
        IncomeCreditSuccess,
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
        status: XaverTransactionStatus,
    }     

    /// Xaver staker
    #[derive(scale::Encode, scale::Decode, Clone, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Stake {
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
        /// Xaver asset, e.g., XAV
        pub asset_id: u128,
        /// Stable asset id, e.g., USDT
        pub stable_asset_id: u128,
        /// Owner
        pub owner: AccountId,
        /// Operator
        pub operator: AccountId,
        /// Price (e.g., $10, $100, $200, $1,000)
        pub price: u16,
        /// Share percentage (0.1%, 1%, 2%, 10%)
        pub share: u16,
        /// Maximum stakes of the xaver node
        pub maximum_stakes: u16,
        /// Stakers
        pub stakes: Vec<Stake>,
        /// Status (0-Open, 1-Close)
        pub status: u8,
    }

    impl Xaver {

        /// Create new xaver
        #[ink(constructor)]
        pub fn new(asset_id: u128, 
            stable_asset_id: u128,
            maximum_stakes: u16) -> Self {

            let caller: ink::primitives::AccountId = Self::env().caller();

            Self { 
                asset_id: asset_id, 
                stable_asset_id: stable_asset_id,
                owner: caller,
                operator: caller,
                price: 0u16,
                share: 0u16,
                maximum_stakes: maximum_stakes,
                stakes: Vec::new(),
                status: 0u8,
            }
        }

        /// Default setup
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(0u128, 0u128, 0u16)
        }

        /// Setup xaver
        #[ink(message)]
        pub fn setup(&mut self,
            asset_id: u128,
            stable_asset_id: u128,
            operator: AccountId,
            price: u16,
            share: u16,
            maximum_stakes: u16) -> Result<(), Error> {
            
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
            self.stable_asset_id = stable_asset_id;
            self.operator = operator;
            self.price = price;
            self.share = share;
            self.maximum_stakes = maximum_stakes;
            self.stakes =  Vec::new();
            self.status = 0;

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::XaverSetupSuccess),
            });

            Ok(())
        }

        /// Get xaver information
        #[ink(message)]
        pub fn get(&self) -> (u128, u128, AccountId, AccountId, u16, u16, u16, u8) {
            (
                self.asset_id,
                self.stable_asset_id,
                self.owner,
                self.operator,
                self.price,
                self.share,
                self.maximum_stakes,
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
            if self.env().caller() != self.operator {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the xaver node is open
            if self.status != 0 {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::XaverIsClose),
                });
                return Ok(());
            }

            // Search if the account exist already, duplicate account is not 
            // allowed.
            for stake in self.stakes.iter_mut() {
                if stake.account == account {
                    self.env().emit_event(XaverEvent {
                        operator: caller,
                        status: XaverTransactionStatus::EmitError(Error::XaverStakeAlreadyExist),
                    });
                    return Ok(());

                }
            }

            // Add to staking
            if self.stakes.len() as u16 >= self.maximum_stakes {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::XaverStakingMaxOut),
                });
                return Ok(());
            }
            let new_stake = Stake {
                account,
                accumulated_income: 0u128,
                cessation_block: self.env().block_number() as u128 + 5_256_000u128, //365 days with 6 seconds per block
                status: 1, // 1 = Liquid
            };
            self.stakes.push(new_stake);

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::StakingSuccess),
            });

            Ok(())
        }

        /// Unstake from xaver
        #[ink(message)]
        pub fn unstake(&mut self,
            account: AccountId) -> Result<(), ContractError> {

            // Unstake can only be done by the operator once the stake
            // is beyond the cessation block and is not renewed.
            let caller = self.env().caller();
            if self.env().caller() != self.operator {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the xaver is open
            if self.status != 0 {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::XaverIsClose),
                });
                return Ok(());
            }

            // Search if the stake exist already, if it does, check if the current block
            // is greater than the cessation block if ok, transfer all income then remove
            // the stake.
            let current_block = self.env().block_number() as u128;
            let mut found_index: Option<usize> = None;

            for (index, stake) in self.stakes.iter().enumerate() {
                if stake.account == account {
                    
                    // Check if cessation block has not been reached yet
                    if current_block <= stake.cessation_block {
                        self.env().emit_event(XaverEvent {
                            operator: caller,
                            status: XaverTransactionStatus::EmitError(Error::XaverStakeNotCeased),
                        });
                        return Ok(());
                    }

                    found_index = Some(index);
                    break;
                }
            }

            // Remove the stake if found, otherwise emit error
            match found_index {
                Some(index) => {
                    self.stakes.swap_remove(index);
                    self.env().emit_event(XaverEvent {
                        operator: caller,
                        status: XaverTransactionStatus::EmitSuccess(Success::UnstakingSuccess),
                    });
                }
                None => {
                    self.env().emit_event(XaverEvent {
                        operator: caller,
                        status: XaverTransactionStatus::EmitError(Error::XaverStakeNotFound),
                    });
                }
            }

            Ok(())
        }

        /// Credit to staker and increment accumulated income
        #[ink(message)]
        pub fn income(&mut self,
            amount: u128) -> Result<(), Error> {
            
            // Credit is adding to the balance of an account, this is done only
            // by the operator.
            let caller = self.env().caller();

            if self.env().caller() != self.operator {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::BadOrigin),
                });
                return Ok(());
            } 

            // Check if the xaver is open
            if self.status != 0 {
                self.env().emit_event(XaverEvent {
                    operator: caller,
                    status: XaverTransactionStatus::EmitError(Error::XaverIsClose),
                });
                return Ok(());
            }

            let income_per_staker = (amount * self.share as u128) / 100u128;
            for stake in self.stakes.iter_mut() {
                if stake.status == 1 {
                    stake.accumulated_income += income_per_staker;
                }
            }

            self.env().emit_event(XaverEvent {
                operator: caller,
                status: XaverTransactionStatus::EmitSuccess(Success::IncomeCreditSuccess),
            });

            Ok(())
        }

        /// Get staker information
        #[ink(message)]
        pub fn get_staker(&self,
            account: AccountId) ->  Option<Stake> {

            for stake in self.stakes.iter() {
                if stake.account == account {
                    return Some(stake.clone()); 
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
            let Xaver = Xaver::default();
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
            let constructor = XaverRef::default();

            // When
            let contract_account_id = client
                .instantiate("xaver", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<XaverRef>(contract_account_id.clone())
                .call(|xaver| xaver.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = XaverRef::new(false);
            let contract_account_id = client
                .instantiate("xaver", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<XaverRef>(contract_account_id.clone())
                .call(|xaver| xaver.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<XaverRef>(contract_account_id.clone())
                .call(|xaver| xaver.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<XaverRef>(contract_account_id.clone())
                .call(|xaver| xaver.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
