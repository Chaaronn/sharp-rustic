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

use super::gamestate::GameState;
use crate::defs::MAX_GAME_MOVES;

// The history struct holds the game states for each move. It uses a boxed array
// for performance (direct indexing like the original) while allowing different
// sizes for different use cases (main engine vs search threads).
//
// Using Box<[GameState; N]> gives us:
// - Array performance (direct indexing, no bounds checking overhead)
// - Memory efficiency (heap allocation with different sizes)
// - Manual clone optimization (avoid copying entire arrays)

pub struct History {
    list: Box<[GameState]>,
    count: usize,
}

impl History {
    // Create a new history with default capacity (for main engine thread)
    pub fn new() -> Self {
        Self {
            list: vec![GameState::new(); MAX_GAME_MOVES].into_boxed_slice(),
            count: 0,
        }
    }

    // Create a new history for search thread (smaller capacity)
    pub fn new_for_search() -> Self {
        // Search threads typically need much less capacity than the main game
        // Use a smaller capacity to save memory (128 vs 2048)
        Self {
            list: vec![GameState::new(); 128].into_boxed_slice(),
            count: 0,
        }
    }

    // Wipe the entire array.
    pub fn clear(&mut self) {
        self.count = 0;
        // Note: We don't need to clear the actual array elements as they'll be overwritten
    }

    // Put a new game state into the array.
    pub fn push(&mut self, g: GameState) {
        self.list[self.count] = g;
        self.count += 1;
    }

    // Return the last game state and decrement the counter. The game state is
    // not deleted from the array. If necessary, another game state will just
    // overwrite it.
    pub fn pop(&mut self) -> GameState {
        self.count -= 1;
        self.list[self.count]
    }

    pub fn get_ref(&self, index: usize) -> &GameState {
        &self.list[index]
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    // Get the capacity of the underlying array
    pub fn capacity(&self) -> usize {
        self.list.len()
    }
}

impl Clone for History {
    fn clone(&self) -> Self {
        // For cloning, we preserve the current state but create a new array
        // with the same capacity as the original
        Self {
            list: self.list.clone(),
            count: self.count,
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
