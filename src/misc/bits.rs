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

use crate::defs::{Bitboard, Square};
use crate::board::defs::{BB_FILES, BB_RANKS};

// Get the next set bit from a bitboard and unset it. When given a piece
// bitboard, this provides the location/square of the next piece of that type.
pub fn next(bitboard: &mut Bitboard) -> Square {
    let square = bitboard.trailing_zeros() as Square;
    *bitboard ^= 1u64 << square;
    square
}

// === Pawn Structure Bitboard Utilities ===

/// Fill all squares on files that have at least one bit set in the input bitboard
pub fn file_fill(bitboard: Bitboard) -> Bitboard {
    let mut result = 0u64;
    for file in 0..8 {
        if bitboard & BB_FILES[file] != 0 {
            result |= BB_FILES[file];
        }
    }
    result
}

/// Get all squares "north" (higher ranks) of the given bitboard, excluding the original squares
pub fn north_fill(bitboard: Bitboard) -> Bitboard {
    let mut fill = bitboard;
    fill |= fill << 8;
    fill |= fill << 16;
    fill |= fill << 32;
    fill & !bitboard
}

/// Get all squares "south" (lower ranks) of the given bitboard, excluding the original squares  
pub fn south_fill(bitboard: Bitboard) -> Bitboard {
    let mut fill = bitboard;
    fill |= fill >> 8;
    fill |= fill >> 16;
    fill |= fill >> 32;
    fill & !bitboard
}

/// White front spans: all squares in front of white pawns (towards 8th rank)
pub fn white_front_spans(pawns: Bitboard) -> Bitboard {
    north_fill(pawns)
}

/// White rear spans: all squares behind white pawns (towards 1st rank)
pub fn white_rear_spans(pawns: Bitboard) -> Bitboard {
    south_fill(pawns)
}

/// Black front spans: all squares in front of black pawns (towards 1st rank)
pub fn black_front_spans(pawns: Bitboard) -> Bitboard {
    south_fill(pawns)
}

/// Black rear spans: all squares behind black pawns (towards 8th rank)
pub fn black_rear_spans(pawns: Bitboard) -> Bitboard {
    north_fill(pawns)
}

/// White pawns with at least one pawn in front on the same file
pub fn white_pawns_behind_own(pawns: Bitboard) -> Bitboard {
    pawns & white_rear_spans(pawns)
}

/// White pawns with at least one pawn behind on the same file
pub fn white_pawns_in_front_own(pawns: Bitboard) -> Bitboard {
    pawns & white_front_spans(pawns)
}

/// Black pawns with at least one pawn in front on the same file
pub fn black_pawns_behind_own(pawns: Bitboard) -> Bitboard {
    pawns & black_rear_spans(pawns)
}

/// Black pawns with at least one pawn behind on the same file
pub fn black_pawns_in_front_own(pawns: Bitboard) -> Bitboard {
    pawns & black_front_spans(pawns)
}

/// White pawns that have pawns both in front and behind (middle of triple+ pawn chains)
pub fn white_pawns_in_front_and_behind_own(pawns: Bitboard) -> Bitboard {
    white_pawns_in_front_own(pawns) & white_pawns_behind_own(pawns)
}

/// Black pawns that have pawns both in front and behind (middle of triple+ pawn chains)
pub fn black_pawns_in_front_and_behind_own(pawns: Bitboard) -> Bitboard {
    black_pawns_in_front_own(pawns) & black_pawns_behind_own(pawns)
}

/// Get files that contain at least one pawn from the input bitboard
pub fn files_with_pawns(pawns: Bitboard) -> Bitboard {
    file_fill(pawns)
}

/// White doubled pawns analysis - returns (rear_doubles, front_doubles)
pub fn white_doubled_pawns(pawns: Bitboard) -> (Bitboard, Bitboard) {
    let behind_own = white_pawns_behind_own(pawns);
    let in_front_own = white_pawns_in_front_own(pawns);
    let in_front_and_behind = white_pawns_in_front_and_behind_own(pawns);
    
    let files_with_triples = file_fill(in_front_and_behind);
    let rear_doubles = behind_own & !files_with_triples;
    let front_doubles = in_front_own & !files_with_triples;
    
    (rear_doubles, front_doubles)
}

/// Black doubled pawns analysis - returns (rear_doubles, front_doubles)  
pub fn black_doubled_pawns(pawns: Bitboard) -> (Bitboard, Bitboard) {
    let behind_own = black_pawns_behind_own(pawns);
    let in_front_own = black_pawns_in_front_own(pawns);
    let in_front_and_behind = black_pawns_in_front_and_behind_own(pawns);
    
    let files_with_triples = file_fill(in_front_and_behind);
    let rear_doubles = behind_own & !files_with_triples;
    let front_doubles = in_front_own & !files_with_triples;
    
    (rear_doubles, front_doubles)
}

/// Get isolated pawns - pawns with no friendly pawns on adjacent files
pub fn isolated_pawns(pawns: Bitboard) -> Bitboard {
    let mut isolated = 0u64;
    
    for file in 0..8 {
        let file_pawns = pawns & BB_FILES[file];
        if file_pawns != 0 {
            let mut has_neighbours = false;
            
            // Check left adjacent file
            if file > 0 && (pawns & BB_FILES[file - 1]) != 0 {
                has_neighbours = true;
            }
            
            // Check right adjacent file  
            if file < 7 && (pawns & BB_FILES[file + 1]) != 0 {
                has_neighbours = true;
            }
            
            if !has_neighbours {
                isolated |= file_pawns;
            }
        }
    }
    
    isolated
}

/// Get backward pawns - pawns that cannot advance safely and have no pawn support
pub fn backward_pawns(own_pawns: Bitboard, enemy_pawns: Bitboard, is_white: bool) -> Bitboard {
    let mut backward = 0u64;
    
    let (front_spans_fn, enemy_attacks) = if is_white {
        (white_front_spans as fn(Bitboard) -> Bitboard, black_pawn_attacks(enemy_pawns))
    } else {
        (black_front_spans as fn(Bitboard) -> Bitboard, white_pawn_attacks(enemy_pawns))
    };
    
    let mut pawns_copy = own_pawns;
    while pawns_copy != 0 {
        let square = next(&mut pawns_copy);
        let pawn_bb = 1u64 << square;
        
        // Check if pawn can advance safely
        let advance_square = if is_white { 
            if square < 56 { 1u64 << (square + 8) } else { 0 }
        } else {
            if square >= 8 { 1u64 << (square - 8) } else { 0 }
        };
        
        if advance_square != 0 && (advance_square & enemy_attacks) != 0 {
            // Cannot advance safely, check for pawn support
            let file = square % 8;
            let mut has_support = false;
            
            // Check adjacent files for supporting pawns behind this pawn
            for adj_file in [file.saturating_sub(1), (file + 1).min(7)] {
                if adj_file != file {
                    let adj_file_pawns = own_pawns & BB_FILES[adj_file];
                    let support_area = if is_white {
                        white_rear_spans(pawn_bb)
                    } else {
                        black_rear_spans(pawn_bb)
                    };
                    
                    if (adj_file_pawns & support_area) != 0 {
                        has_support = true;
                        break;
                    }
                }
            }
            
            if !has_support {
                backward |= pawn_bb;
            }
        }
    }
    
    backward
}

/// Get white pawn attacks
pub fn white_pawn_attacks(pawns: Bitboard) -> Bitboard {
    let left_attacks = (pawns & !BB_FILES[0]) << 7;  // Not on A-file, attack left
    let right_attacks = (pawns & !BB_FILES[7]) << 9; // Not on H-file, attack right
    left_attacks | right_attacks
}

/// Get black pawn attacks
pub fn black_pawn_attacks(pawns: Bitboard) -> Bitboard {
    let left_attacks = (pawns & !BB_FILES[0]) >> 9;  // Not on A-file, attack left
    let right_attacks = (pawns & !BB_FILES[7]) >> 7; // Not on H-file, attack right
    left_attacks | right_attacks
}

/// Get enemy front spans including adjacent files - used for passed pawn detection
pub fn enemy_front_spans_white(enemy_pawns: Bitboard) -> Bitboard {
    let mut spans = black_front_spans(enemy_pawns); // Enemy pawns advancing towards us
    
    // Include adjacent files for more comprehensive blocking detection
    let left_spans = ((enemy_pawns & !BB_FILES[0]) >> 1) & spans;
    let right_spans = ((enemy_pawns & !BB_FILES[7]) << 1) & spans;
    
    spans | black_front_spans(left_spans | right_spans)
}

/// Get enemy front spans including adjacent files - used for passed pawn detection
pub fn enemy_front_spans_black(enemy_pawns: Bitboard) -> Bitboard {
    let mut spans = white_front_spans(enemy_pawns); // Enemy pawns advancing towards us
    
    // Include adjacent files for more comprehensive blocking detection
    let left_spans = ((enemy_pawns & !BB_FILES[0]) >> 1) & spans;
    let right_spans = ((enemy_pawns & !BB_FILES[7]) << 1) & spans;
    
    spans | white_front_spans(left_spans | right_spans)
}

/// Efficient passed pawn detection for white using bitboard operations
pub fn white_passed_pawns(white_pawns: Bitboard, black_pawns: Bitboard) -> Bitboard {
    // Classic formula: passed pawn has no enemy pawns on its file or adjacent files in front
    
    // Get enemy pawn spans on adjacent files as well  
    let mut enemy_spans = black_front_spans(black_pawns); // Enemy pawns' forward squares
    
    // Include adjacent files by shifting enemy pawns left and right, then getting their spans
    let left_adjacent = (black_pawns & !BB_FILES[0]) >> 1; // Shift left (avoid wrap)
    let right_adjacent = (black_pawns & !BB_FILES[7]) << 1; // Shift right (avoid wrap)
    enemy_spans |= black_front_spans(left_adjacent | right_adjacent);
    
    // Include the enemy pawns themselves as blockers
    enemy_spans |= black_pawns;
    
    // Now find white pawns whose forward spans don't intersect with enemy spans
    let mut passed = 0u64;
    let mut pawns_copy = white_pawns;
    
    while pawns_copy != 0 {
        let square = next(&mut pawns_copy);
        let pawn_bit = 1u64 << square;
        let pawn_forward_span = white_front_spans(pawn_bit);
        
        // If this pawn's forward path is clear of enemy influence, it's passed
        if (pawn_forward_span & enemy_spans) == 0 {
            passed |= pawn_bit;
        }
    }
    
    passed
}

/// Efficient passed pawn detection for black using bitboard operations  
pub fn black_passed_pawns(black_pawns: Bitboard, white_pawns: Bitboard) -> Bitboard {
    // Classic formula: passed pawn has no enemy pawns on its file or adjacent files in front
    
    // Get enemy pawn spans on adjacent files as well  
    let mut enemy_spans = white_front_spans(white_pawns); // Enemy pawns' forward squares
    
    // Include adjacent files by shifting enemy pawns left and right, then getting their spans
    let left_adjacent = (white_pawns & !BB_FILES[0]) >> 1; // Shift left (avoid wrap)
    let right_adjacent = (white_pawns & !BB_FILES[7]) << 1; // Shift right (avoid wrap)
    enemy_spans |= white_front_spans(left_adjacent | right_adjacent);
    
    // Include the enemy pawns themselves as blockers
    enemy_spans |= white_pawns;
    
    // Now find black pawns whose forward spans don't intersect with enemy spans
    let mut passed = 0u64;
    let mut pawns_copy = black_pawns;
    
    while pawns_copy != 0 {
        let square = next(&mut pawns_copy);
        let pawn_bit = 1u64 << square;
        let pawn_forward_span = black_front_spans(pawn_bit);
        
        // If this pawn's forward path is clear of enemy influence, it's passed
        if (pawn_forward_span & enemy_spans) == 0 {
            passed |= pawn_bit;
        }
    }
    
    passed
}
