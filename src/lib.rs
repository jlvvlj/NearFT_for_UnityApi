////// Unites Fungible token :
////// Can be bought with InGame Money /
//////
////// When transferred from owner (aka Game Wallet) checks the API via Oracle for valid user Balance (can a smart contract connect with credential)
//////  or
////// The api checks valid user balance and call ft_transfer / When called from API wallet ft_transfer MUST check for Valid API Credential (API access Key)
//////

pub use crate::events::*;

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, PanicOnDefault, Promise,
    PromiseOrValue, PromiseResult, PublicKey,
};
use serde::Serialize;
mod events;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct UnitesContract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner: AccountId,
    authorized_game_api: LookupMap<AccountId, u128>,
    implicit_accounts: LookupMap<PublicKey, Balance>,
    unites_for_player_accounts: LookupMap<AccountId, u128>,
    vault: AccountId,
    apis_ratio: LookupMap<AccountId, u128>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";
/// Gas attached to the callback from account creation.
pub const ON_CREATE_ACCOUNT_CALLBACK_GAS: u64 = 20_000_000_000_000;
/// Access key allowance for linkdrop keys.
const ACCESS_KEY_ALLOWANCE: u128 = 1_000_000_000_000_000_000_000_000;
#[ext_contract(ext_self)]
pub trait ExtUnites {
    /// Callback after plain account creation.
    fn on_user_account_created(
        &mut self,
        predecessor_account_id: AccountId,
        transfer_amount: U128,
        new_account_id: AccountId,
    ) -> bool;
    fn on_api_account_created(
        &mut self,
        predecessor_account_id: AccountId,
        vault_allowance: U128,
        transfer_amount: U128,
        new_account_id: AccountId,
    ) -> bool;
    fn on_withdraw (
        &mut self,
        predecessor_account_id: AccountId,
        transfer_amount: U128,
        memo: Option<String>,
    ) -> bool;
}
/// Helper for promise Result
fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

#[near_bindgen]
impl UnitesContract {
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "Unites".to_string(),
                symbol: "UNTS".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }
    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        // Funds are stored within the contract
        let vault_id = env::current_account_id();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner: owner_id,
            // See to set it to a contract adresse
            vault: vault_id,
            unites_for_player_accounts: LookupMap::new(b"c".to_vec()),
            authorized_game_api: LookupMap::new(b"o".to_vec()),
            apis_ratio: LookupMap::new(b"p".to_vec()),
            implicit_accounts:  LookupMap::new(b"g".to_vec()),
        };
        this.token.internal_register_account(&this.owner);
        this.token.internal_register_account(&this.vault);
        this.token
            .internal_deposit(&this.vault, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &this.owner,
            amount: &total_supply,
            memo: Some("Initial tokens supply minted and deposited into the vault"),
        }
        .emit();
        this
    }
    #[payable]
    /// Creating contract subaccount for API
    /// Owner Only
    pub fn create_api_account(
        &mut self,
        api_id: AccountId,
        new_public_key: PublicKey,
        vault_allowance:U128
    ) -> Promise {
        // function reserved to owner
        assert!(
            env::predecessor_account_id() == self.owner, 
            "Not authorized {} , {}" , env::predecessor_account_id() , self.owner
        );
        
        let new_account_id : AccountId = api_id;
        log!("new public key {:?}", new_public_key);
        Promise::new(new_account_id.clone())
            .create_account()
            .add_full_access_key(new_public_key.into())
            .transfer(env::attached_deposit())
            .then(ext_self::ext(env::current_account_id())
            .on_api_account_created(
                env::predecessor_account_id(),
                vault_allowance.into(),
                env::attached_deposit().into(),
                new_account_id,
            ))
    }
    /// create_account Callback [To be tested]
    pub fn on_api_account_created(
        &mut self,
        predecessor_account_id: AccountId,
        vault_allowance: U128,
        transfer_amount: U128,
        new_account_id: AccountId,
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
          // Construct the mint log as per the events standard.
          let log: EventLog = EventLog {
            standard: "Create account standard".to_string(),
            version: "alpha".to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::AccCreated(vec![AccCreatedLog {
                // Owner of the token.
                owner_id: self.owner.to_string(),
                // Vector of token IDs that were minted.
                acc_created: new_account_id.to_string(),
                // An optional memo to include.
                memo: None,
            }]),
        };
        let creation_succeeded = is_promise_success();
        // check if AccountId is valid
        if !creation_succeeded {
            // In case of failure, send funds back.
            Promise::new(predecessor_account_id).transfer(transfer_amount.into());
          
            env::log_str(&log.to_string());
        } 
        if creation_succeeded {
            
        self.add_whitelisted_api(new_account_id.clone(), vault_allowance.into());
        self.token.internal_register_account(&new_account_id);
        } 
      
        
        env::log_str(&log.to_string());
        creation_succeeded
    }
    #[payable]
    /// Progressive onboarding for Player without NEAR account
    /// Apis Only
    pub fn reserve_user_account(
        &mut self,
        pk: PublicKey,
        
        
    ) -> Promise {
        assert!(
            env::attached_deposit() > ACCESS_KEY_ALLOWANCE,
            "Attached deposit must be greater than ACCESS_KEY_ALLOWANCE"
        );
        let pk = pk.into();
        let value = self.implicit_accounts.get(&pk).unwrap_or(0);
        self.implicit_accounts.insert(
            &pk,
            &(value + env::attached_deposit() - ACCESS_KEY_ALLOWANCE),
        );
        Promise::new(env::current_account_id()).add_access_key(
            pk,
            ACCESS_KEY_ALLOWANCE,
            env::current_account_id(),
            "claim,create_account_and_claim".into(),
        )
    }
    #[payable]
    /// Progressive onboarding for Player without NEAR account
    /// Apis Only
    pub fn create_user_account(
        &mut self,
        new_account_id: AccountId,
        new_public_key: PublicKey,
        
    ) -> Promise {
        //function reserved to apis only 
        assert!(
            self.authorized_game_api.contains_key(&env::predecessor_account_id()), 
            "Not authorized {}", env::predecessor_account_id()
        );

        let amount = self
            .implicit_accounts
            .remove(&env::signer_account_pk())
            .expect("Unexpected public key");

        Promise::new(new_account_id.clone())
            .create_account()
            .add_full_access_key(new_public_key.into())
            .transfer(amount)
            .then(ext_self::ext(env::current_account_id()).on_user_account_created(
                env::predecessor_account_id(),
                amount.into(),
                new_account_id,
            ))
    }
    /// create_account Callback [To be tested]
    pub fn on_user_account_created(
        &mut self,
        predecessor_account_id: AccountId,
        transfer_amount: U128,
        new_account_id: AccountId,
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let creation_succeeded = is_promise_success();
        if creation_succeeded {
            Promise::new(env::current_account_id()).delete_key(env::signer_account_pk());
        } else {
            // In case of failure, put the amount back.
            self.implicit_accounts
                .insert(&env::signer_account_pk(), &transfer_amount.into());
        }
        
        
        // Construct the mint log as per the events standard.
        let log: EventLog = EventLog {
            standard: "Create account standard".to_string(),
            version: "alpha".to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::AccCreated(vec![AccCreatedLog {
                // Owner of the token.
                owner_id: self.owner.to_string(),
                // Vector of token IDs that were minted.
                acc_created: new_account_id.to_string(),
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&log.to_string());
        creation_succeeded
    }

    // should be restricted to owner
    fn add_whitelisted_api(&mut self, account_id: AccountId, amount: u128) {
        log!(
            "Registering new Game Api  @{} with Vault allowance @{}",
            account_id,
            amount
        );
        assert!(
            env::predecessor_account_id() == self.owner || env::predecessor_account_id() == env::current_account_id(),
            "Not authorized"
        );
        assert!(
            env::is_valid_account_id(&account_id.as_bytes().to_vec()),
            "Account Id invalid"
        );
        assert!(
            self.authorized_game_api.contains_key(&account_id) == false,
            "Account already existing"
        );
        self.authorized_game_api.insert(&account_id, &amount);
    }
    pub fn activate_implicit_user_account(&mut self, account_id: AccountId) {
        assert!(
            (self.authorized_game_api.contains_key(&env::predecessor_account_id()) || env::predecessor_account_id() ==  env::current_account_id() ),
            "Not authorized"
        );
        self.register_account_as_player(account_id.clone());
        self.token.internal_register_account(&account_id);
    }
    /// APIs Only
    pub fn register_account_as_player(&mut self, account_id: AccountId) {
        log!("Registering existing Account @{:?}", account_id);
        assert!(
            (self.authorized_game_api.contains_key(&env::predecessor_account_id()) || env::predecessor_account_id() ==  env::current_account_id() ),
            "Not authorized"
        );
        assert!(
            env::is_valid_account_id(&account_id.as_bytes().to_vec()),
            "Account Id invalid"
        );
        self.unites_for_player_accounts.insert(&account_id, &0);
    }
    
    /// Set available Unites for player Account
    /// APIs Only 
    pub fn set_available_unites_to_player(&mut self, account_id: AccountId, amount: U128) {
        // add check function reserved to owner
        assert!(
            (self.authorized_game_api.contains_key(&env::predecessor_account_id()) ), 
            "Not authorized"
        );
        self.unites_for_player_accounts.insert(&account_id, &amount.0);
    }
    /// To be implemented
    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }
    /// to be double checked
    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }

    /// Return the player withdraw allowance
    pub fn get_player_allowance(&self, player_id: AccountId) -> Option<u128> {
        self.unites_for_player_accounts.get(&player_id)
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for UnitesContract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

// overriding transfer functions
#[near_bindgen]
impl UnitesContract {
    /// Overidden function to  limit owner transfers to unknown users
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        // reduire l'autorisation avant de faire le transfer
        self.token.ft_transfer(receiver_id, amount, memo)
    }
    /// Overidden function to limit owner transfers to unknown users
    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        if self.authorized_game_api.contains_key(&receiver_id) {
            log!("Verify Allowance for witdrawal by {}", receiver_id);
            let api_allowance = match self.authorized_game_api.get(&receiver_id) {
                Some(unites) => (unites > 0 && unites >= amount.0),
                None => false,
            };
            assert_ne!(api_allowance, false, "Game Api is out of funds {} ",api_allowance); 
        }
        if self.unites_for_player_accounts.contains_key(&receiver_id) {
            log!("Verify Allowance for witdrawal by {}", receiver_id);
            let allowance = match self.unites_for_player_accounts.get(&receiver_id) {
                Some(unites) => (unites > 0 && unites >= amount.0),
                None => false,
            };
            assert_ne!(allowance, false, "Player is out of funds"); 

        }
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }
    #[payable]
    pub fn withdraw(
        &mut self,
        amount: U128,
        memo: Option<String>,
        
    )  {
        assert!(self.unites_for_player_accounts.contains_key(&env::predecessor_account_id()));
        log!("Verify Allowance for witdrawal for {}", env::predecessor_account_id(),);
            let test: U128;
            let is_allowed = match self.unites_for_player_accounts.get(&env::predecessor_account_id(),) {
                Some(unites) => (unites > 0 && unites >= amount.0),
                None => false,
            };
            assert_ne!(is_allowed, false, "Player is out of funds or not registered");
            let vault_allowance: u128 = self.unites_for_player_accounts.get(&env::predecessor_account_id(),).unwrap();
            let rest  = vault_allowance - amount.0;
            // Is there better way to modify value from  lookup maps 
            self.unites_for_player_accounts.remove(&env::predecessor_account_id(),); 
            self.unites_for_player_accounts.insert(&env::predecessor_account_id(),&rest); 
            // Free the funds from the vault 
            //self.token.internal_register_account(&env::predecessor_account_id());
            self.token.internal_transfer(&env::current_account_id(), &env::predecessor_account_id(), amount.0, memo);
            
            
        }
    pub fn on_withdraw (
        &mut self,
        predecessor_account_id: AccountId,
        transfer_amount: U128,
        memo: Option<String>
    ) -> bool {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let creation_succeeded = is_promise_success();
        // check if AccountId is valid
        if !creation_succeeded {

            assert_eq!(creation_succeeded, false, "That failed")
        }
        let test = self.token.ft_transfer(predecessor_account_id,transfer_amount,memo);
        log!("ft_transfer result: {:?}", test);
        creation_succeeded
    }

}

near_contract_standards::impl_fungible_token_core!(UnitesContract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(UnitesContract, token, on_account_closed);

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};
    use ed25519_dalek::Keypair;
    //use rand::rngs::OsRng;
    use ed25519_dalek::Signature;
    use rand_os::OsRng;


    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    /// Check that the contract holds the minted funds
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = UnitesContract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(contract.vault.clone()).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = UnitesContract::default();
    }
    #[test] 
    fn  test_apis() {

    }

    #[test]
    #[should_panic(expected = "Not authorized")]
    fn test_invalid_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract =
            UnitesContract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);
        
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
    #[test]
    fn test_valid_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract =
            UnitesContract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;

        // Web app  qui  verifie que les ingames sont valables
        // l'utilisateur apuuie sur submit
        contract.register_account_as_player(accounts(1));
        contract.set_available_unites_to_player(accounts(1), transfer_amount.into());
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transfer_amount)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
        let is_registered = contract.get_player_allowance(accounts(1));
        assert_ne!(is_registered, None);
    }
    #[test]
    fn test_create_account() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract =
            UnitesContract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        let mut key = [0u8; 16];
        let mut rngs = OsRng{};
        let keypair:Keypair  = Keypair::generate(&mut rngs);
        
        let game_api = AccountId::new_unchecked(
            "GameaApi".parse().unwrap()
        ); 
        println!("{:?}",keypair.public);
       
       /* contract.create_api_account(
            game_api,
            "qSq3LoufLvTCTNGC3LJePMDGWok8dHMQ5A1YD9psbiz"
                .parse()
                .unwrap(),
        );
        */
        let game_api2 = AccountId::new_unchecked(
            "GameaApi".parse().unwrap()
        ); 
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(game_api2)
            .build());
        /*contract.create_account(
            AccountId::new_unchecked("newplayer".to_string()),
            "qSq3LoufLvTCTNGC3LJePMDGrok8dHMQ5A1YD9psbiz"
                .parse()
                .unwrap(),
        );
*/
        assert_eq!(
            contract
                .ft_balance_of(AccountId::new_unchecked("newplayer".to_string()))
                .0,
            0
        );
        //need to test callback in sandbox mode
        //let is_registered = contract.get_player_allowance(AccountId::new_unchecked("newplayer.test.near".to_string()));
        //assert_ne!(is_registered, None);
    }
}
