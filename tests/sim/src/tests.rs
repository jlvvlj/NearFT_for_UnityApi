use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::{json,to_string};
use workspaces::prelude::*;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker,AccountId,InMemorySigner};
use near_crypto::{PublicKey as PubKey, SecretKey,KeyType};
use std::env;
use std::fs::{File};
use std::io::Write;
use serde::{Deserialize, Serialize};
use std::str;
use bs58;
use hex_string::HexString;
#[derive(Serialize, Deserialize)]
pub struct KeyFile {
    pub account_id: String,
    pub public_key: String,
    pub secret_key: String,
}

const FT_WASM_FILEPATH: &str = "../../out/NearFT_for_UnityApi.wasm";
// const potential game contract
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initiate environemnt
    let worker = workspaces::sandbox().await?;
    let wasm = workspaces::compile_project("../../").await?;

    //let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
    let ft_contract = worker.dev_deploy(&wasm).await?;
  
    // create user accounts
    let owner = worker.root_account();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let bob = owner
        .create_subaccount(&worker, "bob")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let charlie = owner
        .create_subaccount(&worker, "charlie")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;
    let dave = owner
        .create_subaccount(&worker, "dave")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // create api Key pairs   //monnom.mongame.moncontrat.near 
    let api_id : String = format!("gameapi.{}",ft_contract.as_account().id());
    let apipub : PubKey = PubKey::from_seed(KeyType::ED25519,"test").into();
    let apisec : SecretKey = SecretKey::from_seed(KeyType::ED25519,"test");
    let api_pk : String= format!("{}",apipub);
    let api_private : String= format!("{}",apisec);
    // writing api credential file
    let temp_directory = env::temp_dir();
    let temp_file = temp_directory.join(api_id.clone());
    let mut file = File::create(&temp_file).unwrap();

    let data = KeyFile{
        account_id: api_id.clone(),
        public_key: api_pk.clone(),
        secret_key: api_private.clone(),
    };
    let mydata : String =serde_json::to_string(&data).unwrap();
    writeln!(&mut file, "{}" , mydata );
    println!("{:?}",temp_file);

    let api_account : Account = Account::from_file(temp_file);
    //create user Key pairs
    let user_id : String = format!("gameuser.{}",ft_contract.as_account().id());
    let userpub : PubKey = PubKey::from_seed(KeyType::ED25519,"test").into();
    let usersec : SecretKey = SecretKey::from_seed(KeyType::ED25519,"test");
    let user_pk : String= format!("{}",userpub);
    let user_private : String= format!("{}",usersec);
    // writing user credential file
    let temp_directory = env::temp_dir();
    let temp_file = temp_directory.join(user_id.clone());
    let mut file = File::create(&temp_file).unwrap();

    let data = KeyFile{
        account_id: user_id.clone(),
        public_key: user_pk.clone(),
        secret_key: user_private.clone(),
    };
    let mydata : String =serde_json::to_string(&data).unwrap();
    writeln!(&mut file, "{}" , mydata );
    println!("{:?}",user_pk);

    let user_account : Account = Account::from_file(temp_file);
    
    /*let mut split = user_pk.split(":").map(ToString::to_string)
    .collect::<Vec<_>>();
    for s in split {
        println!("{}", s);
        user_implicit_key = bs58::encode(s).into_string();
    }
*/
    
    // Initialize contracts
    ft_contract
        .call(&worker, "new_default_meta")
        .args_json(serde_json::json!({
            "owner_id": alice.id(),
            "total_supply": parse_near!("1,000,000,000 N").to_string(),
        }))?
        .transact()
        .await?;
    

    // begin tests
    test_total_supply(&owner, &ft_contract, &worker).await?;
    //test_create_api_account(&owner, &ft_contract, &worker).await?;
    test_vault_initialized(&owner, &ft_contract, &worker).await?;
    test_create_api_account(&owner,&alice, &ft_contract, &worker,&api_id,&api_pk).await?;
    test_reserve_implicit_account(&owner,&alice,&api_account, &ft_contract, &worker,&user_id,&user_pk).await?;
    test_create_user_account(&owner,&alice,&api_account, &ft_contract, &worker,&user_id,&user_pk).await?;
    test_fund_api_account_withFT(&owner,&alice, &ft_contract, &worker,&api_id).await?;
    test_activate_user_account(&owner,&alice,&api_account,&user_account , &ft_contract, &worker).await?;
    test_set_owner_allowance(&owner,&alice,&api_account,&user_account , &ft_contract, &worker).await?;
    test_set_user_allowance(&owner,&alice,&api_account,&user_account , &ft_contract, &worker).await?;
    test_owner_withdraw(&alice,&user_account,&ft_contract,&worker).await?;
    test_user_withdraw(&alice,&user_account,&ft_contract,&worker).await?;
    //test_simple_withdraw(&owner, &alice, &ft_contract, &worker).await?;
    //test_can_close_empty_balance_account(&bob, &ft_contract, &worker).await?;
    
    //test_close_account_non_empty_balance(&alice, &ft_contract, &worker).await?;
    //test_close_account_force_non_empty_balance(&alice, &ft_contract, &worker).await?;
    //test_transfer_call_with_burned_amount(&owner, &charlie, &ft_contract, &defi_contract, &worker)
     //   .await?;
    /*test_simulate_transfer_call_with_immediate_return_and_no_refund(
        &owner,
        &ft_contract,
        &defi_contract,
        &worker,
    )
    .await?;
    test_transfer_call_when_called_contract_not_registered_with_ft(
        &owner,
        &dave,
        &ft_contract,
        &worker,
    )
    .await?;
    test_transfer_call_promise_panics_for_a_full_refund(&owner, &alice, &ft_contract, &worker)
        .await?;
    */
    Ok(())
}

async fn test_total_supply(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("1,000,000,000 N"));
    let res: U128 = owner
        .call(&worker, contract.id(), "ft_total_supply")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, initial_balance);

    println!("      Passed ✅ test_total_supply");
    Ok(())
}

async fn test_vault_initialized(
    owner: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let initial_balance = U128::from(parse_near!("1,000,000,000 N"));
    let res: U128 = owner
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(json!({"account_id": contract.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(res, initial_balance);
    
    println!("      Passed ✅ vault_initialized");
    Ok(())
}
async fn test_reserve_implicit_account(
    owner: &Account,
    alice: &Account,
    api_account : &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
    implicit_id: &str,
    apipub: &str,
) -> anyhow::Result<()> {
    // fund api account with minimum Near for account creation 
    let res = alice.transfer_near(worker, api_account.id(), parse_near!("10 N")).await?;
    let res = api_account
        .call(&worker, contract.id(), "reserve_user_account")
        .args_json(json!({"pk":apipub}))?
        // implicit account registration  
        .deposit(2_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    // find proper 
    println!("      Passed ✅ test_reserve_user_account");
  
    Ok(())
}
async fn test_create_api_account(
    owner: &Account,
    alice : &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
    id: &str,
    apipub: &str,
) -> anyhow::Result<()> {
    let res = alice
        .call(&worker, contract.id(), "create_api_account")
        .args_json(json!({"api_id":id,"new_public_key":apipub,"vault_allowance":"100"}))?
        // minimum Near needed to register api within the smartcontract
        .deposit(parse_near!("0.0019 N"))
        .transact()
        .await?;
        
        assert_eq!(res.is_success(), true);
    

        println!("      Passed ✅ test_create_api_account");
  /*let res= contract.as_account()
        .call(&worker, contract.id(), "ft_transfer")
        .args_json(json!({"receiver_id": newsubaccount, "amount":"100","memo":"My transfer"}))?
        .deposit(1)
        .transact()
        .await?;

println!("{:?}",res);*/
    Ok(())
}
async fn test_fund_api_account_withFT(
    owner: &Account,
    alice : &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
    id: &str,
) -> anyhow::Result<()> {
    let res= contract.as_account()
        .call(&worker, contract.id(), "ft_transfer")
        .args_json(json!({"receiver_id": id, "amount":"100","memo":"My transfer"}))?
        .deposit(1)
        .transact()
        .await?;
        
        assert_eq!(res.is_success(), true);
    
    let api_balance: U128 = alice
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": id
        }))?
        .transact()
        .await?
        .json()?;
       
        assert_eq!(api_balance, U128::from(100));
 
    Ok(())
}
async fn test_create_user_account(
    owner: &Account,
    alice: &Account,
    api_account : &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
    id: &str,
    apipub: &str,

) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    let res = alice.transfer_near(worker, api_account.id(), parse_near!("10 N")).await?;

    let res = api_account
        .call(&worker, contract.id(), "create_user_account")
        .args_json(json!({"new_account_id":id,"new_public_key":apipub}))?
        // implicit account registration  
        .deposit(4_000_000_000_000_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}",res);
    assert_eq!(res.is_success(), true);
 
        println!("      Passed ✅ test_create_user_account");
    Ok(())
}
async fn test_activate_user_account(
    owner: &Account,
    alice: &Account,
    api_account : &Account,
    user_account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    //let res = alice.transfer_near(worker, api_account.id(), parse_near!("0.015 N")).await?;
   // let res = alice.transfer_near(worker, user_account.id(), parse_near!("0.008 N")).await?;
    let res = api_account
        .call(&worker, contract.id(), "activate_implicit_user_account")
        .args_json(json!({"account_id":user_account.id()}))?
        .deposit(0)
        .transact()
        .await?;
        
    //println!("{:?}",res);
    println!("      Passed ✅ activate_implicit_account");
    Ok(())
}
async fn test_set_owner_allowance(
    owner: &Account,
    alice: &Account,
    api_account : &Account,
    user_account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    let res = alice.transfer_near(worker, api_account.id(), parse_near!("0.014 N")).await?;

    let res = api_account
        .call(&worker, contract.id(), "set_available_unites_to_player")
        .args_json(json!({"account_id":alice.id(),"amount":"100"}))?
        .deposit(0)
        .transact()
        .await?;
        
        assert_eq!(res.is_success(), true);
        println!("      Passed ✅ test_set_owner_allowance");
    Ok(())
}
async fn test_set_user_allowance(
    owner: &Account,
    alice: &Account,
    api_account : &Account,
    user_account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    let res = alice.transfer_near(worker, api_account.id(), parse_near!("0.014 N")).await?;

    let res = api_account
        .call(&worker, contract.id(), "set_available_unites_to_player")
        .args_json(json!({"account_id":user_account.id(),"amount":"100"}))?
        .deposit(0)
        .transact()
        .await?;
        
        assert_eq!(res.is_success(), true);
        println!("      Passed ✅ test_set_user_allowance");
    Ok(())
}
async fn test_owner_withdraw(
    alice: &Account,
    user_account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    //let res = alice.transfer_near(worker, api_account.id(), parse_near!("0.014 N")).await?;

    let res = alice
        .call(&worker, contract.id(), "withdraw")
        .args_json(json!({"amount":"50","memo":"withdaw"}))?
        .deposit(1)
        .transact()
        .await?;
    let owner_balance: U128 = alice
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": alice.id()
        }))?
        .transact()
        .await?
        .json()?;
    let expect :U128 = U128::from(50);
    assert_eq!(owner_balance, expect);
    println!("      Passed ✅ test_owner_withdraw");
    Ok(())
}
async fn test_user_withdraw(
    alice: &Account,
    user_account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {

    // fund api account with minimum Near for account creation 
    //let res = alice.transfer_near(worker, api_account.id(), parse_near!("0.014 N")).await?;

    let res = user_account
        .call(&worker, contract.id(), "withdraw")
        .args_json(json!({"amount":"50","memo":"withdraw"}))?
        .deposit(0)
        .transact()
        .await?;
    println!("{:?}",res);
    let expect :U128 = U128::from(50);
    let user_balance: U128 = alice
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user_account.id()
        }))?
        .transact()
        .await?
        .json()?;     
    assert_eq!(user_balance, expect);

    //
    //assert_eq!(res.is_success(), true);
    println!("      Passed ✅ test_user_withdraw");
    Ok(())
}
async fn test_simple_transfer(
    owner: &Account,
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount = U128::from(parse_near!("1,000 N"));

    // register user
    user.call(&worker, contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // transfer ft
    owner
        .call(&worker, contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount
        }))?
        .deposit(1)
        .transact()
        .await?;

    let root_balance: U128 = owner
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": owner.id()
        }))?
        .transact()
        .await?
        .json()?;

    let alice_balance: U128 = owner
        .call(&worker, contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .transact()
        .await?
        .json()?;

    assert_eq!(root_balance, U128::from(parse_near!("999,999,000 N")));
    assert_eq!(alice_balance, transfer_amount);

    println!("      Passed ✅ test_simple_transfer");
    Ok(())
}

async fn test_can_close_empty_balance_account(
    user: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    // register user
    user.call(&worker, contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    let result: bool = user
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await?
        .json()?;

    assert_eq!(result, true);
    println!("      Passed ✅ test_can_close_empty_balance_account");
    Ok(())
}

async fn test_close_account_non_empty_balance(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    match user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({}))?
        .deposit(1)
        .transact()
        .await
    {
        Ok(_result) => {
            panic!("storage_unregister worked despite account being funded")
        }
        Err(e) => {
            let e_string = e.to_string();
            if !e_string
                .contains("Can't unregister the account with the positive balance without force")
            {
                panic!("storage_unregister with balance displays unexpected error message")
            }
            println!("      Passed ✅ test_close_account_non_empty_balance");
        }
    }
    Ok(())
}

async fn test_close_account_force_non_empty_balance(
    user_with_funds: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let result: CallExecutionDetails = user_with_funds
        .call(&worker, contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    assert_eq!(true, result.is_success());
    assert_eq!(
        result.logs()[0],
        format!(
            "Closed @{} with {}",
            user_with_funds.id(),
            parse_near!("1,000 N") // alice balance from above transfer_amount
        )
    );
    println!("      Passed ✅ test_close_account_force_non_empty_balance");
    Ok(())
}

async fn test_transfer_call_with_burned_amount(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let transfer_amount_str = parse_near!("1,000,000 N").to_string();
    let ftc_amount_str = parse_near!("1,000 N").to_string();

    // register user
    owner
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    // transfer ft
    owner
        .call(&worker, ft_contract.id(), "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": transfer_amount_str
        }))?
        .deposit(1)
        .transact()
        .await?;

    user.call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": ftc_amount_str,
            "msg": "0",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let storage_result: CallExecutionDetails = user
        .call(&worker, ft_contract.id(), "storage_unregister")
        .args_json(serde_json::json!({"force": true }))?
        .deposit(1)
        .transact()
        .await?;

    // assert new state
    assert_eq!(
        storage_result.logs()[0],
        format!(
            "Closed @{} with {}",
            user.id(),
            parse_near!("999,000 N") // balance after defi ft transfer
        )
    );

    let total_supply: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(total_supply, U128::from(parse_near!("999,000,000 N")));

    let defi_balance: U128 = owner
        .call(&worker, ft_contract.id(), "ft_total_supply")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;
    assert_eq!(defi_balance, U128::from(parse_near!("999,000,000 N")));

    println!("      Passed ✅ test_transfer_call_with_burned_amount");
    Ok(())
}

async fn test_simulate_transfer_call_with_immediate_return_and_no_refund(
    owner: &Account,
    ft_contract: &Contract,
    defi_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount: u128 = parse_near!("100,000,000 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": defi_contract.id(),
            "amount": amount_str,
            "msg": "take-my-money"
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;

    let owner_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": owner.id()}))?
        .transact()
        .await?
        .json()?;
    let defi_after_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": defi_contract.id()}))?
        .transact()
        .await?
        .json()?;

    assert_eq!(owner_before_balance.0 - amount, owner_after_balance.0);
    assert_eq!(defi_before_balance.0 + amount, defi_after_balance.0);
    println!("      Passed ✅ test_simulate_transfer_call_with_immediate_return_and_no_refund");
    Ok(())
}

async fn test_transfer_call_when_called_contract_not_registered_with_ft(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount = parse_near!("10 N");
    let amount_str = amount.to_string();
    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id":  owner.id()}))?
        .transact()
        .await?
        .json()?;
    let user_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;

    match owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": amount_str,
            "msg": "take-my-money",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
    {
        Ok(res) => {
            panic!("Was able to transfer FT to an unregistered account");
        }
        Err(err) => {
            let owner_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id":  owner.id()}))?
                .transact()
                .await?
                .json()?;
            let user_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id": user.id()}))?
                .transact()
                .await?
                .json()?;
            assert_eq!(user_before_balance, user_after_balance);
            assert_eq!(owner_before_balance, owner_after_balance);
            println!(
                "      Passed ✅ test_transfer_call_when_called_contract_not_registered_with_ft"
            );
        }
    }
    Ok(())
}

async fn test_transfer_call_promise_panics_for_a_full_refund(
    owner: &Account,
    user: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let amount = parse_near!("10 N");

    // register user
    owner
        .call(&worker, ft_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": user.id()
        }))?
        .deposit(parse_near!("0.008 N"))
        .transact()
        .await?;

    let owner_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id":  owner.id()}))?
        .transact()
        .await?
        .json()?;
    let user_before_balance: U128 = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": user.id()}))?
        .transact()
        .await?
        .json()?;

    match owner
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": user.id(),
            "amount": amount,
            "msg": "no parsey as integer big panic oh no",
        }))?
        .deposit(1)
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await
    {
        Ok(res) => {
            panic!("Did not expect for trx to accept invalid paramenter data types")
        }
        Err(err) => {
            let owner_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id":  owner.id()}))?
                .transact()
                .await?
                .json()?;
            let user_after_balance: U128 = ft_contract
                .call(&worker, "ft_balance_of")
                .args_json(json!({"account_id": user.id()}))?
                .transact()
                .await?
                .json()?;
            assert_eq!(owner_before_balance, owner_after_balance);
            assert_eq!(user_before_balance, user_after_balance);
            println!("      Passed ✅ test_transfer_call_promise_panics_for_a_full_refund");
        }
    }
    Ok(())
}