use crate::*;
use std::ops::Deref;
use crate::piece::Piece;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TileToSave {
    pub player_id: u32,
    pub piece_type: PieceType
}

pub trait Tile {
    fn get_piece(&self) -> Option<&dyn Piece>;
}

pub struct EmptyTile;

impl Tile for EmptyTile {
    fn get_piece(&self) -> Option<&dyn Piece> {
       Option::None
    }
}

pub struct OccupiedTile {
    piece : Box<dyn Piece>
}

impl OccupiedTile {
    pub fn new( piece : Box<dyn Piece> ) -> OccupiedTile {
        OccupiedTile {
            piece
        }
    }
}

impl Tile for OccupiedTile {
    fn get_piece(&self) -> Option<&dyn Piece> {
       Option::Some(self.piece.deref())
    }
}
