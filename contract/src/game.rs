// Need the following import statement for compiling
// the tests but not for compiling the application.
use near_sdk::{PanicOnDefault, Timestamp};

use crate::*;
use crate::board::BoardToSave;

#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    /// The game has not yet finished
    InProgress,

    /// The game has finished. The `usize` is the index of the winning player: current_player_index
    GameOver { winner_id: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub enum MoveError {
    InvalidMove,
    ShouldHaveJumped,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault, Clone)]
pub struct PlayerInfo {
    player: Player,
    direction: Direction,
    pub(crate) account_id: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct GameToSave {
    pub(crate) player_1: PlayerInfo,
    pub(crate) player_2: PlayerInfo,
    pub(crate) reward: TokenBalance,
    pub(crate) winner_index: Option<usize>,
    pub(crate) turns: u64,
    pub(crate) last_turn_timestamp: Timestamp,
    pub(crate) total_time_spent: Vec<Timestamp>,

    pub(crate) board: BoardToSave,

    pub(crate) current_player_index: usize,
}

impl From<GameToSave> for Game {
    fn from(game_to_save: GameToSave) -> Self {
        let board: Board = game_to_save.board.into();

        let mut game = Game {
            players: [game_to_save.player_1, game_to_save.player_2],
            reward: game_to_save.reward,
            winner_index: game_to_save.winner_index,
            turns: game_to_save.turns,
            last_turn_timestamp: game_to_save.last_turn_timestamp,
            total_time_spent: game_to_save.total_time_spent,
            board,
            current_player_index: game_to_save.current_player_index,
            available_simple_moves: Vec::new(),
            available_jump_moves: Vec::new(),
        };

        game.find_available_moves();

        game
    }
}

impl GameToSave {
    pub fn new(account_id_1: AccountId, account_id_2: AccountId, reward: TokenBalance) -> GameToSave {
        let (player1, player2) = Game::create_two_players();

        let board: BoardToSave = BoardToSave::new_checkerboard(&player1, &player2);

        GameToSave::with_board_and_players(board, player1, player2, account_id_1, account_id_2, reward)
    }

    fn with_board_and_players(board: BoardToSave, player1: Player, player2: Player,
                              account_id_1: AccountId, account_id_2: AccountId, reward: TokenBalance)
                              -> GameToSave {
        let player1_info = PlayerInfo {
            player: player1,
            direction: Direction::IncreasingRank,
            account_id: account_id_1,
        };
        let player2_info = PlayerInfo {
            player: player2,
            direction: Direction::DecreasingRank,
            account_id: account_id_2,
        };

        GameToSave {
            player_1: player1_info,
            player_2: player2_info,
            reward,
            winner_index: None,
            turns: 0,
            last_turn_timestamp: env::block_timestamp(),
            total_time_spent: [0, 0].to_vec(),
            board,
            current_player_index: 0,
        }
    }
}

pub struct Game {
    pub(crate) players: [PlayerInfo; 2],
    pub(crate) reward: TokenBalance,
    pub(crate) winner_index: Option<usize>,
    pub(crate) turns: u64,
    pub(crate) last_turn_timestamp: Timestamp,
    pub(crate) total_time_spent: Vec<Timestamp>,
    pub(crate) board: Board,

    pub(crate) current_player_index: usize,

    pub(crate) available_simple_moves: Vec<SimpleMove>,
    pub(crate) available_jump_moves: Vec<JumpMove>,
}

impl From<Game> for GameToSave {
    fn from(game: Game) -> Self {
        let board: BoardToSave = game.board.into();

        let already_spent = game.total_time_spent[1 - game.current_player_index];
        let spent_for_this_turn = env::block_timestamp() - game.last_turn_timestamp;

        let mut total_time_spent = game.total_time_spent;
        total_time_spent[1 - game.current_player_index] = already_spent + spent_for_this_turn;

        GameToSave {
            player_1: game.players[0].clone(),
            player_2: game.players[1].clone(),
            reward: game.reward,
            winner_index: game.winner_index,
            turns: game.turns,
            last_turn_timestamp: env::block_timestamp(),
            total_time_spent,
            board,
            current_player_index: game.current_player_index,
        }
    }
}

impl Game {
    pub fn new(account_id_1: AccountId, account_id_2: AccountId, reward: TokenBalance) -> Game {
        let (player1, player2) = Game::create_two_players();

        let board = Board::new_checkerboard(&player1, &player2);

        Game::with_board_and_players(board, player1, player2, account_id_1, account_id_2, reward)
    }

    /// Creates a new Checkers game with an 8x8 board and the specified piece
    /// layout `player1_positions` contains the pieces for the first player,
    /// and `player2_positions` contains the pieces for the second player. All
    /// pieces are initially `ManPiece`s.
    ///
    /// # Panics
    ///
    /// Panics if any element in *player1_positions* or *player2_positions*
    /// is a `BoardPosition` with a row or column outside the range [0, 7].
    /// Also panics if any two positions are exactly the same.
    #[cfg(test)]
    pub fn with_piece_positions(player1_positions: Vec<BoardPosition>,
                                player2_positions: Vec<BoardPosition>)
                                -> Game {
        let checkerboard_size: usize = 8;
        let mut board = Board::new(checkerboard_size, checkerboard_size);

        let (player1, player2) = Game::create_two_players();

        Game::initialize_pieces(&mut board, &player1, &player1_positions);
        Game::initialize_pieces(&mut board, &player2, &player2_positions);

        Game::with_board_and_players(board, player1, player2, "alice".into(),
                                     "bob".into(),
                                     TokenBalance{ token_id: Some("NEAR".into()), balance: 1 })
    }

    // creates and returns two players with distinct IDs
    fn create_two_players() -> (Player, Player) {
        (
            Player { id: 1 },
            Player { id: 2 }
        )
    }

    // adds man pieces belonging to a particular player
    // at the specified positions on a board
    #[cfg(test)]
    fn initialize_pieces(board: &mut Board, player: &Player, positions: &Vec<BoardPosition>) {
        for pos in positions {
            let piece = ManPiece::new(&player);
            let tile = OccupiedTile::new(Box::new(piece));
            assert!(board.get_tile(pos.row, pos.column).get_piece().is_none());
            board.set_tile(pos.row, pos.column, Box::new(tile));
        }
    }

    fn with_board_and_players(board: Board, player1: Player, player2: Player,
                              account_id_1: AccountId, account_id_2: AccountId, reward: TokenBalance)
                              -> Game {
        let player1_info = PlayerInfo {
            player: player1,
            direction: Direction::IncreasingRank,
            account_id: account_id_1,
        };
        let player2_info = PlayerInfo {
            player: player2,
            direction: Direction::DecreasingRank,
            account_id: account_id_2,
        };

        let mut game = Game {
            players: [player1_info, player2_info],
            reward,
            winner_index: None,
            turns: 0,
            last_turn_timestamp: 0,
            total_time_spent: [0, 0].to_vec(),
            board,
            current_player_index: 0,
            available_simple_moves: Vec::new(),
            available_jump_moves: Vec::new(),
        };

        game.find_available_moves();

        game
    }

    fn current_player_info(&self) -> &PlayerInfo {
        &self.players[self.current_player_index]
    }

    fn find_available_simple_moves(&self) -> Vec<SimpleMove> {
        let mut moves = Vec::new();

        let curr_player_info = self.current_player_info();
        let curr_player = &curr_player_info.player;
        let curr_player_id = curr_player.id;
        let curr_direction = curr_player_info.direction;
        for r in 0..self.board.number_rows() {
            for c in 0..self.board.number_columns() {
                match self.board.get_tile(r, c).get_piece() {
                    Some(piece) =>
                        if piece.get_player_id() == curr_player_id {
                            let piece_moves = match piece.get_type() {
                                PieceType::Man =>
                                    ai::find_simple_moves_for_man(
                                        &self.board, curr_direction, r, c),
                                PieceType::King =>
                                    ai::find_simple_moves_for_king(
                                        &self.board, r, c),
                            };
                            moves.extend(piece_moves);
                        },
                    None => {}
                }
            }
        }

        moves
    }

    fn find_available_jump_moves(&self) -> Vec<JumpMove> {
        let mut moves = Vec::new();

        let curr_player_info = self.current_player_info();
        let curr_player = &curr_player_info.player;
        let curr_player_id = curr_player.id;
        let curr_direction = curr_player_info.direction;
        for r in 0..self.board.number_rows() {
            for c in 0..self.board.number_columns() {
                match self.board.get_tile(r, c).get_piece() {
                    Some(piece) =>
                        if piece.get_player_id() == curr_player_id {
                            let jump_move = match piece.get_type() {
                                PieceType::Man =>
                                    ai::find_jump_moves_for_man(
                                        &self.board, curr_player, curr_direction, r, c),
                                PieceType::King =>
                                    ai::find_jump_moves_for_king(
                                        &self.board, curr_player, r, c),
                            };
                            if !jump_move.jumps().is_empty() {
                                moves.push(jump_move);
                            }
                        },
                    None => {}
                }
            }
        }

        moves
    }

    fn find_available_moves(&mut self) {
        self.available_simple_moves = self.find_available_simple_moves();
        self.available_jump_moves = self.find_available_jump_moves();
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn current_player(&self) -> &Player {
        &self.current_player_info().player
    }

    pub fn current_player_account_id(&self) -> AccountId {
        let index = self.current_player_index;
        self.players[index].account_id.clone()
    }

    fn check_for_coronation
    (&mut self, row: usize, col: usize) {
        let coronate = match self.board.get_tile(row, col).get_piece() {
            Some(piece) =>
                match piece.get_type() {
                    PieceType::Man =>
                        match self.current_player_info().direction {
                            Direction::IncreasingRank =>
                                row + 1 == self.board.number_rows(),
                            Direction::DecreasingRank => row == 0
                        },
                    PieceType::King => false
                },
            None => unreachable!()
        };

        if coronate {
            let king = KingPiece::new(&self.current_player_info().player);
            let tile = OccupiedTile::new(Box::new(king));
            self.board.set_tile(row, col, Box::new(tile))
        }
    }

    fn select_next_player(&mut self) {
        // this assumes a two player game
        self.current_player_index = 1 - self.current_player_index;
    }

    fn is_game_over(&self) -> bool {
        // This works if it is called after the available moves for the
        // next player are computed. If this player has no moves, it means
        // they have no pieces left, or all of their pieces are stuck.
        // Either way, they lose.
        self.available_simple_moves.is_empty()
            && self.available_jump_moves.is_empty()
    }

    fn finish_move(&mut self, final_row: usize, final_col: usize)
                   -> GameState {
        self.check_for_coronation(final_row, final_col);

        self.select_next_player();
        self.find_available_moves();

        if self.is_game_over() {
            //let winner_id = self.players[1 - self.current_player_index].player.id;
            GameState::GameOver { winner_id: (1 - self.current_player_index) }
        } else {
            GameState::InProgress
        }
    }

    pub fn apply_simple_move(&mut self, the_move: SimpleMove) -> Result<GameState, MoveError> {
        if self.available_jump_moves.is_empty() {
            if self.available_simple_moves.contains(&the_move) {
                self.board.swap_tiles(
                    the_move.from_row(),
                    the_move.from_column(),
                    the_move.to_row(),
                    the_move.to_column());

                let game_state = self.finish_move(
                    the_move.to_row(), the_move.to_column());
                Ok(game_state)
            } else {
                Err(MoveError::InvalidMove)
            }
        } else {
            Err(MoveError::ShouldHaveJumped)
        }
    }

    pub fn apply_jump_move(&mut self, the_move: Vec<BoardPosition>) -> Result<GameState, MoveError> {
        let jump_valid = self.available_jump_moves.iter()
            .any(|jump_tree| jump_tree.contains_jump_sequence(&the_move));
        if jump_valid {
            let start_position = the_move.first().unwrap();
            let final_position = the_move.last().unwrap();

            // move the jumping piece
            self.board.swap_tiles(
                start_position.row,
                start_position.column,
                final_position.row,
                final_position.column);

            // remove all jumped pieces
            let iter = the_move[0..].iter().zip(the_move[1..].iter());
            for (jump_from_pos, jump_to_pos) in iter {
                let jumped_row = (jump_from_pos.row + jump_to_pos.row) / 2;
                let jumped_col = (jump_from_pos.column + jump_to_pos.column) / 2;
                self.board.clear_tile(jumped_row, jumped_col);
            }

            let game_state = self.finish_move(
                final_position.row, final_position.column);
            Ok(game_state)
        } else {
            Err(MoveError::InvalidMove)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::BoardPosition;
    use crate::PieceType;
    use crate::SimpleMove;

    use super::*;

    fn get_new_game() -> Game {
        Game::new("alice".into(),
                  "bob".into(),
                  TokenBalance{ token_id: Some("NEAR".into()), balance: 1 }
        )
    }

    #[test]
    fn good_simple_move() {
        let mut game = get_new_game();
        let result = game.apply_simple_move(SimpleMove::new(2, 0, 3, 1));
        let exp_result: Result<GameState, MoveError> = Ok(GameState::InProgress);
        assert_eq!(exp_result, result);

        let player_id = game.current_player().id;
        assert_eq!(2, player_id);

        //TODO this test should be more thorough (e.g. check the piece
        // type, player ID, etc.), but it's good enough for now
        assert!(game.board().get_tile(3, 1).get_piece().is_some());
    }

    #[test]
    fn bad_simple_move() {
        let mut game = get_new_game();
        let result = game.apply_simple_move(SimpleMove::new(2, 0, 3, 0));
        let exp_result: Result<GameState, MoveError> = Err(MoveError::InvalidMove);
        assert_eq!(exp_result, result);

        let player_id = game.current_player().id;
        assert_eq!(1, player_id);

        //TODO this test should be more thorough (e.g. check the piece
        // type, player ID, etc.), but it's good enough for now
        assert!(game.board().get_tile(3, 0).get_piece().is_none());
    }

    #[test]
    fn good_single_jump_move() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(3, 3)],
            vec![BoardPosition::new(4, 4), BoardPosition::new(2, 2)]);

        let result = game.apply_jump_move(
            vec![BoardPosition::new(3, 3), BoardPosition::new(5, 5)]);
        let exp_result: Result<GameState, MoveError> = Ok(GameState::InProgress);
        assert_eq!(exp_result, result);

        let jumped_piece = game.board().get_tile(4, 4).get_piece();
        assert!(jumped_piece.is_none());

        let jumping_piece = game.board().get_tile(5, 5).get_piece();
        //TODO this test should be more thorough (e.g. check the piece
        // type, player ID, etc.), but it's good enough for now
        assert!(jumping_piece.is_some());
    }

    #[test]
    fn good_multi_jump_move() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(3, 3)],
            vec![
                BoardPosition::new(4, 4),
                BoardPosition::new(6, 4),
                BoardPosition::new(2, 2)]);

        let result = game.apply_jump_move(
            vec![
                BoardPosition::new(3, 3),
                BoardPosition::new(5, 5),
                BoardPosition::new(7, 3)]);
        let exp_result: Result<GameState, MoveError> = Ok(GameState::InProgress);
        assert_eq!(exp_result, result);

        let jumped_piece1 = game.board().get_tile(4, 4).get_piece();
        assert!(jumped_piece1.is_none());

        let jumped_piece2 = game.board().get_tile(6, 4).get_piece();
        assert!(jumped_piece2.is_none());

        let jumping_piece = game.board().get_tile(7, 3).get_piece();
        //TODO this test should be more thorough (e.g. check the piece
        // type, player ID, etc.), but it's good enough for now
        assert!(jumping_piece.is_some());
    }

    #[test]
    fn bad_jump_move() {
        let mut game = get_new_game();
        let result = game.apply_jump_move(
            vec![BoardPosition::new(2, 0), BoardPosition::new(4, 2)]);
        let exp_result: Result<GameState, MoveError> = Err(MoveError::InvalidMove);
        assert_eq!(exp_result, result);

        let player_id = game.current_player().id;
        assert_eq!(1, player_id);

        //TODO these tests should be more thorough (e.g. check the piece
        // type, player ID, etc.), but it's good enough for now
        assert!(game.board().get_tile(2, 0).get_piece().is_some());
        assert!(game.board().get_tile(4, 2).get_piece().is_none());
    }

    #[test]
    fn not_jumping_when_jump_available() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(3, 3)],
            vec![BoardPosition::new(4, 4)]);

        let result = game.apply_simple_move(SimpleMove::new(3, 3, 4, 2));
        let exp_result: Result<GameState, MoveError> = Err(MoveError::ShouldHaveJumped);
        assert_eq!(exp_result, result);
    }

    #[test]
    fn player1_coronation() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(6, 5)],
            vec![BoardPosition::new(1, 1)]);

        let result = game.apply_simple_move(SimpleMove::new(6, 5, 7, 4));
        let exp_result: Result<GameState, MoveError> = Ok(GameState::InProgress);
        assert_eq!(exp_result, result);

        let piece_type = game.board().get_tile(7, 4).get_piece().unwrap().get_type();
        match piece_type {
            PieceType::King => {}
            _ => panic!("Expected piece to be a King"),
        }
    }

    #[test]
    fn player2_coronation() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(4, 4)],
            vec![BoardPosition::new(1, 1)]);

        game.apply_simple_move(SimpleMove::new(4, 4, 5, 5)).unwrap();
        game.apply_simple_move(SimpleMove::new(1, 1, 0, 0)).unwrap();

        let piece_type = game.board().get_tile(0, 0).get_piece().unwrap().get_type();
        match piece_type {
            PieceType::King => {}
            _ => panic!("Expected piece to be a King"),
        }
    }

    #[test]
    fn game_over_when_no_moves_for_current_player() {
        let mut game = Game::with_piece_positions(
            vec![BoardPosition::new(4, 4)],
            vec![]);

        let result = game.apply_simple_move(SimpleMove::new(4, 4, 5, 5));
        let exp_result: Result<GameState, MoveError> = Ok(GameState::GameOver { winner_id: 1 });
        assert_eq!(exp_result, result);
    }
}
