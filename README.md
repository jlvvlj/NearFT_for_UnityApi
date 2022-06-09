# NearFT_for_UnityApi

NEAR Ft token Contract implementing near_contract_standards::fungible_token. 
Transfer functions are overriden to provide verification of an initial allowance by the contract owner. 
He MUST set an allowance before the user can request token from this contract.  
This can be used by a game API to convert ingame money to Near FT token.
Handles user with a pre-created Near account and implements progressive onboarding.

### Tests 

Build with wasm-unknown-unknown 

```
cargo test 
test tests::test_default - should panic ... ok
test tests::test_create_account ... ok
test tests::test_invalid_transfer - should panic ... ok
test tests::test_valid_transfer ... ok
test tests::test_new ... ok
```
### Interfaces
```

```

### TO DO

- Simulation test with metaseed-unity-toolkit
- Plain Javascript Simulation Test
- Allow to set an API Wallet 
