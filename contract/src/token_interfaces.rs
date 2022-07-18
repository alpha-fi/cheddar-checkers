use std::collections::HashMap;
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, AccountId, Balance, PanicOnDefault, PromiseOrValue};
use near_sdk::{env, log, Promise, PromiseResult, Gas};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::*;

#[allow(dead_code)]
pub const NO_DEPOSIT:u128 = 0;
const STORAGE_PRICE_PER_BYTE: u128 = 10000000000000000000;
const STORAGE_DEPOSIT: u128 = 125 * STORAGE_PRICE_PER_BYTE;
pub const CALLBACK_GAS:Gas = 5_000_000_000_000;
pub const ONE_YOCTO: Balance = 1;
//pub const GAS_FOR_FT_TRANSFER:Gas = 30_000_000_000_000;


#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_balance_of(&mut self, account_id: AccountId) -> U128;
    fn storage_deposit(&self, account_id: AccountId);
    fn storage_balance_of(&self, account_id: AccountId) -> AccountStorageBalance;
    fn ft_transfer(&mut self, receiver_id: String, amount: String);
    fn ft_transfer_call(&mut self, receiver_id: String, amount: String, msg: String);
    fn ft_metadata(&self) -> FungibleTokenMetadata;
}

#[ext_contract(ext_self)]
pub trait TokenInterfaces {
    fn on_ft_balance_of(&mut self, account_id: AccountId) -> Balance;
    fn ft_on_transfer(&mut self, sender_id: ValidAccountId, amount: U128, msg: String,) -> PromiseOrValue<U128>;
    fn on_ft_metadata(&mut self, token_id: AccountId);
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FungibleTokenBalances {
    balance: HashMap<AccountId, Balance>,
    token_id: AccountId
}

#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountStorageBalance {
    total: U128,
    available: U128
}


#[derive(Deserialize, Serialize, BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct WhitelistedToken {
    pub metadata: FungibleTokenMetadata,
    pub balances: FungibleTokenBalances
}

enum TransferInstruction {
    DepositWithRefferer,
    Deposit,
}

//configure deposit actions via msg
/*
if msg in ft_transfer call is empty - its default deposit with no refferer
if msg sended in format : 'account.testnet' - it inserts in make_available_ft function to add some refferal to available player
*/
impl From<String> for TransferInstruction {
    fn from(item: String) -> Self {
        match &item[..] {
            "" => TransferInstruction::Deposit,
            _ => TransferInstruction::DepositWithRefferer,
        }
    }
}


impl FungibleTokenBalances {

    pub fn new(token_id: AccountId) -> FungibleTokenBalances {
        FungibleTokenBalances {
            token_id,
            balance: HashMap::default()
        }
    }
    //balance check section
    /*
    For now, this section is unused. But it can used for check balance of fungible tokens and also
    for checking is account have registered storage in fungible token contract. Because if account storage
    not registered, this account cannot get transfer from other accounts. But in this game users first of all
    deposit some money from their balances, which allows us don't check their storage or balances.
    */
    #[allow(unused)]
    pub fn check_storage_deposit(&self, account_id: AccountId, token_id: AccountId) {
        ext_ft::storage_balance_of(
            account_id.clone(), 
            &token_id, 
            NO_DEPOSIT, 
            CALLBACK_GAS
        );
        assert_eq!(
            env::promise_results_count(),
            1,
            "this is callback method!"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("failed promise!"),
            PromiseResult::Successful(result) => {
                let storage_balance = near_sdk::serde_json::from_slice::<AccountStorageBalance>(&result).unwrap();
                if storage_balance.available.0.to_string() == "" {
                    ext_ft::storage_deposit(
                        account_id, 
                        &token_id,
                        STORAGE_DEPOSIT,
                        CALLBACK_GAS
                    );
                }
            }
        }
    }
    #[allow(unused)]
    pub fn get_balance(&mut self, account_id: AccountId, token_id: AccountId) -> Promise {
        ext_ft::ft_balance_of(
            account_id.clone(),
            &token_id, // contract account id
            NO_DEPOSIT, // yocto FT to attach
            CALLBACK_GAS // gas to attach
        )
        .then(ext_self::on_ft_balance_of(
            account_id,
            &env::current_account_id(), // this contract's account id
            NO_DEPOSIT, // yocto FT to attach to the callback
            CALLBACK_GAS // gas to attach to the callback
        ))
    }
    #[allow(unused)]
    pub fn on_ft_balance_of(&mut self, account_id: AccountId) -> Balance {
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("failed promise!"),
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                self.balance.insert(account_id.clone(), balance.0);
                balance.0
            }
        }
    }
}

#[near_bindgen]
impl Checkers {
    
    #[private]
    pub fn whitelist_token(
        &mut self,
        token_id: AccountId,
    ) -> Promise {
        assert_eq!(env::predecessor_account_id(), env::current_account_id(), "owner method");

            //storage deposit for our contract for have ability to receive deposits in this token
            ext_ft::storage_deposit (
                env::predecessor_account_id(),
                &token_id,
                STORAGE_DEPOSIT,
                CALLBACK_GAS
            );

            ext_ft::ft_metadata(
                &token_id,
                NO_DEPOSIT,
                CALLBACK_GAS,
            ).then(ext_self::on_ft_metadata(
                token_id.into(),
                &env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK_GAS,
            ))
    }

    pub fn on_ft_metadata(
        &mut self,
        token_id: AccountId)
        {

        
        assert_eq!(
            env::promise_results_count(),
            1,
            "Contract expected a result on the callback"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => panic!("failed promise!"),
            PromiseResult::Successful(result) => {
                let ft_metadata = near_sdk::serde_json::from_slice::<FungibleTokenMetadata>(&result).unwrap();
                self.whitelisted_tokens.insert(
                    &(token_id.clone()),
                    &WhitelistedToken {
                        metadata: ft_metadata,
                        balances: FungibleTokenBalances::new(token_id.clone().into())
                    },
                );
            }
        }
    }

    pub fn get_token_decimals(&self, token_id: String) -> u8 {
        let ft_whitelisted_token = self.whitelisted_tokens
            .get(&token_id)
            .expect("token isn't whitelisted");
        ft_whitelisted_token.metadata.decimals
    }
    pub fn get_token_ticker(&self, token_id: String) -> String {
        let ft_whitelisted_token = self.whitelisted_tokens
            .get(&token_id)
            .expect("token isn't whitelisted");
        ft_whitelisted_token.metadata.symbol
    }
    /*
        player calls ft_transfer_call in token_id contract for transfer amount of tokens to Checkers contract:
        PLAYER -> Checkers_contract 
        in case of success transfer it calls deposit function in Checkers_contract with amount
        deposit(amount, token_id)
        this function update player balance in app
    */
    

    pub fn ft_on_transfer(
        &mut self,
        //token_id: AccountId,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {

        //token contract which calls this function
        let contract_id = env::predecessor_account_id();
        let transfer_message: String = msg.clone();
        let sender: AccountId = sender_id.into();
        let ticker = self.get_token_ticker(contract_id.clone());
        let min_deposit = min_deposit(self.get_token_decimals(contract_id.clone()));
        
        assert!(
            amount.0 >= min_deposit,
            "Deposit in {} token is too small. Attached: {}, Required: {}", 
            ticker,
            amount.0,
            min_deposit
        );

        match TransferInstruction::from(transfer_message) {
            TransferInstruction::Deposit => {
                let referrer_id: Option<AccountId> = None;

                log!("in deposit from @{} with token: ${} amount {:?} ", sender, ticker, amount);
                let available_complete = self.make_available_ft(sender, amount, referrer_id);
                if available_complete {
                    PromiseOrValue::Value(U128(0))
                } else {
                    PromiseOrValue::Value(amount)
                }
            },
            TransferInstruction::DepositWithRefferer => {
                let referrer_id: AccountId = msg.into();
                assert!(
                    env::is_valid_account_id(referrer_id.as_bytes()),
                    "Refferal account @{} is invalid. If you wanna just make a deposit with no refferer don't attach any msg to ft_transfer_call!",
                    referrer_id.clone()
                );

                log!("in deposit from @{} with token: ${} amount {:?} with refferer: @{} ",
                    sender, 
                    ticker, 
                    amount,
                    referrer_id
                );

                let available_complete =self.make_available_ft(sender, amount, Some(referrer_id));
                if available_complete {
                    PromiseOrValue::Value(U128(0))
                } else {
                    PromiseOrValue::Value(amount)
                }
            }
        }
    }
}


#[allow(non_snake_case)]
//human reading balances using metadata decimals
pub fn yoctoToToken(yocto_amount: Balance, decimals: u8) -> u128 {
    (yocto_amount + (5 * 10u128.pow((decimals - 1u8).into()))) / 10u128.pow(decimals.into())
}
#[allow(dead_code)]
pub fn min_deposit(decimals: u8) -> Balance {
    //0.1 FT is MIN_DEPOSIT
    10u128.pow((decimals - 1).into())
}

