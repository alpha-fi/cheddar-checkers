use piece::ManPiece;
use player::Player;
use tile::{EmptyTile, OccupiedTile, Tile};

use crate::*;
use crate::tile::TileToSave;

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub struct BoardPosition {
    pub row: usize,
    pub column: usize,
}

impl BoardPosition {
    pub fn new(row: usize, column: usize) -> BoardPosition {
        BoardPosition { row, column }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BoardToSave {
    pub(crate) number_rows: usize,
    pub(crate) number_columns: usize,
    pub(crate) tiles: Vec<Option<TileToSave>>,
}

impl From<BoardToSave> for Board {
    fn from(board_to_save: BoardToSave) -> Self {
        let mut board = Board {
            number_rows: CHECKERBOARD_SIZE,
            number_columns: CHECKERBOARD_SIZE,
            tiles: Vec::with_capacity(CHECKERS_NUMBER_TILES),
        };

        for row in 0..board_to_save.number_rows {
            for column in 0..board_to_save.number_columns {
                let idx = board_to_save.indices_to_index(row, column);
                let tile: Box<dyn Tile> =
                    if let Some(tile_to_save) = &board_to_save.tiles[idx] {
                        if tile_to_save.piece_type == PieceType::Man {
                            let piece = ManPiece::new(&Player {
                                id: tile_to_save.player_id
                            });
                            Box::new(OccupiedTile::new(Box::new(piece)))
                        } else if tile_to_save.piece_type == PieceType::King {
                            let piece = KingPiece::new(&Player {
                                id: tile_to_save.player_id
                            });
                            Box::new(OccupiedTile::new(Box::new(piece)))
                        } else {
                            Box::new(EmptyTile)
                        }
                    } else {
                        Box::new(EmptyTile)
                    };

                board.tiles.push(tile);
            }
        }

        board
    }
}

impl BoardToSave {
    pub fn new_checkerboard(player1: &Player, player2: &Player) -> BoardToSave {
        if player1.id == player2.id {
            panic!("Player 1 and Player 2 have the same ID: {}", player1.id)
        }

        let mut board = BoardToSave {
            number_rows: CHECKERBOARD_SIZE,
            number_columns: CHECKERBOARD_SIZE,
            tiles: Vec::with_capacity(CHECKERS_NUMBER_TILES),
        };

        BoardToSave::fill_even_row(&mut board, player1);
        BoardToSave::fill_odd_row(&mut board, player1);
        BoardToSave::fill_even_row(&mut board, player1);

        BoardToSave::fill_empty_row(&mut board);
        BoardToSave::fill_empty_row(&mut board);

        BoardToSave::fill_odd_row(&mut board, player2);
        BoardToSave::fill_even_row(&mut board, player2);
        BoardToSave::fill_odd_row(&mut board, player2);

        board
    }

    fn fill_even_row(board: &mut BoardToSave, player: &Player) {
        for t in 0..board.number_columns {
            let tile: Option<TileToSave> = if t % 2 == 0 {
                Some(TileToSave {
                    player_id: player.id,
                    piece_type: PieceType::Man,
                })
            } else {
                None
            };
            board.tiles.push(tile);
        }
    }

    fn fill_odd_row(board: &mut BoardToSave, player: &Player) {
        for t in 0..board.number_columns {
            let tile: Option<TileToSave> = if t % 2 == 1 {
                Some(TileToSave {
                    player_id: player.id,
                    piece_type: PieceType::Man,
                })
            } else {
                None
            };
            board.tiles.push(tile);
        }
    }

    fn fill_empty_row(board: &mut BoardToSave) {
        for _ in 0..board.number_columns {
            board.tiles.push(None);
        }
    }

    fn indices_to_index(&self, row: usize, column: usize) -> usize {
        self.number_columns * row + column
    }
}

pub struct Board {
    number_rows: usize,
    number_columns: usize,
    tiles: Vec<Box<dyn Tile>>,
}

impl From<Board> for BoardToSave {
    fn from(board: Board) -> Self {
        let mut board_to_save = BoardToSave {
            number_rows: CHECKERBOARD_SIZE,
            number_columns: CHECKERBOARD_SIZE,
            tiles: Vec::with_capacity(CHECKERS_NUMBER_TILES),
        };

        for row in 0..board.number_rows {
            for column in 0..board.number_columns {
                let idx = board.indices_to_index(row, column);

                let tile = &board.tiles[idx];
                let tile_to_save: Option<TileToSave> =
                    match tile.get_piece() {
                        None => None,
                        Some(piece) =>
                            Some(TileToSave {
                                player_id: piece.get_player_id(),
                                piece_type: piece.get_type(),
                            })
                    };

                board_to_save.tiles.push(tile_to_save);
            }
        }

        board_to_save
    }
}

impl From<Board> for BoardOutput {
    fn from(board: Board) -> Self {

        let mut board_output = [
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE),
            Vec::with_capacity(CHECKERBOARD_SIZE)
            ];

        for row in 0..board.number_rows {
            for column in 0..board.number_columns {
                let idx = board.indices_to_index(row, column);

                let tile = &board.tiles[idx];
                let tile_to_save: i8 =
                    match tile.get_piece() {
                        None => 0,
                        Some(piece) =>
                            match piece.get_type() {
                                PieceType::Man => piece.get_player_id() as i8,
                                PieceType::King => piece.get_player_id() as i8 * -1
                            }
                    };

                board_output[row].push(tile_to_save);
            }
        }

        board_output

    }
}

impl Board {
    #[cfg(test)]
    pub fn new(number_rows: usize, number_columns: usize) -> Board {
        let number_tiles = number_rows * number_columns;
        let mut board = Board {
            number_rows: number_rows,
            number_columns: number_columns,
            tiles: Vec::with_capacity(number_tiles),
        };

        for _ in 0..number_tiles {
            board.tiles.push(Box::new(EmptyTile));
        }

        board
    }

    pub fn new_checkerboard(player1: &Player, player2: &Player) -> Board {
        if player1.id == player2.id {
            panic!("Player 1 and Player 2 have the same ID: {}", player1.id)
        }

        let mut board = Board {
            number_rows: CHECKERBOARD_SIZE,
            number_columns: CHECKERBOARD_SIZE,
            tiles: Vec::with_capacity(CHECKERS_NUMBER_TILES),
        };

        Board::fill_even_row(&mut board, player1);
        Board::fill_odd_row(&mut board, player1);
        Board::fill_even_row(&mut board, player1);

        Board::fill_empty_row(&mut board);
        Board::fill_empty_row(&mut board);

        Board::fill_odd_row(&mut board, player2);
        Board::fill_even_row(&mut board, player2);
        Board::fill_odd_row(&mut board, player2);

        board
    }

    pub fn number_rows(&self) -> usize {
        self.number_rows
    }

    pub fn number_columns(&self) -> usize {
        self.number_columns
    }

    fn indices_to_index(&self, row: usize, column: usize) -> usize {
        self.number_columns * row + column
    }

    pub fn get_tile(&self, row: usize, column: usize) -> &dyn Tile {
        let idx = self.indices_to_index(row, column);
        &*self.tiles[idx]
    }

    pub fn set_tile(
        &mut self,
        row: usize,
        column: usize,
        tile: Box<dyn Tile>) {
        let idx = self.indices_to_index(row, column);
        self.tiles[idx] = tile;
    }

    pub fn clear_tile(&mut self, row: usize, column: usize) {
        self.set_tile(row, column, Box::new(EmptyTile));
    }

    pub fn swap_tiles(
        &mut self,
        row1: usize,
        column1: usize,
        row2: usize,
        column2: usize) {
        let idx1 = self.indices_to_index(row1, column1);
        let idx2 = self.indices_to_index(row2, column2);
        self.tiles.swap(idx1, idx2);
    }

    fn fill_even_row(board: &mut Board, player: &Player) {
        for t in 0..board.number_columns {
            let tile: Box<dyn Tile> = if t % 2 == 0 {
                let piece = ManPiece::new(player);
                Box::new(OccupiedTile::new(Box::new(piece)))
            } else {
                Box::new(EmptyTile)
            };
            board.tiles.push(tile);
        }
    }

    fn fill_odd_row(board: &mut Board, player: &Player) {
        for t in 0..board.number_columns {
            let tile: Box<dyn Tile> = if t % 2 == 1 {
                let piece = ManPiece::new(player);
                Box::new(OccupiedTile::new(Box::new(piece)))
            } else {
                Box::new(EmptyTile)
            };
            board.tiles.push(tile);
        }
    }

    fn fill_empty_row(board: &mut Board) {
        for _ in 0..board.number_columns {
            board.tiles.push(Box::new(EmptyTile));
        }
    }
}
