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

use super::Engine;
use crate::{defs::About, engine::defs::Settings};

// This notice is displayed if the engine is a debug binary. (Debug
// binaries are unoptimized and slower than release binaries.)
#[cfg(debug_assertions)]
const NOTICE_DEBUG_MODE: &str = "Notice: Running in debug mode";

impl Engine {
    pub fn print_ascii_logo(&self) {
        println!();
        println!("d888888b                      dP   oo        ");
        println!("88     88                     88             ");
        println!("88oooo88  88    88  d8888b  d8888P dP d88888b");
        println!("88    88  88    88  8ooooo    88   88 88     ");
        println!("88     88 88    88       88   88   88 88     ");
        println!("88     88  88888P  888888P    dP   dP 888888P");
        println!("ooooooooooooooooooooooooooooooooooooooooooooo");
        println!();
    }

    // Print information about the engine.
    pub fn print_about(&self, s: &Settings) {
        let bits = std::mem::size_of::<usize>() * 8;
        let hash = if s.tt_size == 0 {
            String::from("off")
        } else {
            format!("{} MB", s.tt_size)
        };
        let threads = format!("{}", s.threads);
        let sharp = format!("{}", s.sharp_margin);

        println!("{:<10} {} {}", "Engine:", About::ENGINE, About::VERSION);
        println!("{:<10} {}", "Author:", About::AUTHOR);
        println!("{:<10} {}", "EMail:", About::EMAIL);
        println!("{:<10} {}", "Website:", About::WEBSITE);
        println!("{:<10} {bits}-bit", "Type:");
        println!("{:<10} {hash}", "Hash:");
        println!("{:<10} {threads}", "Threads:");
        println!("{:<10} {sharp}", "SharpMargin:");

        #[cfg(debug_assertions)]
        println!("{NOTICE_DEBUG_MODE}");
    }
}
