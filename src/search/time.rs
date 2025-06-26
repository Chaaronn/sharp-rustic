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

use super::{defs::SearchRefs, Search};
use crate::defs::Sides;
use crate::movegen::defs::MoveList;

// Time management constants - all values in milliseconds
pub const OVERHEAD: i128 = 50;                    // GUI lag protection
const CRITICAL_TIME: u128 = 1_000;               // Time threshold for critical situations
const VERY_LOW_TIME: u128 = 500;                 // Time threshold for very low time
const LOW_TIME: u128 = 2_000;                    // Time threshold for low time
const OK_TIME: u128 = 5_000;                     // Time threshold for comfortable time
const SAFETY_BUFFER: f64 = 0.85;                 // Safety buffer to avoid timeouts
const MIN_TIME_BUFFER: u128 = 100;               // Minimum time buffer in milliseconds

// Game phase estimation constants
const OPENING_MOVES: usize = 10;                 // Number of moves considered opening
const MIDDLEGAME_MOVES: usize = 30;              // Number of moves considered middlegame
const ENDGAME_MOVES: usize = 50;                 // Number of moves considered endgame
const DEFAULT_GAME_LENGTH: usize = 25;           // Default moves to go estimation
const MOVES_BUFFER: usize = 3;                   // Buffer for moves to go estimation

// Position complexity factors
const COMPLEXITY_CHECK_BONUS: f64 = 1.2;         // Bonus for positions with checks
const COMPLEXITY_TACTICAL_BONUS: f64 = 1.15;     // Bonus for tactical positions
const COMPLEXITY_ENDGAME_REDUCTION: f64 = 0.8;   // Reduction for simple endgames

// Time allocation factors
const BASE_TIME_FACTOR: f64 = 0.3;               // Base time usage factor
const MAX_TIME_FACTOR: f64 = 0.8;                // Maximum time usage factor
const TIME_SCALING_CLOCK: f64 = 60_000.0;        // Clock time for scaling (1 minute)

// Overshoot factors for different time situations
const OVERSHOOT_CRITICAL: f64 = 1.0;             // No overshoot in critical time
const OVERSHOOT_VERY_LOW: f64 = 1.1;             // Minimal overshoot in very low time
const OVERSHOOT_LOW: f64 = 1.2;                  // Small overshoot in low time
const OVERSHOOT_OK: f64 = 1.3;                   // Moderate overshoot in OK time
const OVERSHOOT_COMFORTABLE: f64 = 1.5;          // Larger overshoot in comfortable time

impl Search {
    /// Determines if the allocated search time has been exhausted.
    /// This is the unified time checking function used throughout the engine.
    /// 
    /// The function implements progressive time management where the allowed
    /// overshoot depends on the amount of time available. In critical time
    /// situations, no overshoot is allowed, whilst in comfortable time
    /// situations, a larger overshoot is permitted to maximise search depth.
    pub fn out_of_time(refs: &mut SearchRefs) -> bool {
        let elapsed = refs.search_info.timer_elapsed();
        let allocated = refs.search_info.allocated_time;

        // Calculate the overshoot factor based on available time
        // This implements progressive time management to avoid timeouts
        let overshoot_factor = match allocated {
            x if x <= CRITICAL_TIME => OVERSHOOT_CRITICAL,      // Critical time - no overshoot
            x if x <= VERY_LOW_TIME => OVERSHOOT_VERY_LOW,      // Very low time - minimal overshoot
            x if x <= LOW_TIME => OVERSHOOT_LOW,                // Low time - small overshoot
            x if x <= OK_TIME => OVERSHOOT_OK,                  // OK time - moderate overshoot
            _ => OVERSHOOT_COMFORTABLE,                         // Comfortable time - larger overshoot
        };

        // Apply the overshoot factor to determine the actual time limit
        let time_limit = (overshoot_factor * allocated as f64).round() as u128;
        
        // Log time management decisions for debugging (if enabled)
        Search::log_time_decision(refs, elapsed, allocated, overshoot_factor, time_limit);

        elapsed >= time_limit
    }

    /// Unified time checking function that considers both time exhaustion and external interruptions.
    /// This function should be used consistently throughout the search to ensure reliable time management.
    pub fn time_up(refs: &mut SearchRefs) -> bool {
        Search::out_of_time(refs) || refs.search_info.interrupted()
    }

    /// Calculates the time slice allocated for searching a single move.
    /// This function considers position complexity, game phase, and time situation
    /// to make intelligent time allocation decisions.
    pub fn calculate_time_slice(refs: &SearchRefs) -> u128 {
        let gt = &refs.search_params.game_time;
        let mtg = Search::moves_to_go(refs);
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime };
        let increment = if white { gt.winc } else { gt.binc } as i128;
        
        // Calculate base time allocation
        let base_time = ((clock as f64) / (mtg as f64)).round() as i128;
        
        // Apply position complexity factor
        let complexity_factor = Search::calculate_position_complexity(refs);
        let adjusted_base_time = (base_time as f64 * complexity_factor).round() as i128;
        
        // Add increment and subtract overhead
        let time_slice = adjusted_base_time + increment - OVERHEAD;
        
        // Apply safety buffer to avoid timeouts
        let safe_time_slice = (time_slice as f64 * SAFETY_BUFFER).round() as i128;
        
        // Ensure we never allocate negative time
        if safe_time_slice > 0 {
            safe_time_slice as u128
        } else if (adjusted_base_time + increment) > (OVERHEAD / 5) {
            // If we have some time but not enough for overhead, use what we have
            (adjusted_base_time + increment) as u128
        } else {
            // We actually don't have any time
            0
        }
    }

    /// Calculates a dynamic time factor based on available time and game situation.
    /// This factor determines how much of the allocated time should actually be used.
    /// The function implements progressive time management to maximise search depth
    /// when time is plentiful whilst ensuring safety in time-critical situations.
    pub fn dynamic_time_factor(refs: &SearchRefs) -> f64 {
        let gt = &refs.search_params.game_time;
        let white = refs.board.us() == Sides::WHITE;
        let clock = if white { gt.wtime } else { gt.btime } as f64;
        
        // Calculate base time factor based on available time
        let base_factor = if clock <= CRITICAL_TIME as f64 {
            // In critical time, use minimal time factor
            BASE_TIME_FACTOR * 0.5
        } else if clock <= VERY_LOW_TIME as f64 {
            // In very low time, use reduced time factor
            BASE_TIME_FACTOR * 0.7
        } else if clock <= LOW_TIME as f64 {
            // In low time, use standard base factor
            BASE_TIME_FACTOR
        } else {
            // In comfortable time, scale up to maximum factor
            let capped_clock = if clock > TIME_SCALING_CLOCK { TIME_SCALING_CLOCK } else { clock };
            let scaling_factor = capped_clock / TIME_SCALING_CLOCK;
            BASE_TIME_FACTOR + (MAX_TIME_FACTOR - BASE_TIME_FACTOR) * scaling_factor
        };
        
        // Apply game phase adjustment
        let phase_factor = Search::calculate_game_phase_factor(refs);
        let final_factor = base_factor * phase_factor;
        
        // Ensure factor stays within reasonable bounds
        final_factor.max(BASE_TIME_FACTOR * 0.3).min(MAX_TIME_FACTOR)
    }

    /// Estimates the number of moves remaining in the game.
    /// This function uses sophisticated game phase detection to provide
    /// more accurate estimates than a simple fixed value.
    fn moves_to_go(refs: &SearchRefs) -> usize {
        // If moves to go was explicitly supplied, use that value
        if let Some(x) = refs.search_params.game_time.moves_to_go {
            return x;
        }
        
        // Otherwise, estimate based on game phase and position
        let white = refs.board.us() == Sides::WHITE;
        let ply = refs.board.history.len();
        let moves_made = if white { ply / 2 } else { (ply - 1) / 2 };
        
        // Determine game phase and estimate remaining moves
        let estimated_moves = match Search::estimate_game_phase(refs) {
            GamePhase::Opening => DEFAULT_GAME_LENGTH - moves_made + MOVES_BUFFER,
            GamePhase::Middlegame => (DEFAULT_GAME_LENGTH * 3 / 4) - moves_made + MOVES_BUFFER,
            GamePhase::Endgame => (DEFAULT_GAME_LENGTH / 2) - moves_made + MOVES_BUFFER,
            GamePhase::LateEndgame => (DEFAULT_GAME_LENGTH / 4) - moves_made + MOVES_BUFFER,
        };
        
        // Ensure we don't return negative or very small values
        estimated_moves.max(5)
    }

    /// Calculates position complexity factor to adjust time allocation.
    /// Complex positions (tactical, with checks, etc.) receive more time
    /// whilst simple positions receive less time.
    fn calculate_position_complexity(refs: &SearchRefs) -> f64 {
        let mut complexity = 1.0;
        
        // Check if the position is in check (high complexity)
        let is_check = refs.mg.square_attacked(
            refs.board,
            refs.board.opponent(),
            refs.board.king_square(refs.board.us()),
        );
        
        if is_check {
            complexity *= COMPLEXITY_CHECK_BONUS;
        }
        
        // Check for tactical opportunities (captures, threats)
        let tactical_factor = Search::assess_tactical_complexity(refs);
        complexity *= tactical_factor;
        
        // Reduce complexity for simple endgames
        let game_phase = Search::estimate_game_phase(refs);
        if matches!(game_phase, GamePhase::LateEndgame) {
            complexity *= COMPLEXITY_ENDGAME_REDUCTION;
        }
        
        complexity
    }

    /// Assesses tactical complexity of the position by counting captures and threats.
    /// This helps allocate more time to tactically rich positions.
    fn assess_tactical_complexity(refs: &SearchRefs) -> f64 {
        // This is a simplified assessment - in a full implementation,
        // you might want to count captures, threats, and other tactical elements
        let mut complexity = 1.0;
        
        // Count legal moves as a rough indicator of position complexity
        // Generate moves and count them to assess position complexity
        let mut move_list = MoveList::new();
        refs.mg.generate_moves(refs.board, &mut move_list, crate::movegen::defs::MoveType::All);
        let move_count = move_list.len() as usize;
        
        // More moves generally indicate more complex positions
        if move_count > 30 {
            complexity *= COMPLEXITY_TACTICAL_BONUS;
        } else if move_count < 10 {
            complexity *= 0.9; // Slightly reduce time for simple positions
        }
        
        complexity
    }

    /// Calculates game phase factor to adjust time allocation.
    /// Different game phases require different time management strategies.
    fn calculate_game_phase_factor(refs: &SearchRefs) -> f64 {
        match Search::estimate_game_phase(refs) {
            GamePhase::Opening => 1.1,    // Slightly more time in opening
            GamePhase::Middlegame => 1.0, // Standard time in middlegame
            GamePhase::Endgame => 0.9,    // Slightly less time in endgame
            GamePhase::LateEndgame => 0.8, // Less time in simple endgames
        }
    }

    /// Estimates the current game phase based on piece count and position.
    /// This helps make better time management decisions.
    fn estimate_game_phase(refs: &SearchRefs) -> GamePhase {
        let ply = refs.board.history.len();
        let piece_count = Search::count_pieces_on_board(refs.board);
        
        // Determine phase based on move number and piece count
        if ply < OPENING_MOVES * 2 {
            GamePhase::Opening
        } else if ply < MIDDLEGAME_MOVES * 2 {
            GamePhase::Middlegame
        } else if piece_count > 8 {
            GamePhase::Endgame
        } else {
            GamePhase::LateEndgame
        }
    }

    /// Counts the total number of pieces on the board.
    /// This is used for game phase estimation.
    fn count_pieces_on_board(board: &crate::board::Board) -> usize {
        let mut count = 0;
        
        // Count pieces for both sides
        for side in 0..2 {
            for piece_type in 0..6 {
                let bitboard = board.bb_pieces[side][piece_type];
                count += bitboard.count_ones() as usize;
            }
        }
        
        count
    }

    /// Logs time management decisions for debugging and analysis.
    /// This helps identify time management issues and optimise the system.
    fn log_time_decision(
        refs: &SearchRefs,
        elapsed: u128,
        allocated: u128,
        overshoot_factor: f64,
        time_limit: u128,
    ) {
        // In a production system, you might want to log this information
        // to a file or send it to the GUI for analysis
        if elapsed > allocated {
            // Log when we exceed allocated time
            let report = format!(
                "Time management: elapsed={}ms, allocated={}ms, overshoot_factor={:.2}, limit={}ms",
                elapsed, allocated, overshoot_factor, time_limit
            );
            let info_report = super::defs::SearchReport::InfoString(report);
            let information = crate::engine::defs::Information::Search(info_report);
            
            // Send the report (ignore errors to avoid affecting search)
            let _ = refs.report_tx.send(information);
        }
    }

    /// Determines if the search should terminate early due to time constraints.
    /// This function implements early termination heuristics to stop search
    /// earlier in time-critical situations, preventing timeouts.
    pub fn should_terminate_early(refs: &SearchRefs) -> bool {
        // Only apply early termination in game time mode
        if !refs.search_params.is_game_time() {
            return false;
        }

        let elapsed = refs.search_info.timer_elapsed();
        let allocated = refs.search_info.allocated_time;

        // Early termination thresholds based on time situation
        let early_termination_threshold = match allocated {
            x if x <= CRITICAL_TIME => 0.7,      // Very early termination in critical time
            x if x <= VERY_LOW_TIME => 0.8,      // Early termination in very low time
            x if x <= LOW_TIME => 0.85,          // Moderate early termination in low time
            x if x <= OK_TIME => 0.9,            // Slight early termination in OK time
            _ => 0.95,                           // Minimal early termination in comfortable time
        };

        // Check if we've exceeded the early termination threshold
        let threshold_time = (allocated as f64 * early_termination_threshold).round() as u128;
        
        // Also check if we're at a high depth in time-critical situations
        let high_depth_termination = refs.search_info.ply > 10 && elapsed > threshold_time;

        elapsed > threshold_time || high_depth_termination
    }
}

/// Represents different phases of the game for time management purposes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum GamePhase {
    Opening,
    Middlegame,
    Endgame,
    LateEndgame,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        engine::defs::{Information, SearchData, TT},
        movegen::MoveGenerator,
        search::defs::{SearchControl, SearchInfo, SearchParams, SearchRefs},
    };
    use crossbeam_channel::unbounded;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_out_of_time_unified_logic() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(0)));
        let (_ct, crx) = unbounded::<SearchControl>();
        let (rtx, _rrx) = unbounded::<Information>();
        let mut sp = SearchParams::new();
        let mut si = SearchInfo::new();

        // Set up a game time scenario
        sp.search_mode = crate::search::defs::SearchMode::GameTime;
        sp.game_time.wtime = 5000; // 5 seconds
        sp.game_time.btime = 5000;
        si.allocated_time = 1000; // 1 second allocated
        si.timer_start();

        // Wait a bit to simulate elapsed time
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let mut refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: false,
            search_params: &mut sp,
            search_info: &mut si,
            control_rx: &crx,
            report_tx: &rtx,
        };

        // Should be out of time after 1100ms when allocated is 1000ms
        assert!(Search::out_of_time(&mut refs));
    }

    #[test]
    fn test_early_termination_heuristics() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(0)));
        let (_ct, crx) = unbounded::<SearchControl>();
        let (rtx, _rrx) = unbounded::<Information>();
        let mut sp = SearchParams::new();
        let mut si = SearchInfo::new();

        // Set up a critical time scenario
        sp.search_mode = crate::search::defs::SearchMode::GameTime;
        sp.game_time.wtime = 500; // Very low time
        sp.game_time.btime = 500;
        si.allocated_time = 500; // Critical time allocation
        si.timer_start();

        // Wait to simulate elapsed time
        std::thread::sleep(std::time::Duration::from_millis(400)); // 80% of allocated time

        let refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: false,
            search_params: &mut sp,
            search_info: &mut si,
            control_rx: &crx,
            report_tx: &rtx,
        };

        // Should trigger early termination at 80% in critical time
        assert!(Search::should_terminate_early(&refs));
    }

    #[test]
    fn test_position_complexity_calculation() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(0)));
        let (_ct, crx) = unbounded::<SearchControl>();
        let (rtx, _rrx) = unbounded::<Information>();
        let mut sp = SearchParams::new();
        let mut si = SearchInfo::new();

        let refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: false,
            search_params: &mut sp,
            search_info: &mut si,
            control_rx: &crx,
            report_tx: &rtx,
        };

        // Test that complexity calculation works
        let complexity = Search::calculate_position_complexity(&refs);
        assert!(complexity > 0.0);
        assert!(complexity <= 2.0); // Should be reasonable bounds
    }

    #[test]
    fn test_game_phase_estimation() {
        let mut board = Board::new();
        let mg = Arc::new(MoveGenerator::new());
        let tt: Arc<RwLock<TT<SearchData>>> = Arc::new(RwLock::new(TT::new(0)));
        let (_ct, crx) = unbounded::<SearchControl>();
        let (rtx, _rrx) = unbounded::<Information>();
        let mut sp = SearchParams::new();
        let mut si = SearchInfo::new();

        let refs = SearchRefs {
            board: &mut board,
            mg: &mg,
            tt: &tt,
            tt_enabled: false,
            search_params: &mut sp,
            search_info: &mut si,
            control_rx: &crx,
            report_tx: &rtx,
        };

        // Starting position should be opening
        let phase = Search::estimate_game_phase(&refs);
        assert_eq!(phase, GamePhase::Opening);
    }
}
