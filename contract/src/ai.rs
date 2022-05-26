use near_sdk::serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::*;
use crate::board::*;
use crate::player::Player;

#[derive(BorshDeserialize, BorshSerialize, Copy, Clone)]
pub enum Direction {
    /// The piece is moving such that its rank is increasing
    IncreasingRank,

    /// The piece is moving such that its rank is decreasing
    DecreasingRank,
}

// A move from one tile to an adjacent diagonal one
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SimpleMove {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
}

impl SimpleMove {
    pub fn new
    (from_row: usize,
     from_column: usize,
     to_row: usize,
     to_column: usize)
     -> SimpleMove {
        SimpleMove {
            from_row,
            from_col: from_column,
            to_row,
            to_col: to_column,
		}
    }

    pub fn from_row(&self) -> usize {
        self.from_row
    }

    pub fn from_column(&self) -> usize {
        self.from_col
    }

    pub fn to_row(&self) -> usize {
        self.to_row
    }

    pub fn to_column(&self) -> usize {
        self.to_col
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JumpMove {
    from_row: usize,
    from_col: usize,
    jumps: Vec<JumpMove>,
}

impl JumpMove {
    fn new(from_row: usize, from_col: usize) -> JumpMove {
        JumpMove { from_row, from_col, jumps: Vec::new() }
    }

    #[cfg(test)]
    fn with_jumps(from_row: usize, from_col: usize, jumps: Vec<JumpMove>) -> JumpMove {
        JumpMove { from_row, from_col, jumps }
    }

    pub fn jumps(&self) -> &Vec<JumpMove> {
        &self.jumps
    }

    pub fn contains_jump_sequence(&self, jumps: &[BoardPosition]) -> bool {
        if jumps.len() == 0 {
            true
        } else {
            self.contains_jump_sequence_recursive(jumps)
        }
    }

    fn contains_jump_sequence_recursive(&self, jumps: &[BoardPosition]) -> bool {
        if jumps[0].row == self.from_row && jumps[0].column == self.from_col {
            jumps.len() == 1 || self.jumps.iter()
                .any(|subtree| subtree.contains_jump_sequence(&jumps[1..]))
        } else {
            false
        }
    }
}

/// Given the position of a main piece on a board, and the
/// direction this man piece is moving, determines the simple
/// moves available to this piece.
pub fn find_simple_moves_for_man
(board: &Board,
 direction: Direction,
 row: usize,
 col: usize)
 -> Vec<SimpleMove> {
    let row_offset = match direction {
        Direction::DecreasingRank => TileOffset::Negative(1),
        Direction::IncreasingRank => TileOffset::Positive(1),
    };

    let mut moves = Vec::new();

    {
        let col_offset = TileOffset::Negative(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    {
        let col_offset = TileOffset::Positive(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    moves
}

/// Given the position of a main piece on a board, and the
/// direction this man piece is moving, determines the simple
/// moves available to this piece.
pub fn find_jump_moves_for_man
(board: &Board,
 player: &Player,
 direction: Direction,
 row: usize,
 col: usize)
 -> JumpMove {
    let mut jump_root = JumpMove::new(row, col);

    let (pwnd_row_offset, jump_row_offset) = get_row_offsets(direction);

    find_jump_moves_for_man_rustcursive(
        board, player, &pwnd_row_offset, &jump_row_offset, &mut jump_root);

    jump_root
}

fn find_jump_moves_for_man_rustcursive
(board: &Board,
 player: &Player,
 pwnd_row_offset: &TileOffset,
 jump_row_offset: &TileOffset,
 jumps: &mut JumpMove) {
    let col_offset_left = (TileOffset::Negative(1), TileOffset::Negative(2));
    let col_offset_right = (TileOffset::Positive(1), TileOffset::Positive(2));

    try_jump_moves_for_man(
        board, player, &pwnd_row_offset, &jump_row_offset, col_offset_left, jumps);
    try_jump_moves_for_man(
        board, player, &pwnd_row_offset, &jump_row_offset, col_offset_right, jumps);
}

fn try_jump_moves_for_man
(board: &Board,
 player: &Player,
 pwnd_row_offset: &TileOffset,
 jump_row_offset: &TileOffset,
 col_offset: (TileOffset, TileOffset),
 jumps: &mut JumpMove) {
    let start_row = jumps.from_row;
    let start_col = jumps.from_col;
    let (pwnd_col_offset, jump_col_offset) = col_offset;

    let tile_on_board = is_tile_offset_in_bounds(
        board, start_row, start_col, &jump_row_offset, &jump_col_offset);

    if !tile_on_board {
        return;
    }

    let (offset_row, offset_col)
        = offset_tile(start_row, start_col, &pwnd_row_offset, &pwnd_col_offset);
    let pwnd_tile = board.get_tile(offset_row, offset_col);

    let (offset_row, offset_col)
        = offset_tile(start_row, start_col, &jump_row_offset, &jump_col_offset);
    let jump_tile = board.get_tile(offset_row, offset_col);

    if jump_tile.get_piece().is_some() {
        return;
    }

    let pwnd_piece_enemy = pwnd_tile
        .get_piece()
        .map(|piece| piece.get_player_id() != player.id)
        .unwrap_or(false);

    if !pwnd_piece_enemy {
        return;
    }

    let mut the_move = JumpMove::new(offset_row, offset_col);

    find_jump_moves_for_man_rustcursive(
        board, player, &pwnd_row_offset, &jump_row_offset, &mut the_move);

    jumps.jumps.push(the_move);
}

pub fn find_jump_moves_for_king
(board: &Board,
 player: &Player,
 row: usize,
 col: usize)
 -> JumpMove {
    let mut jump_root = JumpMove::new(row, col);
    let mut jumped_tiles = HashSet::new();

    find_jump_moves_for_king_rustcursive(
        board, player, BoardPosition::new(row, col), &mut jump_root, &mut jumped_tiles);

    jump_root
}

fn find_jump_moves_for_king_rustcursive
(board: &Board,
 player: &Player,
 init_position: BoardPosition,
 curr_jump_root: &mut JumpMove,
 jumped_tiles: &mut HashSet<BoardPosition>) {
    push_jump_for_king_if_valid(
        board,
        player,
        init_position,
        curr_jump_root,
        jumped_tiles,
        TileOffset::Negative(1),
        TileOffset::Negative(2),
        TileOffset::Negative(1),
        TileOffset::Negative(2));

    push_jump_for_king_if_valid(
        board,
        player,
        init_position,
        curr_jump_root,
        jumped_tiles,
        TileOffset::Negative(1),
        TileOffset::Negative(2),
        TileOffset::Positive(1),
        TileOffset::Positive(2));

    push_jump_for_king_if_valid(
        board,
        player,
        init_position,
        curr_jump_root,
        jumped_tiles,
        TileOffset::Positive(1),
        TileOffset::Positive(2),
        TileOffset::Negative(1),
        TileOffset::Negative(2));

    push_jump_for_king_if_valid(
        board,
        player,
        init_position,
        curr_jump_root,
        jumped_tiles,
        TileOffset::Positive(1),
        TileOffset::Positive(2),
        TileOffset::Positive(1),
        TileOffset::Positive(2));
}

fn push_jump_for_king_if_valid
(board: &Board,
 player: &Player,
 init_position: BoardPosition,
 curr_jump_root: &mut JumpMove,
 jumped_tiles: &mut HashSet<BoardPosition>,
 pwnd_row_offset: TileOffset,
 jump_row_offset: TileOffset,
 pwnd_col_offset: TileOffset,
 jump_col_offset: TileOffset) {
    let start_row = curr_jump_root.from_row;
    let start_col = curr_jump_root.from_col;

    let tile_on_board = is_tile_offset_in_bounds(
        board, start_row, start_col, &jump_row_offset, &jump_col_offset);
    if !tile_on_board {
        return;
    }

    let (jumped_row, jumped_col)
        = offset_tile(start_row, start_col, &pwnd_row_offset, &pwnd_col_offset);
    let pwnd_tile = board.get_tile(jumped_row, jumped_col);

    let (end_row, end_col)
        = offset_tile(start_row, start_col, &jump_row_offset, &jump_col_offset);
    let end_tile = board.get_tile(end_row, end_col);

    let tile_blocked = end_tile.get_piece().is_some();

    // The initial position of the jumping piece is OK to jump back to. This is because
    // the jumping piece "floats" around the board while the other pieces remain fixed.
    let at_initial_position = init_position == BoardPosition::new(end_row, end_col);
    if tile_blocked && !at_initial_position {
        return;
    }

    let pwnd_piece_enemy = pwnd_tile
        .get_piece()
        .map(|piece| piece.get_player_id() != player.id)
        .unwrap_or(false);

    if !pwnd_piece_enemy {
        return;
    }

    // check to see if we have already jumped the tile
    let jumped_position = BoardPosition::new(jumped_row, jumped_col);
    if jumped_tiles.contains(&jumped_position) {
        return;
    }

    let mut jump = JumpMove::new(end_row, end_col);

    jumped_tiles.insert(jumped_position);

    find_jump_moves_for_king_rustcursive(board, player, init_position, &mut jump, jumped_tiles);

    jumped_tiles.remove(&jumped_position);

    curr_jump_root.jumps.push(jump);
}

fn get_row_offsets(direction: Direction) -> (TileOffset, TileOffset) {
    let (pwnd_row_offset, jump_row_offset) = match direction {
        Direction::DecreasingRank =>
            (TileOffset::Negative(1), TileOffset::Negative(2)),
        Direction::IncreasingRank =>
            (TileOffset::Positive(1), TileOffset::Positive(2))
    };

    (pwnd_row_offset, jump_row_offset)
}

/// Given the position of a king piece on a board, determines
/// the simple moves available to this piece.
///
/// This function does not require a Direction like the
/// find_simple_moves_for_man function, because kings can move
/// in all directions.
pub fn find_simple_moves_for_king
(board: &Board,
 row: usize,
 col: usize)
 -> Vec<SimpleMove> {
    let mut moves = Vec::new();

    {
        let row_offset = TileOffset::Negative(1);
        let col_offset = TileOffset::Negative(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    {
        let row_offset = TileOffset::Negative(1);
        let col_offset = TileOffset::Positive(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    {
        let row_offset = TileOffset::Positive(1);
        let col_offset = TileOffset::Negative(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    {
        let row_offset = TileOffset::Positive(1);
        let col_offset = TileOffset::Positive(1);
        push_simple_move_if_valid(
            board, row, col, &row_offset, &col_offset, &mut moves);
    }

    moves
}

// checks if it is possible to make a simple move with the given row
// and tile offset, and if so, adds the move to the vector
fn push_simple_move_if_valid
(board: &Board,
 start_row: usize,
 start_col: usize,
 row_offset: &TileOffset,
 col_offset: &TileOffset,
 moves: &mut Vec<SimpleMove>) {
    let tile_on_board = is_tile_offset_in_bounds(
        board, start_row, start_col, &row_offset, &col_offset);
    if tile_on_board {
        let (offset_row, offset_col)
            = offset_tile(start_row, start_col, &row_offset, &col_offset);
        let tile = board.get_tile(offset_row, offset_col);
        if tile.get_piece().is_none() {
            let the_move = SimpleMove::new(
                start_row, start_col, offset_row, offset_col);
            moves.push(the_move);
        }
    }
}

// This enum describes an offset direction and magnitude.
enum TileOffset {
    Positive(usize),
    Negative(usize),
}

// offsets a value based on the given offset direction and magnitude
fn offset_value
(start_value: usize, value_offset: &TileOffset)
 -> usize {
    match *value_offset {
        TileOffset::Negative(magnitude) => start_value - magnitude,
        TileOffset::Positive(magnitude) => start_value + magnitude,
    }
}

// Offsets a tile based on the given offset direction
// and magnitude in the row and column dimensions.
//
// Returns a 2 element tuple, where the first element
// is the offset row, and the second element is the
// offset column.
fn offset_tile
(start_row: usize,
 start_col: usize,
 row_offset: &TileOffset,
 col_offset: &TileOffset)
 -> (usize, usize) {
    (offset_value(start_row, row_offset),
     offset_value(start_col, col_offset))
}

// checks if a value is in the given range using the given offset
//TODO maybe a range object can be used here as a param instead
// of the start and max values
fn is_offset_value_in_range
(start_value: usize,
 max_value: usize,
 value_offset: &TileOffset)
 -> bool {
    match *value_offset {
        TileOffset::Negative(magnitude) => start_value >= magnitude,
        TileOffset::Positive(magnitude) => start_value + magnitude <= max_value
    }
}

// checks if a tile on the board can be reached when
// moving from one position on the board to another
fn is_tile_offset_in_bounds
(board: &Board,
 start_row: usize,
 start_col: usize,
 row_offset: &TileOffset,
 col_offset: &TileOffset)
 -> bool {
    let max_row_index = board.number_rows() - 1;
    let max_col_index = board.number_columns() - 1;

    is_offset_value_in_range(start_row, max_row_index, row_offset)
        && is_offset_value_in_range(start_col, max_col_index, col_offset)
}