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
    defs::{Bitboard, Piece, Side, Sides, Square},
    misc::bits,
    movegen::MoveGenerator,
};

// Mobility bonuses per piece type (indexed by mobility count)
const KNIGHT_MOBILITY: [i16; 9] = [-25, -11, -3, 3, 8, 12, 15, 17, 18];
const BISHOP_MOBILITY: [i16; 14] = [-25, -11, -3, 3, 8, 12, 15, 17, 18, 20, 22, 23, 24, 25];
const ROOK_MOBILITY: [i16; 15] = [-25, -11, -3, 3, 8, 12, 15, 17, 18, 20, 22, 23, 24, 25, 26];
const QUEEN_MOBILITY: [i16; 28] = [
    -25, -11, -3, 3, 8, 12, 15, 17, 18, 20, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
    35, 36, 37, 38, 39,
];

// Special bonuses
const ROOK_OPEN_FILE_BONUS: i16 = 30;
const ROOK_HALF_OPEN_FILE_BONUS: i16 = 15;
const BISHOP_LONG_DIAGONAL_BONUS: i16 = 10;

pub fn evaluate_mobility(board: &Board, move_gen: &MoveGenerator) -> i16 {
    let white_mobility = calculate_side_mobility(board, move_gen, Sides::WHITE);
    let black_mobility = calculate_side_mobility(board, move_gen, Sides::BLACK);
    
    white_mobility - black_mobility
}

/// Calculate mobility for both sides and return per-side scores
pub fn evaluate_mobility_per_side(board: &Board, move_gen: &MoveGenerator) -> [i16; 2] {
    let white_mobility = calculate_side_mobility(board, move_gen, Sides::WHITE);
    let black_mobility = calculate_side_mobility(board, move_gen, Sides::BLACK);
    
    [white_mobility, black_mobility]
}

fn calculate_side_mobility(board: &Board, move_gen: &MoveGenerator, side: Side) -> i16 {
    let mut mobility_score = 0;
    let occupancy = board.occupancy();
    let own_pieces = board.bb_side[side];
    let _opponent_pieces = board.bb_side[side ^ 1];
    
    // Knight mobility
    let mut knights = board.get_pieces(Pieces::KNIGHT, side);
    while knights > 0 {
        let square = bits::next(&mut knights);
        let attacks = move_gen.get_non_slider_attacks(Pieces::KNIGHT, square);
        let legal_moves = attacks & !own_pieces;
        let mobility_count = legal_moves.count_ones() as usize;
        mobility_score += get_knight_mobility_bonus(mobility_count);
    }
    
    // Bishop mobility
    let mut bishops = board.get_pieces(Pieces::BISHOP, side);
    while bishops > 0 {
        let square = bits::next(&mut bishops);
        let attacks = move_gen.get_slider_attacks(Pieces::BISHOP, square, occupancy);
        let legal_moves = attacks & !own_pieces;
        let mobility_count = legal_moves.count_ones() as usize;
        mobility_score += get_bishop_mobility_bonus(mobility_count);
        
        // Long diagonal bonus
        if is_bishop_on_long_diagonal(square, attacks) {
            mobility_score += BISHOP_LONG_DIAGONAL_BONUS;
        }
    }
    
    // Rook mobility
    let mut rooks = board.get_pieces(Pieces::ROOK, side);
    while rooks > 0 {
        let square = bits::next(&mut rooks);
        let attacks = move_gen.get_slider_attacks(Pieces::ROOK, square, occupancy);
        let legal_moves = attacks & !own_pieces;
        let mobility_count = legal_moves.count_ones() as usize;
        mobility_score += get_rook_mobility_bonus(mobility_count);
        
        // Open/half-open file bonus
        mobility_score += evaluate_rook_file_bonus(board, square, side);
    }
    
    // Queen mobility
    let mut queens = board.get_pieces(Pieces::QUEEN, side);
    while queens > 0 {
        let square = bits::next(&mut queens);
        let attacks = move_gen.get_slider_attacks(Pieces::QUEEN, square, occupancy);
        let legal_moves = attacks & !own_pieces;
        let mobility_count = legal_moves.count_ones() as usize;
        mobility_score += get_queen_mobility_bonus(mobility_count);
    }
    
    mobility_score
}

fn get_knight_mobility_bonus(mobility_count: usize) -> i16 {
    if mobility_count < KNIGHT_MOBILITY.len() {
        KNIGHT_MOBILITY[mobility_count]
    } else {
        KNIGHT_MOBILITY[KNIGHT_MOBILITY.len() - 1]
    }
}

fn get_bishop_mobility_bonus(mobility_count: usize) -> i16 {
    if mobility_count < BISHOP_MOBILITY.len() {
        BISHOP_MOBILITY[mobility_count]
    } else {
        BISHOP_MOBILITY[BISHOP_MOBILITY.len() - 1]
    }
}

fn get_rook_mobility_bonus(mobility_count: usize) -> i16 {
    if mobility_count < ROOK_MOBILITY.len() {
        ROOK_MOBILITY[mobility_count]
    } else {
        ROOK_MOBILITY[ROOK_MOBILITY.len() - 1]
    }
}

fn get_queen_mobility_bonus(mobility_count: usize) -> i16 {
    if mobility_count < QUEEN_MOBILITY.len() {
        QUEEN_MOBILITY[mobility_count]
    } else {
        QUEEN_MOBILITY[QUEEN_MOBILITY.len() - 1]
    }
}

fn evaluate_rook_file_bonus(board: &Board, rook_square: Square, side: Side) -> i16 {
    let file = Board::square_on_file_rank(rook_square).0 as usize;
    let file_bb = BB_FILES[file];
    
    let friendly_pawns = board.get_pieces(Pieces::PAWN, side) & file_bb;
    let enemy_pawns = board.get_pieces(Pieces::PAWN, side ^ 1) & file_bb;
    
    if friendly_pawns == 0 && enemy_pawns == 0 {
        // Open file
        ROOK_OPEN_FILE_BONUS
    } else if friendly_pawns == 0 && enemy_pawns > 0 {
        // Half-open file
        ROOK_HALF_OPEN_FILE_BONUS
    } else {
        0
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

// === Incremental Mobility Functions ===

/// Calculate mobility for a single piece at a specific square
pub fn calculate_piece_mobility(board: &Board, move_gen: &MoveGenerator, piece: Piece, square: Square, side: Side) -> i16 {
    let occupancy = board.occupancy();
    let own_pieces = board.bb_side[side];
    
    match piece {
        Pieces::KNIGHT => {
            let attacks = move_gen.get_non_slider_attacks(Pieces::KNIGHT, square);
            let legal_moves = attacks & !own_pieces;
            let mobility_count = legal_moves.count_ones() as usize;
            get_knight_mobility_bonus(mobility_count)
        }
        Pieces::BISHOP => {
            let attacks = move_gen.get_slider_attacks(Pieces::BISHOP, square, occupancy);
            let legal_moves = attacks & !own_pieces;
            let mobility_count = legal_moves.count_ones() as usize;
            let mut mobility_score = get_bishop_mobility_bonus(mobility_count);
            
            // Long diagonal bonus
            if is_bishop_on_long_diagonal(square, attacks) {
                mobility_score += BISHOP_LONG_DIAGONAL_BONUS;
            }
            
            mobility_score
        }
        Pieces::ROOK => {
            let attacks = move_gen.get_slider_attacks(Pieces::ROOK, square, occupancy);
            let legal_moves = attacks & !own_pieces;
            let mobility_count = legal_moves.count_ones() as usize;
            let mut mobility_score = get_rook_mobility_bonus(mobility_count);
            
            // Open/half-open file bonus
            mobility_score += evaluate_rook_file_bonus(board, square, side);
            
            mobility_score
        }
        Pieces::QUEEN => {
            let attacks = move_gen.get_slider_attacks(Pieces::QUEEN, square, occupancy);
            let legal_moves = attacks & !own_pieces;
            let mobility_count = legal_moves.count_ones() as usize;
            get_queen_mobility_bonus(mobility_count)
        }
        _ => 0, // King and pawn mobility is not evaluated
    }
}

/// Update mobility incrementally when a piece moves
pub fn update_mobility_incremental(
    board: &Board,
    move_gen: &MoveGenerator,
    mobility_scores: &mut [i16; 2],
    piece: Piece,
    from_square: Square,
    to_square: Square,
    side: Side,
    captured_piece: Option<Piece>,
) {
    // Remove old mobility for the moving piece
    if piece != Pieces::PAWN && piece != Pieces::KING {
        let old_mobility = calculate_piece_mobility(board, move_gen, piece, from_square, side);
        mobility_scores[side] -= old_mobility;
    }
    
    // Add new mobility for the moving piece at its new location
    if piece != Pieces::PAWN && piece != Pieces::KING {
        let new_mobility = calculate_piece_mobility(board, move_gen, piece, to_square, side);
        mobility_scores[side] += new_mobility;
    }
    
    // If a piece was captured, remove its mobility
    if let Some(captured) = captured_piece {
        if captured != Pieces::PAWN && captured != Pieces::KING {
            let captured_mobility = calculate_piece_mobility(board, move_gen, captured, to_square, side ^ 1);
            mobility_scores[side ^ 1] -= captured_mobility;
        }
    }
    
    // Update mobility for pieces affected by the move
    // This is a simplified version - in a full implementation, you'd track
    // which sliding pieces are affected by the move
    update_affected_pieces_mobility(board, move_gen, mobility_scores, from_square, to_square);
}

/// Update mobility for pieces affected by a move (simplified version)
fn update_affected_pieces_mobility(
    board: &Board,
    move_gen: &MoveGenerator,
    mobility_scores: &mut [i16; 2],
    from_square: Square,
    to_square: Square,
) {
    let occupancy = board.occupancy();
    
    // Check all sliding pieces that might be affected by the move
    for side in [Sides::WHITE, Sides::BLACK] {
        // Check bishops and queens on diagonals
        let bishops_and_queens = board.get_pieces(Pieces::BISHOP, side) | board.get_pieces(Pieces::QUEEN, side);
        let mut pieces = bishops_and_queens;
        
        while pieces > 0 {
            let square = bits::next(&mut pieces);
            let attacks = move_gen.get_slider_attacks(Pieces::BISHOP, square, occupancy);
            
            // If this piece's attacks include the from or to square, recalculate its mobility
            if (attacks & (BB_SQUARES[from_square] | BB_SQUARES[to_square])) > 0 {
                let piece_type = if (board.get_pieces(Pieces::BISHOP, side) & BB_SQUARES[square]) > 0 {
                    Pieces::BISHOP
                } else {
                    Pieces::QUEEN
                };
                
                // Remove old mobility and add new mobility
                let old_mobility = calculate_piece_mobility(board, move_gen, piece_type, square, side);
                mobility_scores[side] -= old_mobility;
                
                let new_mobility = calculate_piece_mobility(board, move_gen, piece_type, square, side);
                mobility_scores[side] += new_mobility;
            }
        }
        
        // Check rooks and queens on ranks/files
        let rooks_and_queens = board.get_pieces(Pieces::ROOK, side) | board.get_pieces(Pieces::QUEEN, side);
        let mut pieces = rooks_and_queens;
        
        while pieces > 0 {
            let square = bits::next(&mut pieces);
            let attacks = move_gen.get_slider_attacks(Pieces::ROOK, square, occupancy);
            
            // If this piece's attacks include the from or to square, recalculate its mobility
            if (attacks & (BB_SQUARES[from_square] | BB_SQUARES[to_square])) > 0 {
                let piece_type = if (board.get_pieces(Pieces::ROOK, side) & BB_SQUARES[square]) > 0 {
                    Pieces::ROOK
                } else {
                    Pieces::QUEEN
                };
                
                // Remove old mobility and add new mobility
                let old_mobility = calculate_piece_mobility(board, move_gen, piece_type, square, side);
                mobility_scores[side] -= old_mobility;
                
                let new_mobility = calculate_piece_mobility(board, move_gen, piece_type, square, side);
                mobility_scores[side] += new_mobility;
            }
        }
    }
} 