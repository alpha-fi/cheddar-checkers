use near_sdk::{Promise, PromiseOrValue, Timestamp};
use near_sdk::collections::UnorderedSet;
use near_sdk::json_types::U128;
use token_interfaces::{ext_ft, CALLBACK_GAS, ONE_YOCTO};

use crate::*;
pub (crate) type TokenId = AccountId;
pub (crate) type AffiliateId = AccountId;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum FirstMoveOptions {
    Random,
    First,
    Second,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GameConfig {
    pub(crate) token_id: Option<AccountId>,
    pub(crate) deposit: Option<Balance>,
    pub(crate) first_move: FirstMoveOptions,
    pub(crate) opponent_id: Option<AccountId>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VGameConfig {
    Current(GameConfig)
}

impl From<VGameConfig> for GameConfig {
    fn from(v_game_config: VGameConfig) -> Self {
        match v_game_config {
            VGameConfig::Current(game_config) => game_config,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GameConfigOutput {
    token_id: Option<AccountId>,
    deposit: U128,
    first_move: FirstMoveOptions,
    opponent_id: Option<AccountId>,
}

impl From<GameConfig> for GameConfigOutput {
    fn from(config: GameConfig) -> Self {
        GameConfigOutput {
            token_id: config.token_id,
            deposit: U128::from(config.deposit.unwrap_or(0)),
            first_move: config.first_move,
            opponent_id: config.opponent_id,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Stats {
    referrer_id: Option<AccountId>,
    affiliates: UnorderedSet<AffiliateId>,
    games_num: u64,
    victories_num: u64,
    penalties_num: u64,
    total_reward: UnorderedMap<Option<TokenId>, Balance>,
    total_affiliate_reward: UnorderedMap<Option<AffiliateId>, Balance>,
}

impl Stats {
    pub fn new(account_id: &AccountId) -> Stats {
        Stats {
            referrer_id: None,
            affiliates: UnorderedSet::new(StorageKey::Affiliates { account_id: account_id.clone() }),
            games_num: 0,
            victories_num: 0,
            penalties_num: 0,
            total_reward: UnorderedMap::new(StorageKey::TotalRewards { account_id: account_id.clone() }),
            total_affiliate_reward: UnorderedMap::new(StorageKey::TotalAffiliateRewards { account_id: account_id.clone() }),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum VStats {
    Current(Stats),
}

impl From<VStats> for Stats {
    fn from(v_stats: VStats) -> Self {
        match v_stats {
            VStats::Current(stats) => stats,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StatsOutput {
    referrer_id: Option<AccountId>,
    affiliates: Vec<AffiliateId>,
    games_num: u64,
    victories_num: u64,
    penalties_num: u64,
    token_id: TokenId,
    total_reward: U128,
    total_affiliate_reward: U128,
}

impl StatsOutput {
    fn from_by_token(stats: Stats, token_id: Option<TokenId>) -> StatsOutput {
        StatsOutput {
            referrer_id: stats.referrer_id,
            affiliates: stats.affiliates.to_vec(),
            games_num: stats.games_num,
            victories_num: stats.victories_num,
            penalties_num: stats.penalties_num,
            token_id: token_id.clone().unwrap_or_else(|| "NEAR".into()),
            total_reward: U128::from(stats.total_reward.get(&token_id).unwrap_or(0)),
            total_affiliate_reward: U128::from(stats.total_affiliate_reward.get(&token_id).unwrap_or(0)),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenBalance {
    pub(crate) token_id: Option<AccountId>,
    pub(crate) balance: Balance,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenBalanceOutput {
    token_id: AccountId,
    balance: U128,
}

impl From<TokenBalance> for TokenBalanceOutput {
    fn from(token_balance: TokenBalance) -> Self {
        TokenBalanceOutput {
            token_id: token_balance.token_id.unwrap_or_else(|| "NEAR".into()),
            balance: U128::from(token_balance.balance),
        }
    }
}

pub type BoardOutput = [Vec<i8>; 8];

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct GameOutput {
    player_1: AccountId,
    player_2: AccountId,
    current_player_index: usize,
    reward: TokenBalanceOutput,
    winner_index: Option<usize>,
    turns: u64,
    last_turn_timestamp: Timestamp,
    total_time_spent: Vec<Timestamp>,
    board: BoardOutput,
}


#[derive(PartialEq)]
pub enum UpdateStatsAction {
    AddPlayedGame,
    AddReferral,
    AddAffiliate,
    AddWonGame,
    AddTotalReward,
    AddAffiliateReward,
    AddPenaltyGame,
}

impl Checkers {

    pub(crate) fn internal_distribute_reward(&mut self, token_balance: &TokenBalance, winner_id: &AccountId) {
       
        let amount = token_balance.balance;
        let token_id = token_balance.token_id.clone();
        let fee = amount / 10;

        let winner_reward: Balance = amount - fee;
        if token_id == Some("NEAR".into()) {
            Promise::new(winner_id.clone()).transfer(winner_reward);
        } else {
            match token_id {
                Some(ref token_contract) => {
                    ext_ft::ft_transfer(
                        winner_id.clone(),
                        winner_reward.to_string(),
                        &token_contract,
                        ONE_YOCTO,
                        CALLBACK_GAS 
                    );
                }
                _ => {
                    panic!("no token_id find in contract") 
                }
            }
        }

        log!("Winner is {}. Reward: {}", winner_id, winner_reward);

        // Referrer rewards
        let stats = self.internal_get_stats(winner_id);
        let referrer_fee = if let Some(referrer_id) = stats.referrer_id {
            let referrer_fee = fee / 2;
            log!("Affiliate reward for {} is {}", referrer_id, referrer_fee);
            self.internal_update_stats(&token_id, &referrer_id, UpdateStatsAction::AddAffiliateReward, None, Some(referrer_fee));

            if token_id == Some("NEAR".into()) {
                Promise::new(referrer_id.clone()).transfer(referrer_fee);
            } else {
                match token_id {
                    Some(ref token_contract) => {
                        ext_ft::ft_transfer(
                            referrer_id.clone(),
                            referrer_fee.to_string(),
                            &token_contract,
                            ONE_YOCTO,
                            CALLBACK_GAS 
                        );
                    }
                    _ => {
                        panic!("no token_id find in contract") 
                    }
                }
            }
            referrer_fee
        } else {
            0
        };

        self.service_fee += fee - referrer_fee;

        self.internal_update_stats(&token_id.clone(),winner_id, UpdateStatsAction::AddWonGame, None   , None);
        self.internal_update_stats(&token_id, winner_id, UpdateStatsAction::AddTotalReward, None, Some(winner_reward));

        // finish
        // TODO add to stats
    }

    pub(crate) fn internal_update_stats(&mut self,
                                        token_id: &Option<String>,
                                        account_id: &AccountId,
                                        action: UpdateStatsAction,
                                        additional_account_id: Option<AccountId>,
                                        balance: Option<Balance>) {
        let mut stats = self.internal_get_stats(account_id);

        if action == UpdateStatsAction::AddPlayedGame {
            stats.games_num += 1
        } else if action == UpdateStatsAction::AddReferral {
            if additional_account_id.is_some() {
                stats.referrer_id = additional_account_id;
                log!("referrer added here");
            }
        } else if action == UpdateStatsAction::AddAffiliate {
            if let Some(additional_account_id_unwrapped) = additional_account_id {
                stats.affiliates.insert(&additional_account_id_unwrapped);
            }
        } else if action == UpdateStatsAction::AddWonGame {
            stats.victories_num += 1;
        } else if action == UpdateStatsAction::AddTotalReward {
            if let Some(balance_unwrapped) = balance {
                //near
                if token_id == &Some("NEAR".into()) {
                    let total_reward = stats.total_reward.get(&None).unwrap_or(0);
                    stats.total_reward.insert(&None, &(total_reward + balance_unwrapped));
                } else {
                    //ft
                    let total_reward = stats.total_reward
                        .get(&token_id)
                        .unwrap_or(0);
                    stats.total_reward.insert(&token_id, &(total_reward + balance_unwrapped));
                }
            }
        } else if action == UpdateStatsAction::AddAffiliateReward {
            if let Some(balance_unwrapped) = balance {
                //near
                if token_id == &Some("NEAR".into()) {
                    let total_affiliate_reward = stats.total_affiliate_reward.get(&None).unwrap_or(0);
                    stats.total_affiliate_reward.insert(&None, &(total_affiliate_reward + balance_unwrapped));
                } else {
                    //ft
                    let total_affiliate_reward = stats.total_affiliate_reward
                        .get(&token_id)
                        .unwrap_or(0);
                    stats.total_affiliate_reward.insert(&token_id, &(total_affiliate_reward + balance_unwrapped));
                }
            }
        } else if action == UpdateStatsAction::AddPenaltyGame {
            stats.penalties_num += 1;
        }

        self.stats.insert(account_id, &VStats::Current(stats));
    }

    pub(crate) fn internal_get_game(&self, game_id: &GameId) -> GameToSave {
        self.games.get(game_id).expect("Game not found")
    }

    pub(crate) fn is_account_exists(&self, account_id: &Option<AccountId>) -> bool {
        match account_id {
            Some(account) => {
                if let Some(_stats) = self.stats.get(account) {
                    true
                } else {
                    false
                }
            }
            None => {
                log!("Account args was not added in function call!");
                false
            }
        }
    }

    pub(crate) fn internal_get_stats(&self, account_id: &AccountId) -> Stats {
        if let Some(stats) = self.stats.get(account_id) {
            stats.into()
        } else {
            Stats::new(&account_id)
        }
    }
}

#[near_bindgen]
impl Checkers {
    pub fn get_available_games(&self, from_index: u64, limit: u64) -> Vec<(GameId, (AccountId, AccountId))> {
        let keys = self.available_games.keys_as_vector();
        let values = self.available_games.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let accounts: (AccountId, AccountId) = values.get(index).unwrap().into();
                (keys.get(index).unwrap(), accounts)
            })
            .collect()
    }
    #[payable]
    pub fn make_unavailable(&mut self) -> PromiseOrValue<bool> {
        let account_id = env::predecessor_account_id();
        if let Some(v_game_config) = self.available_players.get(&account_id) {
            let config: GameConfig = v_game_config.into();
            let token_id = config.token_id;
            self.available_players.remove(&account_id);
            if token_id == Some("NEAR".into()) {
                PromiseOrValue::Promise(Promise::new(account_id).transfer(config.deposit.unwrap_or(0)))
            } else {
                match token_id {
                    
                    Some(ref token_contract) => {
                        assert_eq!(env::attached_deposit(), ONE_YOCTO, "Attach 1 yocto");
                        PromiseOrValue::Promise(ext_ft::ft_transfer(
                            account_id.clone(),
                            config.deposit.unwrap_or(0).to_string(),
                            &token_contract,
                            ONE_YOCTO,
                            CALLBACK_GAS  
                        ))
                    }
                    _ => {
                        PromiseOrValue::Value(false) 
                    }
                }
            }
        } else {
            PromiseOrValue::Value(false)
        }
    }

    pub fn get_stats(&self, account_id: AccountId, token_id: Option<TokenId>) -> StatsOutput {
        let stats = self.internal_get_stats(&account_id);
        if let Some(token_id) = token_id.clone() {
            assert!(self.is_whitelisted_token(token_id), "Token isn't whitelisted!");
        }
        StatsOutput::from_by_token(stats, token_id)
    }

    pub fn get_game(&self, game_id: GameId) -> GameOutput {
        let game: Game = self.internal_get_game(&game_id).into();

        GameOutput {
            player_1: game.players[0].account_id.clone(),
            player_2: game.players[1].account_id.clone(),
            current_player_index: game.current_player_index,
            reward: game.reward.into(),
            winner_index: game.winner_index,
            turns: game.turns,
            last_turn_timestamp: game.last_turn_timestamp,
            total_time_spent: game.total_time_spent,
            board: game.board.into(),
        }
    }

    pub fn get_available_moves(&self, game_id: GameId) -> (Vec<SimpleMove>, Vec<JumpMove>) {
        let game: Game = self.internal_get_game(&game_id).into();
        (game.available_simple_moves, game.available_jump_moves)
    }

    pub fn get_available_players(&self, from_index: u64, limit: u64) -> Vec<(AccountId, GameConfigOutput)> {
        let keys = self.available_players.keys_as_vector();
        let values = self.available_players.values_as_vector();
        (from_index..std::cmp::min(from_index + limit, keys.len()))
            .map(|index| {
                let config: GameConfig = values.get(index).unwrap().into();
                (keys.get(index).unwrap(), config.into())
            })
            .collect()
    }

    pub fn get_active_player(&self, game_id: GameId) -> AccountId {
        let game: Game = self.internal_get_game(&game_id).into();
        game.current_player_account_id()
    }

    pub fn get_service_fee(&self) -> U128 {
        U128::from(self.service_fee)
    }

    //FT
    pub fn is_whitelisted_token(&self, token_id: AccountId) -> bool {
        match self.whitelisted_tokens.get(&token_id) {
            Some(_) => true,
            None => false,
        }
    }
}
