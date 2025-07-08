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
        defs::{Pieces, BB_FILES},
        Board,
    },
    defs::{Bitboard, Side, Sides, Square},
    misc::bits,
    movegen::MoveGenerator,
};

// King safety evaluation constants - rebalanced for better performance
const MISSING_PAWN_PENALTY: [i16; 4] = [0, 15, 25, 35]; // Penalty for 0, 1, 2, 3 missing pawns
const OPEN_FILE_PENALTY: i16 = 20;
const HALF_OPEN_FILE_PENALTY: i16 = 10;
const PAWN_STORM_PENALTY: i16 = 8;
const WEAK_SQUARES_PENALTY: i16 = 12;

// Attack evaluation constants
const ATTACK_UNIT_WEIGHTS: [i16; 6] = [0, 0, 30, 50, 70, 85]; // For 0-5+ pieces attacking
const SAFE_CHECK_BONUS: i16 = 40;
const UNSAFE_CHECK_BONUS: i16 = 20;

// Piece attack values
const KNIGHT_ATTACK_VALUE: i16 = 15;
const BISHOP_ATTACK_VALUE: i16 = 15;
const ROOK_ATTACK_VALUE: i16 = 25;
const QUEEN_ATTACK_VALUE: i16 = 40;

// Castling zones for pawn shield evaluation
const KINGSIDE_CASTLE_MASK: [Bitboard; 2] = [
    0x00000000000000E0, // White kingside (f1, g1, h1)
    0xE000000000000000, // Black kingside (f8, g8, h8)
];

const QUEENSIDE_CASTLE_MASK: [Bitboard; 2] = [
    0x000000000000001C, // White queenside (c1, d1, e1)
    0x1C00000000000000, // Black queenside (c8, d8, e8)
];

// King zone masks for attack evaluation (squares around king)
const KING_ZONE_MASKS: [Bitboard; 64] = init_king_zone_masks();

// Note: Pawn shield evaluation is done by checking specific squares rather than using masks

pub fn evaluate_king_safety(board: &Board, move_gen: &MoveGenerator) -> i16 {
    let white_safety = calculate_king_safety(board, move_gen, Sides::WHITE);
    let black_safety = calculate_king_safety(board, move_gen, Sides::BLACK);
    
    let raw_score = white_safety - black_safety;
    
    // Apply game phase scaling - king safety matters more in middle game than endgame
    let game_phase_factor = calculate_game_phase_factor(board);
    (raw_score * game_phase_factor) / 100
}

fn calculate_game_phase_factor(board: &Board) -> i16 {
    // Calculate a simple game phase factor based on piece count
    // 100 = full middle game, 50 = endgame
    let mut piece_count = 0;
    
    // Count major and minor pieces (exclude pawns and kings)
    for side in [Sides::WHITE, Sides::BLACK] {
        piece_count += board.get_pieces(Pieces::QUEEN, side).count_ones();
        piece_count += board.get_pieces(Pieces::ROOK, side).count_ones();
        piece_count += board.get_pieces(Pieces::BISHOP, side).count_ones();
        piece_count += board.get_pieces(Pieces::KNIGHT, side).count_ones();
    }
    
    // Scale from 50 (endgame) to 100 (middle game)
    // With 30 pieces at start, we get 100%; with 6 pieces, we get 50%
    let factor = 50 + (piece_count as i16 * 50) / 30;
    factor.min(100).max(50)
}

fn calculate_king_safety(board: &Board, move_gen: &MoveGenerator, side: Side) -> i16 {
    let king_square = board.king_square(side);
    
    // Check if king square is valid (0-63 for chess board)
    if king_square >= 64 {
        return 0; // No king found, return neutral score
    }
    
    let mut safety_score = 0;
    
    // Determine castling status
    let castling_status = determine_castling_status(board, king_square, side);
    
    // Evaluate pawn shield based on castling position
    safety_score += evaluate_pawn_shield(board, king_square, side, castling_status);
    
    // Evaluate open files near king
    safety_score += evaluate_open_files(board, king_square, side);
    
    // Evaluate enemy pawn storms
    safety_score += evaluate_pawn_storm(board, king_square, side);
    
    // Evaluate attacks on king zone
    safety_score += evaluate_king_attacks(board, move_gen, king_square, side);
    
    // Evaluate weak squares around king
    safety_score += evaluate_weak_squares(board, king_square, side);
    
    safety_score
}

#[derive(Debug, Clone, Copy)]
enum CastlingStatus {
    Kingside,
    Queenside,
    Center,
}

fn determine_castling_status(_board: &Board, king_square: Square, side: Side) -> CastlingStatus {
    // Additional safety check
    if king_square >= 64 {
        return CastlingStatus::Center;
    }
    
    let king_bb = 1u64 << king_square;
    
    if (king_bb & KINGSIDE_CASTLE_MASK[side]) != 0 {
        CastlingStatus::Kingside
    } else if (king_bb & QUEENSIDE_CASTLE_MASK[side]) != 0 {
        CastlingStatus::Queenside
    } else {
        CastlingStatus::Center
    }
}

fn evaluate_pawn_shield(board: &Board, king_square: Square, side: Side, castling_status: CastlingStatus) -> i16 {
    // Additional safety check
    if king_square >= 64 {
        return 0;
    }
    
    let friendly_pawns = board.get_pieces(Pieces::PAWN, side);
    let king_file = king_square % 8;
    let mut missing_pawns = 0;
    
    match castling_status {
        CastlingStatus::Kingside => {
            // Count key pawn positions for kingside castling
            if side == Sides::WHITE {
                // Check only g2, h2 for white (most important pawns)
                let g2 = (friendly_pawns & (1u64 << 14)) != 0; // g2
                let h2 = (friendly_pawns & (1u64 << 15)) != 0; // h2
                
                if !g2 { missing_pawns += 1; }
                if !h2 { missing_pawns += 1; }
            } else {
                // Check only g7, h7 for black (most important pawns)
                let g7 = (friendly_pawns & (1u64 << 54)) != 0; // g7
                let h7 = (friendly_pawns & (1u64 << 55)) != 0; // h7
                
                if !g7 { missing_pawns += 1; }
                if !h7 { missing_pawns += 1; }
            }
        },
        CastlingStatus::Queenside => {
            // Count key pawn positions for queenside castling
            if side == Sides::WHITE {
                // Check only b2, c2 for white (most important pawns)
                let b2 = (friendly_pawns & (1u64 << 9)) != 0;  // b2
                let c2 = (friendly_pawns & (1u64 << 10)) != 0; // c2
                
                if !b2 { missing_pawns += 1; }
                if !c2 { missing_pawns += 1; }
            } else {
                // Check only b7, c7 for black (most important pawns)
                let b7 = (friendly_pawns & (1u64 << 49)) != 0; // b7
                let c7 = (friendly_pawns & (1u64 << 50)) != 0; // c7
                
                if !b7 { missing_pawns += 1; }
                if !c7 { missing_pawns += 1; }
            }
        },
        CastlingStatus::Center => {
            // King in center - only check king's file and one adjacent file for basic protection
            let pawn_rank = if side == Sides::WHITE { 1 } else { 6 };
            
            // Check king's file
            let king_file_pawn = pawn_rank * 8 + king_file;
            if (friendly_pawns & (1u64 << king_file_pawn)) == 0 {
                missing_pawns += 1;
            }
            
            // Check one adjacent file (prefer e-file for central king)
            let adjacent_file = if king_file >= 4 { king_file - 1 } else { king_file + 1 };
            if adjacent_file < 8 {
                let adjacent_pawn = pawn_rank * 8 + adjacent_file;
                if (friendly_pawns & (1u64 << adjacent_pawn)) == 0 {
                    missing_pawns += 1;
                }
            }
        }
    }
    
    missing_pawns = missing_pawns.min(3);
    -MISSING_PAWN_PENALTY[missing_pawns]
}

fn evaluate_open_files(board: &Board, king_square: Square, side: Side) -> i16 {
    // Additional safety check
    if king_square >= 64 {
        return 0;
    }
    
    let friendly_pawns = board.get_pieces(Pieces::PAWN, side);
    let enemy_pawns = board.get_pieces(Pieces::PAWN, side ^ 1);
    let king_file = king_square % 8;
    let mut penalty = 0;
    
    // Check king's file and adjacent files
    let files_to_check = [
        (king_file as i32 - 1).max(0) as usize,
        king_file,
        (king_file as i32 + 1).min(7) as usize,
    ];
    
    for &file in &files_to_check {
        let file_mask = BB_FILES[file];
        let friendly_on_file = (friendly_pawns & file_mask) != 0;
        let enemy_on_file = (enemy_pawns & file_mask) != 0;
        
        if !friendly_on_file && !enemy_on_file {
            // Completely open file
            penalty += OPEN_FILE_PENALTY;
        } else if !friendly_on_file && enemy_on_file {
            // Half-open file (dangerous for king)
            penalty += HALF_OPEN_FILE_PENALTY;
        }
    }
    
    -penalty
}

fn evaluate_pawn_storm(board: &Board, king_square: Square, side: Side) -> i16 {
    // Additional safety check
    if king_square >= 64 {
        return 0;
    }
    
    let enemy_pawns = board.get_pieces(Pieces::PAWN, side ^ 1);
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    let mut storm_penalty = 0;
    
    // Check for enemy pawns advancing towards the king
    let files_to_check = [
        (king_file as i32 - 1).max(0) as usize,
        king_file,
        (king_file as i32 + 1).min(7) as usize,
    ];
    
    for &file in &files_to_check {
        let file_mask = BB_FILES[file];
        let enemy_pawns_on_file = enemy_pawns & file_mask;
        
        if enemy_pawns_on_file != 0 {
            // Find the most advanced enemy pawn on this file
            let mut pawns = enemy_pawns_on_file;
            let mut most_advanced_rank = if side == Sides::WHITE { 0 } else { 7 };
            
            while pawns != 0 {
                let pawn_square = bits::next(&mut pawns);
                let pawn_rank = pawn_square / 8;
                
                if side == Sides::WHITE {
                    // For white king, enemy pawns advancing from higher ranks
                    if pawn_rank > most_advanced_rank {
                        most_advanced_rank = pawn_rank;
                    }
                } else {
                    // For black king, enemy pawns advancing from lower ranks
                    if pawn_rank < most_advanced_rank {
                        most_advanced_rank = pawn_rank;
                    }
                }
            }
            
            // Calculate storm penalty based on proximity to king
            let distance = (king_rank as i32 - most_advanced_rank as i32).abs();
            if distance <= 2 {
                storm_penalty += PAWN_STORM_PENALTY * (3 - distance as i16);
            }
        }
    }
    
    -storm_penalty
}

fn evaluate_king_attacks(board: &Board, move_gen: &MoveGenerator, king_square: Square, side: Side) -> i16 {
    // Additional safety check
    if king_square >= 64 {
        return 0;
    }
    
    let king_zone = KING_ZONE_MASKS[king_square];
    let enemy_side = side ^ 1;
    let occupancy = board.occupancy();
    let mut attack_value = 0;
    let mut attacker_count = 0;
    let mut safe_checks = 0;
    let mut unsafe_checks = 0;
    
    // Evaluate knight attacks
    let mut enemy_knights = board.get_pieces(Pieces::KNIGHT, enemy_side);
    while enemy_knights != 0 {
        let knight_square = bits::next(&mut enemy_knights);
        let knight_attacks = move_gen.get_non_slider_attacks(Pieces::KNIGHT, knight_square);
        
        if (knight_attacks & king_zone) != 0 {
            attack_value += KNIGHT_ATTACK_VALUE;
            attacker_count += 1;
        }
        
        // Check for knight checks
        let king_bb = 1u64 << king_square;
        if (knight_attacks & king_bb) != 0 {
            if is_safe_check(board, knight_square, enemy_side) {
                safe_checks += 1;
            } else {
                unsafe_checks += 1;
            }
        }
    }
    
    // Evaluate bishop attacks
    let mut enemy_bishops = board.get_pieces(Pieces::BISHOP, enemy_side);
    while enemy_bishops != 0 {
        let bishop_square = bits::next(&mut enemy_bishops);
        let bishop_attacks = move_gen.get_slider_attacks(Pieces::BISHOP, bishop_square, occupancy);
        
        if (bishop_attacks & king_zone) != 0 {
            attack_value += BISHOP_ATTACK_VALUE;
            attacker_count += 1;
        }
        
        // Check for bishop checks
        let king_bb = 1u64 << king_square;
        if (bishop_attacks & king_bb) != 0 {
            if is_safe_check(board, bishop_square, enemy_side) {
                safe_checks += 1;
            } else {
                unsafe_checks += 1;
            }
        }
    }
    
    // Evaluate rook attacks
    let mut enemy_rooks = board.get_pieces(Pieces::ROOK, enemy_side);
    while enemy_rooks != 0 {
        let rook_square = bits::next(&mut enemy_rooks);
        let rook_attacks = move_gen.get_slider_attacks(Pieces::ROOK, rook_square, occupancy);
        
        if (rook_attacks & king_zone) != 0 {
            attack_value += ROOK_ATTACK_VALUE;
            attacker_count += 1;
        }
        
        // Check for rook checks
        let king_bb = 1u64 << king_square;
        if (rook_attacks & king_bb) != 0 {
            if is_safe_check(board, rook_square, enemy_side) {
                safe_checks += 1;
            } else {
                unsafe_checks += 1;
            }
        }
    }
    
    // Evaluate queen attacks
    let mut enemy_queens = board.get_pieces(Pieces::QUEEN, enemy_side);
    while enemy_queens != 0 {
        let queen_square = bits::next(&mut enemy_queens);
        let queen_attacks = move_gen.get_slider_attacks(Pieces::QUEEN, queen_square, occupancy);
        
        if (queen_attacks & king_zone) != 0 {
            attack_value += QUEEN_ATTACK_VALUE;
            attacker_count += 1;
        }
        
        // Check for queen checks
        let king_bb = 1u64 << king_square;
        if (queen_attacks & king_bb) != 0 {
            if is_safe_check(board, queen_square, enemy_side) {
                safe_checks += 1;
            } else {
                unsafe_checks += 1;
            }
        }
    }
    
    // Apply attack weight based on number of attackers
    let weight_index = attacker_count.min(5);
    let weighted_attack = (attack_value * ATTACK_UNIT_WEIGHTS[weight_index]) / 100;
    
    // Add check bonuses
    let check_bonus = safe_checks * SAFE_CHECK_BONUS + unsafe_checks * UNSAFE_CHECK_BONUS;
    
    -(weighted_attack + check_bonus)
}

fn evaluate_weak_squares(board: &Board, king_square: Square, side: Side) -> i16 {
    // Additional safety check
    if king_square >= 64 {
        return 0;
    }
    
    let friendly_pawns = board.get_pieces(Pieces::PAWN, side);
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    let mut weak_squares = 0;
    
    // Only check the most critical squares around the king (adjacent squares)
    let critical_squares = [
        (king_file as i32 - 1, king_rank as i32 - 1), // SW
        (king_file as i32, king_rank as i32 - 1),     // S
        (king_file as i32 + 1, king_rank as i32 - 1), // SE
        (king_file as i32 - 1, king_rank as i32),     // W
        (king_file as i32 + 1, king_rank as i32),     // E
        (king_file as i32 - 1, king_rank as i32 + 1), // NW
        (king_file as i32, king_rank as i32 + 1),     // N
        (king_file as i32 + 1, king_rank as i32 + 1), // NE
    ];
    
    for &(file, rank) in &critical_squares {
        if file >= 0 && file < 8 && rank >= 0 && rank < 8 {
            let square = (rank * 8 + file) as usize;
            if !can_be_defended_by_pawn(square, friendly_pawns, side) {
                weak_squares += 1;
            }
        }
    }
    
    // Cap the penalty to avoid excessive punishment
    let capped_weak_squares = weak_squares.min(4);
    -capped_weak_squares * WEAK_SQUARES_PENALTY
}

fn is_safe_check(board: &Board, attacker_square: Square, attacker_side: Side) -> bool {
    // A check is "safe" if the attacking piece is defended
    let attacker_bb = 1u64 << attacker_square;
    
    // Simple heuristic: if the attacker is defended by a pawn, it's safer
    let pawns = board.get_pieces(Pieces::PAWN, attacker_side);
    let pawn_attacks = if attacker_side == Sides::WHITE {
        ((pawns & !BB_FILES[0]) >> 9) | ((pawns & !BB_FILES[7]) >> 7)
    } else {
        ((pawns & !BB_FILES[0]) << 7) | ((pawns & !BB_FILES[7]) << 9)
    };
    
    (pawn_attacks & attacker_bb) != 0
}

fn can_be_defended_by_pawn(square: Square, friendly_pawns: Bitboard, side: Side) -> bool {
    let file = square % 8;
    let rank = square / 8;
    
    if side == Sides::WHITE {
        // For white, check if pawns can advance to defend this square
        let defending_files = [
            if file > 0 { Some(file - 1) } else { None },
            if file < 7 { Some(file + 1) } else { None },
        ];
        
        for defending_file in defending_files.iter().flatten() {
            let pawn_rank = rank.saturating_sub(1);
            if pawn_rank >= 1 {
                let pawn_square = pawn_rank * 8 + defending_file;
                if (friendly_pawns & (1u64 << pawn_square)) != 0 {
                    return true;
                }
            }
        }
    } else {
        // For black, check if pawns can advance to defend this square
        let defending_files = [
            if file > 0 { Some(file - 1) } else { None },
            if file < 7 { Some(file + 1) } else { None },
        ];
        
        for defending_file in defending_files.iter().flatten() {
            let pawn_rank = (rank + 1).min(6);
            if pawn_rank <= 6 {
                let pawn_square = pawn_rank * 8 + defending_file;
                if (friendly_pawns & (1u64 << pawn_square)) != 0 {
                    return true;
                }
            }
        }
    }
    
    false
}

// Initialize king zone masks (3x3 zone around king)
const fn init_king_zone_masks() -> [Bitboard; 64] {
    let mut masks = [0; 64];
    let mut square = 0;
    
    while square < 64 {
        let file = square % 8;
        let rank = square / 8;
        let mut mask = 0;
        
        // Add squares in a 3x3 pattern around the king
        let mut df = -1i32;
        while df <= 1 {
            let mut dr = -1i32;
            while dr <= 1 {
                let new_file = file as i32 + df;
                let new_rank = rank as i32 + dr;
                
                if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
                    let new_square = (new_rank * 8 + new_file) as usize;
                    mask |= 1u64 << new_square;
                }
                
                dr += 1;
            }
            df += 1;
        }
        
        masks[square] = mask;
        square += 1;
    }
    
    masks
} 