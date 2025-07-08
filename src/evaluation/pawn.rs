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
    board::{defs::Pieces, Board},
    defs::{Bitboard, Sides},
    misc::bits,
};

// Pawn structure evaluation scores - optimized single values
pub const DOUBLED_PAWN_PENALTY: i16 = -30;
pub const ISOLATED_PAWN_PENALTY: i16 = -35;
pub const BACKWARD_PAWN_PENALTY: i16 = -22;

// Passed pawn bonuses by rank - stronger than before but single values
pub const PASSED_PAWN_BONUS: [i16; 8] = [0, 20, 30, 55, 95, 160, 240, 0];

pub const CONNECTED_PAWN_BONUS: i16 = 10;
pub const PAWN_CHAIN_BONUS: i16 = 6;

// File-specific bonuses/penalties
pub const CENTRAL_PAWN_BONUS: i16 = 6; // For pawns on d/e files
pub const ROOK_FILE_PAWN_PENALTY: i16 = -10; // For pawns on a/h files

/// Comprehensive pawn structure evaluation - optimized for performance
pub fn evaluate_pawn_structure(board: &Board) -> i16 {
    let white_pawns = board.get_pieces(Pieces::PAWN, Sides::WHITE);
    let black_pawns = board.get_pieces(Pieces::PAWN, Sides::BLACK);
    
    let white_score = evaluate_side_pawns(white_pawns, black_pawns, true);
    let black_score = evaluate_side_pawns(black_pawns, white_pawns, false);
    
    white_score - black_score
}

/// Evaluate pawn structure for one side - performance optimized
fn evaluate_side_pawns(own_pawns: Bitboard, enemy_pawns: Bitboard, is_white: bool) -> i16 {
    let mut score = 0i16;
    
    // Doubled pawns analysis using the sophisticated bitboard approach
    let (rear_doubles, front_doubles) = if is_white {
        bits::white_doubled_pawns(own_pawns)
    } else {
        bits::black_doubled_pawns(own_pawns)
    };
    
    let doubled_count = (rear_doubles.count_ones() + front_doubles.count_ones()) as i16;
    score += doubled_count * DOUBLED_PAWN_PENALTY;
    
    // Isolated pawns
    let isolated = bits::isolated_pawns(own_pawns);
    score += isolated.count_ones() as i16 * ISOLATED_PAWN_PENALTY;
    
    // Backward pawns  
    let backward = bits::backward_pawns(own_pawns, enemy_pawns, is_white);
    score += backward.count_ones() as i16 * BACKWARD_PAWN_PENALTY;
    
    // Passed pawns
    let passed = get_passed_pawns(own_pawns, enemy_pawns, is_white);
    score += evaluate_passed_pawns(passed, is_white);
    
    // Connected and chained pawns
    score += evaluate_pawn_connections(own_pawns, is_white);
    
    // File-specific evaluations
    score += evaluate_pawn_files(own_pawns);
    
    score
}

/// Get passed pawns for a side using efficient bitboard operations
fn get_passed_pawns(own_pawns: Bitboard, enemy_pawns: Bitboard, is_white: bool) -> Bitboard {
    if is_white {
        bits::white_passed_pawns(own_pawns, enemy_pawns)
    } else {
        bits::black_passed_pawns(own_pawns, enemy_pawns)
    }
}

/// Evaluate passed pawns with rank-based bonuses
fn evaluate_passed_pawns(passed_pawns: Bitboard, is_white: bool) -> i16 {
    let mut score = 0i16;
    let mut pawns_copy = passed_pawns;
    
    while pawns_copy != 0 {
        let square = bits::next(&mut pawns_copy);
        let rank = square / 8;
        
        // Adjust rank for white/black perspective
        let pawn_rank = if is_white { rank } else { 7 - rank };
        score += PASSED_PAWN_BONUS[pawn_rank];
    }
    
    score
}

/// Evaluate pawn connections and chains
fn evaluate_pawn_connections(pawns: Bitboard, is_white: bool) -> i16 {
    let mut score = 0i16;
    let pawn_attacks = if is_white {
        bits::white_pawn_attacks(pawns)
    } else {
        bits::black_pawn_attacks(pawns)
    };
    
    // Connected pawns: pawns that defend each other
    let connected = pawns & pawn_attacks;
    score += connected.count_ones() as i16 * CONNECTED_PAWN_BONUS;
    
    // Pawn chains: evaluate longer chains more favorably
    let chain_count = count_pawn_chains(pawns, is_white);
    score += chain_count * PAWN_CHAIN_BONUS;
    
    score
}

/// Count pawn chains (connected groups of pawns)
fn count_pawn_chains(pawns: Bitboard, is_white: bool) -> i16 {
    let mut chains = 0i16;
    let mut processed = 0u64;
    let mut pawns_copy = pawns;
    
    while pawns_copy != 0 {
        let square = bits::next(&mut pawns_copy);
        let pawn_bb = 1u64 << square;
        
        if processed & pawn_bb != 0 {
            continue; // Already processed in a chain
        }
        
        // Find connected pawns starting from this pawn
        let chain_pawns = find_connected_pawns(pawn_bb, pawns, is_white);
        
        if chain_pawns.count_ones() >= 2 {
            chains += (chain_pawns.count_ones() - 1) as i16; // Chain bonus scales with length
        }
        
        processed |= chain_pawns;
    }
    
    chains
}

/// Find all pawns connected to the starting pawn
fn find_connected_pawns(start: Bitboard, all_pawns: Bitboard, is_white: bool) -> Bitboard {
    let mut connected = start;
    let mut to_check = start;
    
    loop {
        let pawn_attacks = if is_white {
            bits::white_pawn_attacks(to_check)
        } else {
            bits::black_pawn_attacks(to_check)
        };
        
        let new_connections = (pawn_attacks & all_pawns) & !connected;
        
        if new_connections == 0 {
            break; // No new connections found
        }
        
        connected |= new_connections;
        to_check = new_connections;
    }
    
    connected
}

/// Evaluate pawns based on their files
fn evaluate_pawn_files(pawns: Bitboard) -> i16 {
    let mut score = 0i16;
    
    // Central files (d, e) are valuable
    let d_file_pawns = pawns & crate::board::defs::BB_FILES[3]; // d-file
    let e_file_pawns = pawns & crate::board::defs::BB_FILES[4]; // e-file
    score += (d_file_pawns.count_ones() + e_file_pawns.count_ones()) as i16 * CENTRAL_PAWN_BONUS;
    
    // Rook files (a, h) are less valuable
    let a_file_pawns = pawns & crate::board::defs::BB_FILES[0]; // a-file
    let h_file_pawns = pawns & crate::board::defs::BB_FILES[7]; // h-file
    score += (a_file_pawns.count_ones() + h_file_pawns.count_ones()) as i16 * ROOK_FILE_PAWN_PENALTY;
    
    score
}

/// Get detailed pawn structure info for debugging/analysis
#[allow(dead_code)]
pub fn get_pawn_structure_info(board: &Board) -> PawnStructureInfo {
    let white_pawns = board.get_pieces(Pieces::PAWN, Sides::WHITE);
    let black_pawns = board.get_pieces(Pieces::PAWN, Sides::BLACK);
    
    let (w_rear_doubles, w_front_doubles) = bits::white_doubled_pawns(white_pawns);
    let (b_rear_doubles, b_front_doubles) = bits::black_doubled_pawns(black_pawns);
    
    PawnStructureInfo {
        white_doubled: w_rear_doubles | w_front_doubles,
        black_doubled: b_rear_doubles | b_front_doubles,
        white_isolated: bits::isolated_pawns(white_pawns),
        black_isolated: bits::isolated_pawns(black_pawns),
        white_backward: bits::backward_pawns(white_pawns, black_pawns, true),
        black_backward: bits::backward_pawns(black_pawns, white_pawns, false),
        white_passed: get_passed_pawns(white_pawns, black_pawns, true),
        black_passed: get_passed_pawns(black_pawns, white_pawns, false),
    }
}

/// Detailed pawn structure information for analysis
#[allow(dead_code)]
pub struct PawnStructureInfo {
    pub white_doubled: Bitboard,
    pub black_doubled: Bitboard,
    pub white_isolated: Bitboard,
    pub black_isolated: Bitboard,
    pub white_backward: Bitboard,
    pub black_backward: Bitboard,
    pub white_passed: Bitboard,
    pub black_passed: Bitboard,
} 

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::defs::BB_SQUARES;

    /// Old implementation for comparison in tests
    fn get_passed_pawns_old(own_pawns: Bitboard, enemy_pawns: Bitboard, is_white: bool) -> Bitboard {
        let mut passed = 0u64;
        let mut pawns_copy = own_pawns;
        
        while pawns_copy != 0 {
            let square = bits::next(&mut pawns_copy);
            let file = square % 8;
            let rank = square / 8;
            
            // Check if this pawn has any enemy pawns blocking its path or on adjacent files
            let mut is_passed = true;
            
            // Check the pawn's file and adjacent files for enemy pawns in front
            for check_file in [file.saturating_sub(1), file, (file + 1).min(7)] {
                let file_enemies = enemy_pawns & crate::board::defs::BB_FILES[check_file];
                
                if file_enemies != 0 {
                    let mut enemy_pawns_copy = file_enemies;
                    while enemy_pawns_copy != 0 {
                        let enemy_square = bits::next(&mut enemy_pawns_copy);
                        let enemy_rank = enemy_square / 8;
                        
                        // Check if enemy pawn blocks this pawn's advancement
                        if is_white && enemy_rank > rank {
                            is_passed = false;
                            break;
                        } else if !is_white && enemy_rank < rank {
                            is_passed = false;
                            break;
                        }
                    }
                }
                
                if !is_passed {
                    break;
                }
            }
            
            if is_passed {
                passed |= 1u64 << square;
            }
        }
        
        passed
    }

    #[test]
    fn test_passed_pawns_empty_board() {
        let white_pawns = 0u64;
        let black_pawns = 0u64;
        
        let old_white = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_white = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_white, new_white, "Empty board should produce identical results");
        
        let old_black = get_passed_pawns_old(black_pawns, white_pawns, false);
        let new_black = get_passed_pawns(black_pawns, white_pawns, false);
        assert_eq!(old_black, new_black, "Empty board should produce identical results");
    }

    #[test]
    fn test_passed_pawns_single_pawn() {
        // Single white pawn on d4, no enemy pawns
        let white_pawns = BB_SQUARES[27]; // d4
        let black_pawns = 0u64;
        
        let old_result = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_result = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_result, new_result, "Single passed pawn should be detected identically");
        assert_eq!(new_result, white_pawns, "Single pawn with no enemies should be passed");
    }

    #[test] 
    fn test_passed_pawns_blocked() {
        // White pawn on d4, black pawn on d6 blocking
        let white_pawns = BB_SQUARES[27]; // d4
        let black_pawns = BB_SQUARES[43]; // d6
        
        let old_result = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_result = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_result, new_result, "Blocked pawn should be detected identically");
        assert_eq!(new_result, 0u64, "Blocked pawn should not be passed");
    }

    #[test]
    fn test_passed_pawns_adjacent_file_blocker() {
        // White pawn on d4, black pawn on e6 (adjacent file)
        let white_pawns = BB_SQUARES[27]; // d4
        let black_pawns = BB_SQUARES[44]; // e6
        
        let old_result = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_result = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_result, new_result, "Adjacent file blocker should be detected identically");
        assert_eq!(new_result, 0u64, "Pawn blocked by adjacent file should not be passed");
    }

    #[test]
    fn test_passed_pawns_complex_position() {
        // Multiple pawns in complex position
        let white_pawns = BB_SQUARES[8] | BB_SQUARES[19] | BB_SQUARES[35]; // a2, d3, d5
        let black_pawns = BB_SQUARES[48] | BB_SQUARES[51] | BB_SQUARES[36]; // a7, d7, e5
        
        let old_white = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_white = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_white, new_white, "Complex white position should match");
        
        let old_black = get_passed_pawns_old(black_pawns, white_pawns, false);
        let new_black = get_passed_pawns(black_pawns, white_pawns, false);
        assert_eq!(old_black, new_black, "Complex black position should match");
    }

    #[test]
    fn test_passed_pawns_edge_files() {
        // Test edge cases with a-file and h-file pawns
        let white_pawns = BB_SQUARES[8] | BB_SQUARES[15]; // a2, h2
        let black_pawns = BB_SQUARES[49] | BB_SQUARES[50]; // b7, c7
        
        let old_result = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_result = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_result, new_result, "Edge file pawns should be detected identically");
    }

    #[test]
    fn test_passed_pawns_seventh_rank() {
        // Advanced pawns near promotion
        let white_pawns = BB_SQUARES[51] | BB_SQUARES[52]; // d7, e7
        let black_pawns = BB_SQUARES[20]; // e3
        
        let old_result = get_passed_pawns_old(white_pawns, black_pawns, true);
        let new_result = get_passed_pawns(white_pawns, black_pawns, true);
        assert_eq!(old_result, new_result, "Advanced pawns should be detected identically");
    }

    #[test]
    fn test_all_starting_position_combinations() {
        // Test various combinations from starting positions
        for white_mask in 0..256u64 { // 8 files for white pawns on 2nd rank
            for black_mask in 0..256u64 { // 8 files for black pawns on 7th rank
                let white_pawns = white_mask << 8; // 2nd rank
                let black_pawns = black_mask << 48; // 7th rank
                
                let old_white = get_passed_pawns_old(white_pawns, black_pawns, true);
                let new_white = get_passed_pawns(white_pawns, black_pawns, true);
                assert_eq!(old_white, new_white, 
                    "Starting position variation failed: white={:08b}, black={:08b}", 
                    white_mask, black_mask);
                
                let old_black = get_passed_pawns_old(black_pawns, white_pawns, false);
                let new_black = get_passed_pawns(black_pawns, white_pawns, false);
                assert_eq!(old_black, new_black,
                    "Starting position variation failed: white={:08b}, black={:08b}", 
                    white_mask, black_mask);
            }
        }
    }
} 