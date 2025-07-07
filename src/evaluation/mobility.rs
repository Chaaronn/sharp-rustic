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

use crate::{
    board::{
        defs::{Pieces, BB_FILES, BB_SQUARES},
        Board,
    },
    defs::{Bitboard, Side, Sides, Square},
    misc::bits,
    movegen::MoveGenerator,
};

// Mobility bonuses per piece type (indexed by mobility count)
// Separate middle game and endgame values for better evaluation
const KNIGHT_MOBILITY_MG: [i16; 9] = [-30, -20, -10, 0, 10, 20, 25, 30, 32];
const KNIGHT_MOBILITY_EG: [i16; 9] = [-20, -15, -5, 5, 15, 20, 25, 28, 30];

const BISHOP_MOBILITY_MG: [i16; 14] = [-30, -20, -10, 0, 10, 20, 25, 30, 32, 35, 37, 40, 42, 45];
const BISHOP_MOBILITY_EG: [i16; 14] = [-20, -15, -5, 5, 15, 20, 25, 28, 30, 32, 35, 37, 40, 42];

const ROOK_MOBILITY_MG: [i16; 15] = [-30, -20, -10, 0, 5, 10, 15, 20, 25, 30, 32, 35, 37, 40, 42];
const ROOK_MOBILITY_EG: [i16; 15] = [-20, -15, -5, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60];

const QUEEN_MOBILITY_MG: [i16; 28] = [
    -30, -20, -10, 0, 5, 10, 15, 20, 25, 30, 32, 35, 37, 40, 42, 45, 47, 50, 52, 55, 57, 60, 62,
    65, 67, 70, 72, 75,
];
const QUEEN_MOBILITY_EG: [i16; 28] = [
    -20, -15, -5, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95, 100,
    105, 110, 115, 120, 125,
];

// Special bonuses - also split by game phase
const ROOK_OPEN_FILE_BONUS_MG: i16 = 40;
const ROOK_OPEN_FILE_BONUS_EG: i16 = 50;
const ROOK_HALF_OPEN_FILE_BONUS_MG: i16 = 20;
const ROOK_HALF_OPEN_FILE_BONUS_EG: i16 = 25;
const BISHOP_LONG_DIAGONAL_BONUS_MG: i16 = 15;
const BISHOP_LONG_DIAGONAL_BONUS_EG: i16 = 10;

// Game phase calculation
fn calculate_game_phase(board: &Board) -> i16 {
    let mut phase = 0;
    
    // Count material for phase calculation
    for side in [Sides::WHITE, Sides::BLACK] {
        phase += board.get_pieces(Pieces::QUEEN, side).count_ones() as i16 * 4;
        phase += board.get_pieces(Pieces::ROOK, side).count_ones() as i16 * 2;
        phase += board.get_pieces(Pieces::BISHOP, side).count_ones() as i16 * 1;
        phase += board.get_pieces(Pieces::KNIGHT, side).count_ones() as i16 * 1;
    }
    
    // Phase ranges from 0 (endgame) to 24 (opening)
    phase.min(24)
}

pub fn evaluate_mobility(board: &Board, move_gen: &MoveGenerator) -> i16 {
    // Use cached game phase if available, otherwise calculate it
    let game_phase = if board.game_state.game_phase > 0 {
        board.game_state.game_phase
    } else {
        calculate_game_phase(board)
    };
    
    let white_mobility = calculate_side_mobility(board, move_gen, Sides::WHITE, game_phase);
    let black_mobility = calculate_side_mobility(board, move_gen, Sides::BLACK, game_phase);
    
    white_mobility - black_mobility
}

fn calculate_side_mobility(board: &Board, move_gen: &MoveGenerator, side: Side, game_phase: i16) -> i16 {
    let mut mobility_score_mg = 0;
    let mut mobility_score_eg = 0;
    let occupancy = board.occupancy();
    let own_pieces = board.bb_side[side];
    let opponent_pieces = board.bb_side[side ^ 1];
    
    // Calculate opponent attacks for better mobility assessment
    let opponent_attacks = calculate_opponent_attacks(board, move_gen, side ^ 1);
    
    // Knight mobility
    let mut knights = board.get_pieces(Pieces::KNIGHT, side);
    while knights > 0 {
        let square = bits::next(&mut knights);
        let attacks = move_gen.get_non_slider_attacks(Pieces::KNIGHT, square);
        let safe_moves = attacks & !own_pieces & !opponent_attacks;
        let mobility_count = safe_moves.count_ones() as usize;
        
        mobility_score_mg += get_knight_mobility_bonus_mg(mobility_count);
        mobility_score_eg += get_knight_mobility_bonus_eg(mobility_count);
    }
    
    // Bishop mobility
    let mut bishops = board.get_pieces(Pieces::BISHOP, side);
    while bishops > 0 {
        let square = bits::next(&mut bishops);
        let attacks = move_gen.get_slider_attacks(Pieces::BISHOP, square, occupancy);
        let safe_moves = attacks & !own_pieces & !opponent_attacks;
        let mobility_count = safe_moves.count_ones() as usize;
        
        mobility_score_mg += get_bishop_mobility_bonus_mg(mobility_count);
        mobility_score_eg += get_bishop_mobility_bonus_eg(mobility_count);
        
        // Long diagonal bonus
        if is_bishop_on_long_diagonal(square, attacks) {
            mobility_score_mg += BISHOP_LONG_DIAGONAL_BONUS_MG;
            mobility_score_eg += BISHOP_LONG_DIAGONAL_BONUS_EG;
        }
    }
    
    // Rook mobility
    let mut rooks = board.get_pieces(Pieces::ROOK, side);
    while rooks > 0 {
        let square = bits::next(&mut rooks);
        let attacks = move_gen.get_slider_attacks(Pieces::ROOK, square, occupancy);
        let safe_moves = attacks & !own_pieces & !opponent_attacks;
        let mobility_count = safe_moves.count_ones() as usize;
        
        mobility_score_mg += get_rook_mobility_bonus_mg(mobility_count);
        mobility_score_eg += get_rook_mobility_bonus_eg(mobility_count);
        
        // Open/half-open file bonus
        let (mg_bonus, eg_bonus) = evaluate_rook_file_bonus_phased(board, square, side);
        mobility_score_mg += mg_bonus;
        mobility_score_eg += eg_bonus;
    }
    
    // Queen mobility
    let mut queens = board.get_pieces(Pieces::QUEEN, side);
    while queens > 0 {
        let square = bits::next(&mut queens);
        let attacks = move_gen.get_slider_attacks(Pieces::QUEEN, square, occupancy);
        let safe_moves = attacks & !own_pieces & !opponent_attacks;
        let mobility_count = safe_moves.count_ones() as usize;
        
        mobility_score_mg += get_queen_mobility_bonus_mg(mobility_count);
        mobility_score_eg += get_queen_mobility_bonus_eg(mobility_count);
    }
    
    // Interpolate between middle game and endgame scores
    let mg_weight = game_phase;
    let eg_weight = 24 - game_phase;
    
    (mobility_score_mg * mg_weight + mobility_score_eg * eg_weight) / 24
}

// Helper function to calculate opponent attacks
fn calculate_opponent_attacks(board: &Board, move_gen: &MoveGenerator, side: Side) -> Bitboard {
    let mut attacks = 0u64;
    let occupancy = board.occupancy();
    
    // Pawn attacks
    let pawns = board.get_pieces(Pieces::PAWN, side);
    attacks |= if side == Sides::WHITE {
        bits::white_pawn_attacks(pawns)
    } else {
        bits::black_pawn_attacks(pawns)
    };
    
    // Knight attacks
    let mut knights = board.get_pieces(Pieces::KNIGHT, side);
    while knights > 0 {
        let square = bits::next(&mut knights);
        attacks |= move_gen.get_non_slider_attacks(Pieces::KNIGHT, square);
    }
    
    // Bishop attacks
    let mut bishops = board.get_pieces(Pieces::BISHOP, side);
    while bishops > 0 {
        let square = bits::next(&mut bishops);
        attacks |= move_gen.get_slider_attacks(Pieces::BISHOP, square, occupancy);
    }
    
    // Rook attacks
    let mut rooks = board.get_pieces(Pieces::ROOK, side);
    while rooks > 0 {
        let square = bits::next(&mut rooks);
        attacks |= move_gen.get_slider_attacks(Pieces::ROOK, square, occupancy);
    }
    
    // Queen attacks
    let mut queens = board.get_pieces(Pieces::QUEEN, side);
    while queens > 0 {
        let square = bits::next(&mut queens);
        attacks |= move_gen.get_slider_attacks(Pieces::QUEEN, square, occupancy);
    }
    
    // King attacks
    let king_square = board.king_square(side);
    if king_square < 64 {
        attacks |= move_gen.get_non_slider_attacks(Pieces::KING, king_square);
    }
    
    attacks
}

// Updated mobility bonus functions with game phase support
fn get_knight_mobility_bonus_mg(mobility_count: usize) -> i16 {
    if mobility_count < KNIGHT_MOBILITY_MG.len() {
        KNIGHT_MOBILITY_MG[mobility_count]
    } else {
        KNIGHT_MOBILITY_MG[KNIGHT_MOBILITY_MG.len() - 1]
    }
}

fn get_knight_mobility_bonus_eg(mobility_count: usize) -> i16 {
    if mobility_count < KNIGHT_MOBILITY_EG.len() {
        KNIGHT_MOBILITY_EG[mobility_count]
    } else {
        KNIGHT_MOBILITY_EG[KNIGHT_MOBILITY_EG.len() - 1]
    }
}

fn get_bishop_mobility_bonus_mg(mobility_count: usize) -> i16 {
    if mobility_count < BISHOP_MOBILITY_MG.len() {
        BISHOP_MOBILITY_MG[mobility_count]
    } else {
        BISHOP_MOBILITY_MG[BISHOP_MOBILITY_MG.len() - 1]
    }
}

fn get_bishop_mobility_bonus_eg(mobility_count: usize) -> i16 {
    if mobility_count < BISHOP_MOBILITY_EG.len() {
        BISHOP_MOBILITY_EG[mobility_count]
    } else {
        BISHOP_MOBILITY_EG[BISHOP_MOBILITY_EG.len() - 1]
    }
}

fn get_rook_mobility_bonus_mg(mobility_count: usize) -> i16 {
    if mobility_count < ROOK_MOBILITY_MG.len() {
        ROOK_MOBILITY_MG[mobility_count]
    } else {
        ROOK_MOBILITY_MG[ROOK_MOBILITY_MG.len() - 1]
    }
}

fn get_rook_mobility_bonus_eg(mobility_count: usize) -> i16 {
    if mobility_count < ROOK_MOBILITY_EG.len() {
        ROOK_MOBILITY_EG[mobility_count]
    } else {
        ROOK_MOBILITY_EG[ROOK_MOBILITY_EG.len() - 1]
    }
}

fn get_queen_mobility_bonus_mg(mobility_count: usize) -> i16 {
    if mobility_count < QUEEN_MOBILITY_MG.len() {
        QUEEN_MOBILITY_MG[mobility_count]
    } else {
        QUEEN_MOBILITY_MG[QUEEN_MOBILITY_MG.len() - 1]
    }
}

fn get_queen_mobility_bonus_eg(mobility_count: usize) -> i16 {
    if mobility_count < QUEEN_MOBILITY_EG.len() {
        QUEEN_MOBILITY_EG[mobility_count]
    } else {
        QUEEN_MOBILITY_EG[QUEEN_MOBILITY_EG.len() - 1]
    }
}

fn evaluate_rook_file_bonus_phased(board: &Board, rook_square: Square, side: Side) -> (i16, i16) {
    let file = Board::square_on_file_rank(rook_square).0 as usize;
    let file_bb = BB_FILES[file];
    
    let friendly_pawns = board.get_pieces(Pieces::PAWN, side) & file_bb;
    let enemy_pawns = board.get_pieces(Pieces::PAWN, side ^ 1) & file_bb;
    
    if friendly_pawns == 0 && enemy_pawns == 0 {
        // Open file
        (ROOK_OPEN_FILE_BONUS_MG, ROOK_OPEN_FILE_BONUS_EG)
    } else if friendly_pawns == 0 && enemy_pawns > 0 {
        // Half-open file
        (ROOK_HALF_OPEN_FILE_BONUS_MG, ROOK_HALF_OPEN_FILE_BONUS_EG)
    } else {
        (0, 0)
    }
}

fn is_bishop_on_long_diagonal(square: Square, attacks: Bitboard) -> bool {
    // Check if bishop is on one of the long diagonals (a1-h8 or h1-a8)
    let long_diagonal_a1h8 = 0x8040201008040201u64;
    let long_diagonal_h1a8 = 0x0102040810204080u64;
    
    let square_bb = BB_SQUARES[square];
    let on_long_diagonal = (square_bb & (long_diagonal_a1h8 | long_diagonal_h1a8)) > 0;
    
    if on_long_diagonal {
        // Count unobstructed squares on the diagonal
        let diagonal_attacks = attacks & (long_diagonal_a1h8 | long_diagonal_h1a8);
        diagonal_attacks.count_ones() >= 4
    } else {
        false
    }
} 