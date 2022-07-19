NEAR Checkers For Fungible Tokens

How to deploy
==================

- Create & deploy game contract (`/contract`)
### build
RUSTFLAGS="-C link-arg=-s" cargo build --target wasm32-unknown-unknown --release 
cp target/wasm32-unknown-unknown/release/*.wasm ./res/

```shell
export CHECKERS_CONTRACT=checkers.cheddar.testnet
export CHEDDAR_CONTRACT=token-v3.cheddar.testnet
export USER_ACCOUNT_1=rmlsnk.testnet
export USER_ACCOUNT_2=participant_1.testnet
export USER_ACCOUNT_3=participant_2.testnet
export USER_ACCOUNT_4=participant_3.testnet
export USER_ACCOUNT_5=participant_4.testnet
export GAS=300000000000000
```

### deploy (-f is optional for redeploy).
```bash 
near dev-deploy -f --wasmFile target/wasm32-unknown-unknown/release/checkers.wasm
near deploy -f --wasmFile target/wasm32-unknown-unknown/release/checkers.wasm --accountId $CHECKERS_CONTRACT
```  

##### step 0  

```bash 
#initialize contract
near call $CHECKERS_CONTRACT new "{}" --accountId $CHECKERS_CONTRACT
#whitelist cheddar (or any FT)  
near call $CHECKERS_CONTRACT whitelist_token "{"token_id":""$CHEDDAR_CONTRACT""}" --accountId $CHECKERS_CONTRACT --gas $GAS
#check is_whitelisted_token  
near call $CHECKERS_CONTRACT is_whitelisted_token "{"token_id":""$CHEDDAR_CONTRACT""}" --accountId $CHECKERS_CONTRACT
```  

##### step 1 -> deposit to game and look available players
```bash

#make_available -> join  
#---> NEAR  
near call $CHECKERS_CONTRACT make_available "{"config": {"first_move": "Random"}, "referrer_id": null}" --accountId=$USER_ACCOUNT_2 --depositYocto 10000000000000000000000
near call $CHECKERS_CONTRACT make_available "{"config": {"first_move": "Random"}, "referrer_id": null}" --accountId=$USER_ACCOUNT_4 --depositYocto 10000000000000000000000

#---> CHEDDAR(or any FT), (+30 Tgas for call) 
#*no referral*
near call $CHEDDAR_CONTRACT ft_transfer_call "{"receiver_id":""$CHECKERS_CONTRACT"","amount":"1000000000000000000000000", "msg":""}" --accountId=$USER_ACCOUNT_2 --depositYocto 1 --gas $GAS 
#Note: this calls make_available_ft in contract 

#*with referral_id = $USER_ACCOUNT_2* 
near call $CHEDDAR_CONTRACT ft_transfer_call "{"receiver_id":""$CHECKERS_CONTRACT"","amount":"1000000000000000000000000", "msg":""$USER_ACCOUNT_2""}" --accountId=$USER_ACCOUNT_1 --depositYocto 1 --gas $GAS 
```

```bash
#get_available_players  
near call $CHECKERS_CONTRACT get_available_players "{"from_index":0, "limit": 50}" --accountId $CHECKERS_CONTRACT
```
##### step 2 -> start game and look for available games and try to give_up 
```bash 
#start_game NEAR , game_id = 0
near call $CHECKERS_CONTRACT start_game "{"opponent_id": ""$USER_ACCOUNT_4""}" --accountId=$USER_ACCOUNT_3
#start_game CHEDDAR, game_id = 1
near call $CHECKERS_CONTRACT start_game "{"opponent_id": ""$USER_ACCOUNT_3""}" --accountId=$USER_ACCOUNT_1 
#get_available_games  
near call $CHECKERS_CONTRACT get_available_games "{"from_index":0, "limit": 50}" --accountId $CHECKERS_CONTRACT   
#give up  
near call $CHECKERS_CONTRACT give_up "{"game_id":0}" --accountId=$USER_ACCOUNT_2 --depositYocto 1
near call $CHECKERS_CONTRACT give_up "{"game_id":0}" --accountId=$USER_ACCOUNT_1 --depositYocto 1
```  
##### step 3 -> make you unavailable 
```bash 
#make_unavailable  
near call $CHECKERS_CONTRACT make_unavailable "" --accountId=$USER_ACCOUNT_1 --depositYocto 1   
#get stats(for CHEDDAR)
near call $CHECKERS_CONTRACT get_stats "{"account_id":""$USER_ACCOUNT_3"", "token_id":""$CHEDDAR_CONTRACT""}" --accountId $CHECKERS_CONTRACT
near call $CHECKERS_CONTRACT get_stats "{"account_id":""$USER_ACCOUNT_1"", "token_id":""$CHEDDAR_CONTRACT""}" --accountId $CHECKERS_CONTRACT
#get stats(for NEAR)
near call $CHECKERS_CONTRACT get_stats "{"account_id":""$USER_ACCOUNT_2""}" --accountId $CHECKERS_CONTRACT
near call $CHECKERS_CONTRACT get_stats "{"account_id":""$USER_ACCOUNT_4""}" --accountId $CHECKERS_CONTRACT   

```