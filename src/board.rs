/* =======================================================================
Rustic is a chess playing engine.
Copyright (C) 2019-2024, Marcel Vanthoor
https://rustic-chess.org/

Rustic is written in the Rust programming language. It is an original
work, not derived from any engine that came before it. However, it does
use a lot of concepts which are well-known and are in use by most if not
all classical alpha/beta-based chess engines.

Rustic is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License version 3 as published by
the Free Software Foundation.

Rustic is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
for more details.

You should have received a copy of the GNU General Public License along
with this program.  If not, see <http://www.gnu.org/licenses/>.
======================================================================= */

pub mod defs;
mod fen;
mod gamestate;
mod history;
mod playmove;
mod utils;
mod zobrist;

use self::{
    defs::{Pieces, BB_SQUARES},
    gamestate::GameState,
    history::History,
    zobrist::{ZobristKey, ZobristRandoms},
};
use crate::{
    defs::{Bitboard, NrOf, Piece, Side, Sides, Square, EMPTY},
    evaluation::{pawn, mobility, psqt::{self, FLIP, PSQT_MG}},
    misc::bits,
};
use std::sync::Arc;

// This file implements the engine's board representation; it is bit-board
// based, with the least significant bit being A1.
pub struct Board {
    pub bb_pieces: [[Bitboard; NrOf::PIECE_TYPES]; Sides::BOTH],
    pub bb_side: [Bitboard; Sides::BOTH],
    pub game_state: GameState,
    pub history: History,
    pub piece_list: [Piece; NrOf::SQUARES],
    zr: Arc<ZobristRandoms>,
}

// Public functions for use by other modules.
impl Board {
    // Creates a new board with either the provided FEN, or the starting position.
    pub fn new() -> Self {
        Self {
            bb_pieces: [[EMPTY; NrOf::PIECE_TYPES]; Sides::BOTH],
            bb_side: [EMPTY; Sides::BOTH],
            game_state: GameState::new(),
            history: History::new(),
            piece_list: [Pieces::NONE; NrOf::SQUARES],
            zr: Arc::new(ZobristRandoms::new()),
        }
    }

    // Return a bitboard with locations of a certain piece type for one of the sides.
    pub fn get_pieces(&self, piece: Piece, side: Side) -> Bitboard {
        self.bb_pieces[side][piece]
    }

    // Return a bitboard containing all the pieces on the board.
    pub fn occupancy(&self) -> Bitboard {
        self.bb_side[Sides::WHITE] | self.bb_side[Sides::BLACK]
    }

    // Returns the side to move.
    pub fn us(&self) -> usize {
        self.game_state.active_color as usize
    }

    // Returns the side that is NOT moving.
    pub fn opponent(&self) -> usize {
        (self.game_state.active_color ^ 1) as usize
    }

    // Returns the square the king is currently on.
    pub fn king_square(&self, side: Side) -> Square {
        self.bb_pieces[side][Pieces::KING].trailing_zeros() as Square
    }

    // Remove a piece from the board, for the given side, piece, and square.
    pub fn remove_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] ^= BB_SQUARES[square];
        self.bb_side[side] ^= BB_SQUARES[square];
        self.piece_list[square] = Pieces::NONE;
        self.game_state.zobrist_key ^= self.zr.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        let flip = side == Sides::WHITE;
        let s = if flip { FLIP[square] } else { square };
        self.game_state.psqt[side] -= PSQT_MG[piece][s];
    }

    // Put a piece onto the board, for the given side, piece, and square.
    pub fn put_piece(&mut self, side: Side, piece: Piece, square: Square) {
        self.bb_pieces[side][piece] |= BB_SQUARES[square];
        self.bb_side[side] |= BB_SQUARES[square];
        self.piece_list[square] = piece;
        self.game_state.zobrist_key ^= self.zr.piece(side, piece, square);

        // Incremental updates
        // =============================================================
        let flip = side == Sides::WHITE;
        let s = if flip { FLIP[square] } else { square };
        self.game_state.psqt[side] += PSQT_MG[piece][s];
    }

    // Remove a piece from the from-square, and put it onto the to-square.
    pub fn move_piece(&mut self, side: Side, piece: Piece, from: Square, to: Square) {
        self.remove_piece(side, piece, from);
        self.put_piece(side, piece, to);
    }

    // Set a square as being the current ep-square.
    pub fn set_ep_square(&mut self, square: Square) {
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = Some(square as u8);
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
    }

    // Clear the ep-square. (If the ep-square is None already, nothing changes.)
    pub fn clear_ep_square(&mut self) {
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
        self.game_state.en_passant = None;
        self.game_state.zobrist_key ^= self.zr.en_passant(self.game_state.en_passant);
    }

    // Swap side from WHITE <==> BLACK
    pub fn swap_side(&mut self) {
        self.game_state.zobrist_key ^= self.zr.side(self.game_state.active_color as usize);
        self.game_state.active_color ^= 1;
        self.game_state.zobrist_key ^= self.zr.side(self.game_state.active_color as usize);
    }

    // Update castling permissions and take Zobrist-key into account.
    pub fn update_castling_permissions(&mut self, new_permissions: u8) {
        self.game_state.zobrist_key ^= self.zr.castling(self.game_state.castling);
        self.game_state.castling = new_permissions;
        self.game_state.zobrist_key ^= self.zr.castling(self.game_state.castling);
    }

    // Count total pieces on the board (excluding pawns)
    pub fn piece_count(&self) -> usize {
        let mut count = 0;
        for side in 0..Sides::BOTH {
            for piece in 0..Pieces::PAWN {
                count += self.bb_pieces[side][piece].count_ones() as usize;
            }
        }
        count
    }

    // Count total pieces including pawns
    pub fn total_piece_count(&self) -> usize {
        let mut count = 0;
        for side in 0..Sides::BOTH {
            for piece in 0..NrOf::PIECE_TYPES {
                count += self.bb_pieces[side][piece].count_ones() as usize;
            }
        }
        count
    }

    // Check if the current side is in check
    pub fn in_check(&self) -> bool {
        let king_square = self.king_square(self.us());
        let opponent = self.opponent();
        
        // Check if opponent's pieces attack our king
        for piece in 0..NrOf::PIECE_TYPES {
            let piece_bitboard = self.bb_pieces[opponent][piece];
            if piece_bitboard == 0 {
                continue;
            }
            
            let mut pieces = piece_bitboard;
            while pieces > 0 {
                let from = bits::next(&mut pieces);
                let attacks = self.get_attacks_from(piece, from);
                if (attacks & (1u64 << king_square)) > 0 {
                    return true;
                }
            }
        }
        false
    }

    // Get attacks from a piece on a given square
    fn get_attacks_from(&self, piece: Piece, square: Square) -> Bitboard {
        match piece {
            Pieces::KING => {
                let king_attacks = [
                    (1, 0), (1, 1), (0, 1), (-1, 1),
                    (-1, 0), (-1, -1), (0, -1), (1, -1)
                ];
                let mut attacks = 0u64;
                let file = square % 8;
                let rank = square / 8;
                
                for (df, dr) in king_attacks.iter() {
                    let new_file = file as i8 + df;
                    let new_rank = rank as i8 + dr;
                    if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                        attacks |= 1u64 << (new_rank * 8 + new_file);
                    }
                }
                attacks
            }
            Pieces::KNIGHT => {
                let knight_attacks = [
                    (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                    (1, -2), (1, 2), (2, -1), (2, 1)
                ];
                let mut attacks = 0u64;
                let file = square % 8;
                let rank = square / 8;
                
                for (df, dr) in knight_attacks.iter() {
                    let new_file = file as i8 + df;
                    let new_rank = rank as i8 + dr;
                    if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                        attacks |= 1u64 << (new_rank * 8 + new_file);
                    }
                }
                attacks
            }
            Pieces::PAWN => {
                let side = if self.us() == Sides::WHITE { Sides::WHITE } else { Sides::BLACK };
                let direction = if side == Sides::WHITE { -1 } else { 1 };
                let mut attacks = 0u64;
                let file = square % 8;
                let rank = square / 8;
                
                // Pawn captures
                for df in [-1, 1].iter() {
                    let new_file = file as i8 + df;
                    let new_rank = rank as i8 + direction;
                    if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                        attacks |= 1u64 << (new_rank * 8 + new_file);
                    }
                }
                attacks
            }
            _ => {
                // For sliding pieces, we'll use a simplified approach
                // In a real implementation, you'd use the magic bitboard tables
                0u64
            }
        }
    }

    // === Cache Management Functions ===

    /// Compute pawn hash for cache invalidation
    fn compute_pawn_hash(&self) -> u64 {
        let white_pawns = self.bb_pieces[Sides::WHITE][Pieces::PAWN];
        let black_pawns = self.bb_pieces[Sides::BLACK][Pieces::PAWN];
        
        // Simple hash combining both pawn bitboards
        white_pawns.wrapping_mul(0x517cc1b727220a95) ^ black_pawns.wrapping_mul(0x517cc1b727220a97)
    }

    /// Update the cached pawn structure score
    pub fn update_pawn_structure_cache(&mut self) {
        let current_hash = self.compute_pawn_hash();
        
        // Only recompute if pawn structure changed
        if current_hash != self.game_state.pawn_hash {
            self.game_state.pawn_structure_score = pawn::evaluate_pawn_structure(self);
            self.game_state.pawn_hash = current_hash;
        }
    }



    /// Get cached pawn structure score (update if needed)
    pub fn get_cached_pawn_structure_score(&mut self) -> i16 {
        self.update_pawn_structure_cache();
        self.game_state.pawn_structure_score
    }

    /// Get cached mobility score (update if needed)
    pub fn get_cached_mobility_score(&mut self, move_gen: &crate::movegen::MoveGenerator) -> i16 {
        self.update_mobility_cache(move_gen);
        self.game_state.mobility_score
    }

    /// Initialize all caches (called after board setup)
    pub fn init_evaluation_caches(&mut self, move_gen: &crate::movegen::MoveGenerator) {
        self.game_state.pawn_hash = self.compute_pawn_hash();
        self.game_state.pawn_structure_score = pawn::evaluate_pawn_structure(self);
        self.game_state.game_phase = self.calculate_game_phase();
        self.game_state.mobility_score = mobility::evaluate_mobility(self, move_gen);
    }

    /// Calculate current game phase based on piece material
    pub fn calculate_game_phase(&self) -> i16 {
        let mut phase = 0;
        
        // Count material for phase calculation
        for side in [Sides::WHITE, Sides::BLACK] {
            phase += self.get_pieces(Pieces::QUEEN, side).count_ones() as i16 * 4;
            phase += self.get_pieces(Pieces::ROOK, side).count_ones() as i16 * 2;
            phase += self.get_pieces(Pieces::BISHOP, side).count_ones() as i16 * 1;
            phase += self.get_pieces(Pieces::KNIGHT, side).count_ones() as i16 * 1;
        }
        
        // Phase ranges from 0 (endgame) to 24 (opening)
        phase.min(24)
    }

    /// Update game phase cache (called when pieces are captured/promoted)
    pub fn update_game_phase_cache(&mut self) {
        self.game_state.game_phase = self.calculate_game_phase();
    }

    /// Mark caches as invalid (called when pieces move)
    pub fn invalidate_caches(&mut self) {
        // For pawn structure, we'll let the hash check handle it
        // For mobility, we need to track if any pieces actually moved
        // This is a simplified approach - in a full implementation, you'd track 
        // specific piece movements more efficiently
        
        // Reset mobility cache to mark it as needing recalculation
        // In practice, you could implement more sophisticated invalidation
        // by tracking which pieces moved and only invalidating when necessary
        self.game_state.mobility_score = 0;
        
        // Game phase only changes when pieces are captured, not moved
        // So we don't invalidate it here unless it's a capture
    }

    /// Invalidate caches when pieces are captured (more expensive operation)
    pub fn invalidate_caches_on_capture(&mut self) {
        self.game_state.mobility_score = 0;
        self.update_game_phase_cache();
    }

    /// More efficient cache invalidation - only invalidate specific caches
    pub fn invalidate_mobility_cache(&mut self) {
        self.game_state.mobility_score = 0;
    }

    /// Check if mobility cache is valid
    pub fn is_mobility_cache_valid(&self) -> bool {
        // Simple check - in practice you'd have a more sophisticated validation
        self.game_state.mobility_score != 0
    }

    /// Update the cached mobility score with smarter invalidation
    pub fn update_mobility_cache(&mut self, move_gen: &crate::movegen::MoveGenerator) {
        // Only recompute if cache is invalid
        if !self.is_mobility_cache_valid() {
            self.game_state.mobility_score = mobility::evaluate_mobility(self, move_gen);
        }
    }
}

// Private board functions (for initializating on startup)
impl Board {
    // Resets/wipes the board. Used by the FEN reader function.
    fn reset(&mut self) {
        self.bb_pieces = [[0; NrOf::PIECE_TYPES]; Sides::BOTH];
        self.bb_side = [EMPTY; Sides::BOTH];
        self.game_state = GameState::new();
        self.history.clear();
        self.piece_list = [Pieces::NONE; NrOf::SQUARES];
    }

    // Main initialization function. This is used to initialize the "other"
    // bit-boards that are not set up by the FEN-reader function.
    fn init(&mut self) {
        // Gather all the pieces of a side into one bitboard; one bitboard
        // with all the white pieces, and one with all black pieces.
        let pieces_per_side_bitboards = self.init_pieces_per_side_bitboards();
        self.bb_side[Sides::WHITE] = pieces_per_side_bitboards.0;
        self.bb_side[Sides::BLACK] = pieces_per_side_bitboards.1;

        // Initialize the piece list, zobrist key, and material count. These will
        // later be updated incrementally.
        self.piece_list = self.init_piece_list();
        self.game_state.zobrist_key = self.init_zobrist_key();

        let psqt = psqt::apply(self);
        self.game_state.psqt[Sides::WHITE] = psqt.0;
        self.game_state.psqt[Sides::BLACK] = psqt.1;
    }

    // Gather the pieces for each side into their own bitboard.
    fn init_pieces_per_side_bitboards(&self) -> (Bitboard, Bitboard) {
        let mut bb_white: Bitboard = 0;
        let mut bb_black: Bitboard = 0;

        // Iterate over the bitboards of every piece type.
        for (bb_w, bb_b) in self.bb_pieces[Sides::WHITE]
            .iter()
            .zip(self.bb_pieces[Sides::BLACK].iter())
        {
            bb_white |= *bb_w;
            bb_black |= *bb_b;
        }

        // Return a bitboard with all white pieces, and a bitboard with all
        // black pieces.
        (bb_white, bb_black)
    }

    // Initialize the piece list. This list is used to quickly determine
    // which piece type (rook, knight...) is on a square without having to
    // loop through the piece bitboards.
    fn init_piece_list(&self) -> [Piece; NrOf::SQUARES] {
        let bb_w = self.bb_pieces[Sides::WHITE]; // White piece bitboards
        let bb_b = self.bb_pieces[Sides::BLACK]; // Black piece bitboards
        let mut piece_list: [Piece; NrOf::SQUARES] = [Pieces::NONE; NrOf::SQUARES];

        // piece_type is enumerated, from 0 to 6.
        // 0 = KING, 1 = QUEEN, and so on, as defined in board::defs.
        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            let mut white_pieces = *w; // White pieces of type "piece_type"
            let mut black_pieces = *b; // Black pieces of type "piece_type"

            // Put white pieces into the piece list.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                piece_list[square] = piece_type;
            }

            // Put black pieces into the piece list.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                piece_list[square] = piece_type;
            }
        }

        piece_list
    }

    // Initialize the zobrist hash. This hash will later be updated incrementally.
    fn init_zobrist_key(&self) -> ZobristKey {
        // Keep the key here.
        let mut key: u64 = 0;

        // Same here: "bb_w" is shorthand for
        // "self.bb_pieces[Sides::WHITE]".
        let bb_w = self.bb_pieces[Sides::WHITE];
        let bb_b = self.bb_pieces[Sides::BLACK];

        // Iterate through all piece types, for both white and black.
        // "piece_type" is enumerated, and it'll start at 0 (KING), then 1
        // (QUEEN), and so on.
        for (piece_type, (w, b)) in bb_w.iter().zip(bb_b.iter()).enumerate() {
            // Assume the first iteration; piece_type will be 0 (KING). The
            // following two statements will thus get all the pieces of
            // type "KING" for white and black. (This will obviously only
            // be one king, but with rooks, there will be two in the
            // starting position.)
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            // Iterate through all the piece locations of the current piece
            // type. Get the square the piece is on, and then hash that
            // square/piece combination into the zobrist key.
            while white_pieces > 0 {
                let square = bits::next(&mut white_pieces);
                key ^= self.zr.piece(Sides::WHITE, piece_type, square);
            }

            // Same for black.
            while black_pieces > 0 {
                let square = bits::next(&mut black_pieces);
                key ^= self.zr.piece(Sides::BLACK, piece_type, square);
            }
        }

        // Hash the castling, active color, and en-passant state into the key.
        key ^= self.zr.castling(self.game_state.castling);
        key ^= self.zr.side(self.game_state.active_color as usize);
        key ^= self.zr.en_passant(self.game_state.en_passant);

        // Done; return the key.
        key
    }
}

// Manual Clone implementation for Board to optimize history allocation
impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            bb_pieces: self.bb_pieces,
            bb_side: self.bb_side,
            game_state: self.game_state,
            // Create a fresh history for search threads with smaller capacity
            // This avoids copying the entire history array and saves memory
            history: History::new_for_search(),
            piece_list: self.piece_list,
            zr: Arc::clone(&self.zr), // Reuse the ZobristRandoms
        }
    }
}

// Additional methods for Board to support different cloning strategies
impl Board {
    /// Clone for main engine thread (preserves full history)
    pub fn clone_for_engine(&self) -> Self {
        Self {
            bb_pieces: self.bb_pieces,
            bb_side: self.bb_side,
            game_state: self.game_state,
            history: self.history.clone(), // Full history clone
            piece_list: self.piece_list,
            zr: Arc::clone(&self.zr),
        }
    }

    /// Clone for search thread (fresh history with smaller capacity)
    pub fn clone_for_search(&self) -> Self {
        Self {
            bb_pieces: self.bb_pieces,
            bb_side: self.bb_side,
            game_state: self.game_state,
            history: History::new_for_search(), // Fresh history, smaller capacity
            piece_list: self.piece_list,
            zr: Arc::clone(&self.zr),
        }
    }
}
