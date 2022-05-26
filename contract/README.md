NEAR Checkers For Fungible Tokens

How to deploy
==================

- Create & deploy game contract (`/contract`)
### build
RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release

### deploy (-f is optional for redeploy)
near deploy -f --wasmFile target/wasm32-unknown-unknown/release/checkers.wasm --accountId $CONTRACT_ID

##### step 0

initialize contract
`near call $CONTRACT_ID new '{}' --accountId $CONTRACT_ID`
whitelist cheddar (or any FT)
`near call $CONTRACT_ID whitelist_token '{"token_id":"'token-v3.cheddar.testnet'"}' --accountId $CONTRACT_ID`
check is_whitelisted_token
`near call $CONTRACT_ID is_whitelisted_token '{"token_id":"'token-v3.cheddar.testnet'"}' --accountId $CONTRACT_ID`

##### step 1 -> deposit to game and look available players

make_available -> join
---> NEAR
`near call $CONTRACT_ID make_available '{"config": {"token_id":"'NEAR'","first_move": "Random"}, "referrer_id": null}' --accountId
$USER_ACCOUNT_1 --depositYocto 10000000000000000000000`
---> CHEDDAR(or any FT), (+30 Tgas for call)
`near call token-v3.cheddar.testnet ft_transfer_call '{"receiver_id":"'$CONTRACT_ID'","amount":"1000000000000000000000000", "msg":"deposit"}' -
accountId $USER_ACCOUNT_2 --depositYocto 1 --gas 300000000000000` this calls make_available_ft in contract
`near call token-v3.cheddar.testnet ft_transfer_call '{"receiver_id":"'$CONTRACT_ID'","amount":"1000000000000000000000000", "msg":"deposit"}' -
accountId $USER_ACCOUNT_3 --depositYocto 1 --gas 300000000000000` this calls make_available_ft in contract

get_available_players
`near call $CONTRACT_ID get_available_players '{"from_index":0, "limit": 50}' --accountId $CONTRACT_ID`

##### step 2 -> start game and look for available games and try to give_up
start_game
`near call $CONTRACT_ID start_game '{"opponent_id": "'USER_ACCOUNT_2'"}' --accountId USER_ACCOUNT_3`
get_available_games
`near call $CONTRACT_ID get_available_games '{"from_index":0, "limit": 50}' --accountId $CONTRACT_ID` 
give up
`near call $CONTRACT_ID give_up '{"game_id":0}' --accountId USER_ACCOUNT_3 --depositYocto 1`
##### step 3 -> make you unavailable
make_unavailable
`near call $CONTRACT_ID make_unavailable '' --accountId USER_ACCOUNT_2 --depositYocto 1` 
get stats
`near call $CONTRACT_ID get_stats '{"account_id":"'USER_ACCOUNT'"}' --accountId $CONTRACT_ID`






