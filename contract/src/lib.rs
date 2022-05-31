use near_sdk::{AccountId, Balance, BorshStorageKey, env, log, near_bindgen, PanicOnDefault, setup_alloc, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
pub use ai::{
    Direction,
    find_jump_moves_for_king,
    find_jump_moves_for_man,
    find_simple_moves_for_king,
    find_simple_moves_for_man,
    JumpMove,
    SimpleMove};
pub use board::{Board, BoardPosition};
pub use display::print_board;
pub use game::{Game, GameState, MoveError};
pub use input::{InputError, parse_move, TokenError};
pub use piece::{KingPiece, ManPiece, Piece, PieceType};
pub use player::Player;
pub use tile::{EmptyTile, OccupiedTile, Tile};
pub use token_interfaces::WhitelistedToken;


use crate::game::GameToSave;
use crate::manager::*;

use crate::token_interfaces::{ONE_YOCTO, yoctoToToken};

mod ai;
mod board;
mod display;
mod game;
mod input;
mod piece;
mod player;
mod tile;
mod util;
mod manager;
mod token_interfaces;

type GameId = u64;

// 0.01 NEAR
const MIN_DEPOSIT_NEAR: Balance = 10_000_000_000_000_000_000_000;

const ONE_HOUR: Timestamp = 3_600_000_000_000;

const CHECKERBOARD_SIZE: usize = 8;
const CHECKERS_NUMBER_TILES: usize = CHECKERBOARD_SIZE * CHECKERBOARD_SIZE;

setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Games,
    AvailablePlayers,
    Stats,
    AvailableGames,
    Affiliates {account_id: AccountId},
    TotalRewards {account_id: AccountId},
    TotalAffiliateRewards{ account_id: AccountId},
    WhitelistedTokens
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Checkers {
    games: LookupMap<GameId, GameToSave>,
    available_players: UnorderedMap<AccountId, VGameConfig>,
    stats: UnorderedMap<AccountId, VStats>,
    available_games: UnorderedMap<GameId, (AccountId, AccountId)>,
    whitelisted_tokens: UnorderedMap<AccountId, WhitelistedToken>,

    next_game_id: GameId,
    service_fee: Balance,
}

#[near_bindgen]
impl Checkers {
    #[init]
    pub fn new() -> Self {
        Self {
            games: LookupMap::new(StorageKey::Games),
            available_players: UnorderedMap::new(StorageKey::AvailablePlayers),
            stats: UnorderedMap::new(StorageKey::Stats),
            available_games: UnorderedMap::new(StorageKey::AvailableGames),
            whitelisted_tokens: UnorderedMap::new(StorageKey::WhitelistedTokens),

            next_game_id: 0,
            service_fee: 0,
        }
    }
}

#[near_bindgen]
impl Checkers {
    pub(crate) fn internal_add_referral(&mut self, token_id: Option<String>, account_id: &AccountId, referrer_id: &Option<AccountId>) {
        if self.stats.get(account_id).is_none() && self.is_account_exists(referrer_id) {
            if let Some(referrer_id_unwrapped) = referrer_id.clone() {
                self.internal_update_stats(&token_id, account_id, UpdateStatsAction::AddReferral, referrer_id.clone(), None);
                self.internal_update_stats(&token_id, &referrer_id_unwrapped, UpdateStatsAction::AddAffiliate, Some(account_id.clone()), None);
                log!("Referrer {} added for {}", referrer_id_unwrapped, account_id);
            }
        } else {
            log!("referrer was not added")
        }
    }

    #[payable]
    pub fn make_available(&mut self, config: GameConfig, referrer_id: Option<AccountId>) {
        let account_id: &AccountId = &env::predecessor_account_id();
        assert!(self.available_players.get(account_id).is_none(), "Already in the waiting list the list");
        let token_id = config.token_id.clone();
        //SWITCH TOKEN OPTION
        if token_id == Some("NEAR".into()) {
            let deposit: Balance = env::attached_deposit();
            assert!(deposit >= MIN_DEPOSIT_NEAR, "Deposit is too small. Attached: {}, Required: {}", deposit, MIN_DEPOSIT_NEAR);
            self.available_players.insert(account_id,
                &VGameConfig::Current(GameConfig {
                    token_id: config.token_id,
                    deposit: Some(deposit),
                    first_move: config.first_move,
                    opponent_id: config.opponent_id,
            }));
            self.internal_check_if_has_game_started(account_id);
            self.internal_add_referral(token_id, account_id, &referrer_id);
        } else {
            match token_id {
                Some(ref token_contract) => {
                    let ticker = self.get_token_ticker(token_contract.clone());
                    log!(
                        "Transfer some ${} tokens to deposit into contract and make you available to start play",
                        ticker
                    );
                },
                _ => panic!("token {:?} is not whitelisted", token_id )
            }
        }
    }
    //calls in cross-contract transfer into checkers app
    pub fn make_available_ft(&mut self, sender_id: AccountId, amount: u128, token_id: AccountId, referrer_id: Option<AccountId>) {
    
        //get token data
        let is_token_whitelisted = self.is_whitelisted_token(token_id.clone());
        if is_token_whitelisted {
            let decimals = self.get_token_decimals(token_id.clone());
            let ticker = self.get_token_ticker(token_id.clone());

            //create config
                self.available_players.insert(&sender_id,
                    &VGameConfig::Current(GameConfig {
                        token_id: Some(token_id.clone()),
                        deposit: Some(amount),
                        first_move: FirstMoveOptions::Random,
                        opponent_id: None,
                    }));

            log!("Success deposit from @{} with amount {} of ${}' contract ", sender_id.clone(), yoctoToToken(amount, decimals), ticker);
            self.internal_check_if_has_game_started(&sender_id);
            self.internal_add_referral(Some(token_id), &sender_id, &referrer_id);
        } else {
            log!("Failed deposit from @{}. Game Config not found! ", sender_id.clone());
        }
    }

    pub(crate) fn internal_check_if_has_game_started(&self, account_id: &AccountId) {
        let games_already_started: Vec<(AccountId, AccountId)> = self.available_games.values_as_vector()
            .iter()
            .filter(|(player_1, player_2)| *player_1 == *account_id || *player_2 == *account_id)
            .collect();
        assert_eq!(games_already_started.len(), 0, "Another game already started");
    }
    
    #[payable]
    pub fn start_game(&mut self, opponent_id: AccountId, referrer_id: Option<AccountId>) -> GameId {
        if let Some(opponent_config) = self.available_players.get(&opponent_id) {
            let config: GameConfig = opponent_config.into();
            
            let account_id = env::predecessor_account_id();
            assert_ne!(account_id.clone(), opponent_id.clone(), "Find a friend to play");

            self.internal_check_if_has_game_started(&account_id);

            if let Some(player_id) = config.opponent_id {
                assert_eq!(player_id, account_id, "Wrong account");
            }

            let game_id = self.next_game_id;

            let reward = TokenBalance {
                token_id: config.token_id,
                balance: config.deposit.unwrap_or(0) * 2,
            };

            log!("game reward:  token {:?} ", reward.token_id.clone());
            
            let token_id = reward.token_id.clone();
            if token_id.clone() == Some("NEAR".into()) {
                assert_eq!(env::attached_deposit(), config.deposit.unwrap_or(0), "Wrong deposit");
            }
            let game_to_save =
                match config.first_move {
                    FirstMoveOptions::First => GameToSave::new(
                        account_id.clone(),
                        opponent_id.clone(),
                        reward),

                    FirstMoveOptions::Second => GameToSave::new(
                        opponent_id.clone(),
                        account_id.clone(),
                        reward),

                    FirstMoveOptions::Random => {
                        let seed = near_sdk::env::random_seed();
                        match seed[0] % 2 {
                            0 => GameToSave::new(
                                opponent_id.clone(),
                                account_id.clone(),
                                reward),
                            _ => GameToSave::new(
                                account_id.clone(),
                                opponent_id.clone(),
                                reward)
                        }
                    }
                };

            self.games.insert(&game_id, &game_to_save);

            self.available_games.insert(&game_id, &(account_id.clone(), opponent_id.clone()));

            self.next_game_id += 1;

            self.available_players.remove(&opponent_id);
            self.available_players.remove(&account_id);
            
            self.internal_add_referral(token_id.clone(), &account_id, &referrer_id);
 
            self.internal_update_stats(&token_id, &account_id, UpdateStatsAction::AddPlayedGame, None, None);
            self.internal_update_stats(&token_id, &opponent_id, UpdateStatsAction::AddPlayedGame, None, None);

            game_id
        } else {
            panic!("Your opponent is not ready");
        }
    }

    pub fn draw(&self, game_id: GameId) {
        let game: Game = self.internal_get_game(&game_id).into();
        display::print_board(game.board());
    }

    #[payable]
    pub fn give_up(&mut self, game_id: GameId) {
        //assert one yocto
        assert_eq!(env::attached_deposit(), ONE_YOCTO, "Attach 1 yocto");
        let mut game: GameToSave = self.internal_get_game(&game_id);
        assert!(game.winner_index.is_none(), "Game already finished");
        let account_id = env::predecessor_account_id();

        let player_1 = game.player_1.account_id.clone();
        let player_2 = game.player_2.account_id.clone();

        let (winner_index, winner_account) = if account_id == player_1 {
            (1, player_2)
        } else if account_id == player_2 {
            (0, player_1)
        } else { panic!("No access") };

        self.internal_distribute_reward(&game.reward, &winner_account);
        game.winner_index = Some(winner_index);
        self.games.insert(&game_id, &game);

        self.internal_stop_game(game_id);
    }

    pub fn make_move(&mut self, game_id: GameId, line: String) {
        let mut game: Game = self.internal_get_game(&game_id).into();
        assert!(game.winner_index.is_none(), "Game already finished");

        let mut update_game = false;
        let active_player = game.current_player_account_id();
        assert_eq!(active_player, env::predecessor_account_id(), "No access");

        // display::print_board(game.board());

        let parse_result = input::parse_move(&line);

        match parse_result {
            Ok(positions) => {
                let move_result = util::apply_positions_as_move(&mut game, positions);
                match move_result {
                    Ok(game_state) => match game_state {
                        GameState::InProgress => {
                            update_game = true;
                        }
                        GameState::GameOver { winner_id: winner_index } => {
                            let winner_account = game.players[winner_index].account_id.clone();
                            self.internal_distribute_reward(&game.reward, &winner_account);
                            game.winner_index = Some(winner_index);

                            self.internal_stop_game(game_id);

                            update_game = true;

                            log!("\nGame over! {} won!", winner_account);
                        }
                    },
                    Err(e) => match e {
                        MoveError::InvalidMove => panic!("\n *** Illegal move"),
                        MoveError::ShouldHaveJumped => panic!("\n *** Must take jump")
                    }
                }
            }
            Err(e) => match e {
                InputError::TooFewTokens =>
                    panic!("\n *** You must specify at least two board positions"),
                InputError::InvalidTokens { tokens: errors } => {
                    for error in errors {
                        match error {
                            TokenError::MissingFile { token } =>
                                panic!("\n *** Board position '{}' must specify file", token),
                            TokenError::MissingRank { token } =>
                                panic!("\n *** Board position '{}' must specify rank", token),
                            TokenError::ZeroRank { token } =>
                                panic!("\n *** Rank cannot be zero: {}", token),
                            TokenError::InvalidCharacter { token, char_index } => {
                                let ch = token.chars().nth(char_index).unwrap();
                                panic!("\n *** Board position '{}' contains invalid character '{}'", token, ch);
                            }
                        }
                    }
                }
            }
        }

        if update_game {
            // display::print_board(game.board());
            game.turns += 1;
            let game_to_save: GameToSave = game.into();
            self.games.insert(&game_id, &game_to_save);
        }
    }

    fn internal_stop_game(&mut self, game_id: GameId) {
        self.available_games.remove(&game_id);
    }

    pub fn stop_game(&mut self, game_id: GameId) {
        let mut game: GameToSave = self.internal_get_game(&game_id);
        assert!(game.winner_index.is_none(), "Game already finished");

        let account_id = env::predecessor_account_id();

        let player_1 = game.player_1.account_id.clone();
        let player_2 = game.player_2.account_id.clone();

        let (winner_index, winner_account, looser_account) = if account_id == player_1 {
            let total_spent =
                if game.current_player_index == 0{
                    game.total_time_spent[1]
                }
                else{
                    env::block_timestamp() - game.last_turn_timestamp + game.total_time_spent[1]
                };
            log!("Player {} already spent: {} nanoseconds", player_2, total_spent);
            assert!(total_spent > ONE_HOUR, "Too early to stop the game");

            (0, player_1, player_2)
        } else if account_id == player_2 {
            let total_spent =
                if game.current_player_index == 1{
                    game.total_time_spent[0]
                }
                else{
                    env::block_timestamp() - game.last_turn_timestamp + game.total_time_spent[0]
                };
            log!("Player {} already spent: {} nanoseconds", player_1, total_spent);
            assert!(total_spent > ONE_HOUR, "Too early to stop the game");

            (1, player_2, player_1)
        } else { panic!("No access") };
        let token_balance = &game.reward;
        let token_id = &token_balance.token_id;
        self.internal_update_stats(&token_id, &looser_account,UpdateStatsAction::AddPenaltyGame, None, None);

        self.internal_distribute_reward(token_balance, &winner_account);
        game.winner_index = Some(winner_index);
        self.games.insert(&game_id, &game);

        self.internal_stop_game(game_id);
    }
}
