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
    defs::{About, FEN_START_POSITION},
    engine::defs::EngineOptionDefaults,
};
use clap::{value_parser, Arg, ArgAction, ArgMatches};

// Consts for command line options, flags and arguments

struct CmdLineArgs {}
impl CmdLineArgs {
    // FEN
    const FEN_LONG: &'static str = "fen";
    const FEN_SHORT: char = 'f';
    const FEN_HELP: &'static str = "Set up the given position";

    // Perft
    const PERFT_LONG: &'static str = "perft";
    const PERFT_SHORT: char = 'p';
    const PERFT_HELP: &'static str = "Run perft to the given depth";
    const PERFT_DEFAULT: i8 = 0;

    // Interface
    const COMM_LONG: &'static str = "comm";
    const COMM_SHORT: char = 'c';
    const COMM_HELP: &'static str = "Select communication protocol to use";
    const COMM_VALUES: [&'static str; 2] = ["uci", "xboard"];
    const COMM_DEFAULT: &'static str = "uci";

    // Threads
    const THREADS_LONG: &'static str = "threads";
    const THREADS_SHORT: char = 't';
    const THREADS_HELP: &'static str = "Number of CPU-threads to use";
    const THREADS_DEFAULT: usize = 1;

    const HASH_LONG: &'static str = "hash";
    const HASH_SHORT: char = 'h';
    const HASH_HELP: &'static str = "Transposition Table size in MB";
    const HASH_DEFAULT: usize = EngineOptionDefaults::HASH_DEFAULT;

    // Quiet (no search stats updates except on depth change)
    const QUIET_LONG: &'static str = "quiet";
    const QUIET_SHORT: char = 'q';
    const QUIET_HELP: &'static str = "No intermediate search stats updates";

    // Kiwipete
    const KIWI_LONG: &'static str = "kiwipete";
    const KIWI_SHORT: char = 'k';
    const KIWI_HELP: &'static str = "Set up KiwiPete position (ignore --fen)";

    // Wizardry
    const WIZARDRY_LONG: &'static str = "wizardry";
    const WIZARDRY_SHORT: char = 'w';
    const WIZARDRY_HELP: &'static str = "Generate magic numbers";

    // Test
    const EPD_TEST_LONG: &'static str = "epdtest";
    const EPD_TEST_SHORT: char = 'e';
    const EPD_TEST_HELP: &'static str = "Run EPD Test Suite";
}

pub struct CmdLine {
    arguments: ArgMatches,
}

impl CmdLine {
    pub fn new() -> Self {
        Self {
            arguments: Self::get(),
        }
    }

    pub fn comm(&self) -> String {
        self.arguments
            .get_one::<String>(CmdLineArgs::COMM_LONG)
            .unwrap_or(&CmdLineArgs::COMM_DEFAULT.to_string())
            .clone()
    }

    pub fn fen(&self) -> String {
        self.arguments
            .get_one::<String>(CmdLineArgs::FEN_LONG)
            .unwrap_or(&FEN_START_POSITION.to_string())
            .clone()
    }

    pub fn perft(&self) -> i8 {
        *self
            .arguments
            .get_one::<i8>(CmdLineArgs::PERFT_LONG)
            .unwrap_or(&CmdLineArgs::PERFT_DEFAULT)
    }

    pub fn threads(&self) -> usize {
        *self
            .arguments
            .get_one::<usize>(CmdLineArgs::THREADS_LONG)
            .unwrap_or(&CmdLineArgs::THREADS_DEFAULT)
    }

    pub fn hash(&self) -> usize {
        *self
            .arguments
            .get_one::<usize>(CmdLineArgs::HASH_LONG)
            .unwrap_or(&CmdLineArgs::HASH_DEFAULT)
    }

    pub fn has_kiwipete(&self) -> bool {
        self.arguments.get_flag(CmdLineArgs::KIWI_LONG)
    }

    pub fn has_quiet(&self) -> bool {
        self.arguments.get_flag(CmdLineArgs::QUIET_LONG)
    }

    #[cfg(feature = "extra")]
    pub fn has_wizardry(&self) -> bool {
        self.arguments.get_flag(CmdLineArgs::WIZARDRY_LONG)
    }

    #[cfg(feature = "extra")]
    pub fn has_test(&self) -> bool {
        self.arguments.get_flag(CmdLineArgs::EPD_TEST_LONG)
    }

    fn get() -> ArgMatches {
        let mut cmd_line = clap::Command::new(About::ENGINE)
            .version(About::VERSION)
            .author(About::AUTHOR)
            .about(About::WEBSITE)
            .disable_help_flag(true)
            .arg(
                Arg::new(CmdLineArgs::COMM_LONG)
                    .short(CmdLineArgs::COMM_SHORT)
                    .long(CmdLineArgs::COMM_LONG)
                    .help(CmdLineArgs::COMM_HELP)
                    .num_args(1)
                    .default_value(CmdLineArgs::COMM_DEFAULT)
                    .value_parser(CmdLineArgs::COMM_VALUES),
            )
            .arg(
                Arg::new(CmdLineArgs::FEN_LONG)
                    .short(CmdLineArgs::FEN_SHORT)
                    .long(CmdLineArgs::FEN_LONG)
                    .help(CmdLineArgs::FEN_HELP)
                    .num_args(1)
                    .default_value(FEN_START_POSITION)
                    .value_parser(value_parser!(String)),
            )
            .arg(
                Arg::new(CmdLineArgs::PERFT_LONG)
                    .short(CmdLineArgs::PERFT_SHORT)
                    .long(CmdLineArgs::PERFT_LONG)
                    .help(CmdLineArgs::PERFT_HELP)
                    .value_parser(value_parser!(i8))
                    .num_args(1),
            )
            .arg(
                Arg::new(CmdLineArgs::THREADS_LONG)
                    .short(CmdLineArgs::THREADS_SHORT)
                    .long(CmdLineArgs::THREADS_LONG)
                    .help(CmdLineArgs::THREADS_HELP)
                    .value_parser(value_parser!(usize))
                    .num_args(1),
            )
            .arg(
                Arg::new(CmdLineArgs::HASH_LONG)
                    .short(CmdLineArgs::HASH_SHORT)
                    .long(CmdLineArgs::HASH_LONG)
                    .help(CmdLineArgs::HASH_HELP)
                    .value_parser(value_parser!(usize))
                    .num_args(1),
            )
            .arg(
                Arg::new(CmdLineArgs::KIWI_LONG)
                    .long(CmdLineArgs::KIWI_LONG)
                    .short(CmdLineArgs::KIWI_SHORT)
                    .help(CmdLineArgs::KIWI_HELP)
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new(CmdLineArgs::QUIET_LONG)
                    .long(CmdLineArgs::QUIET_LONG)
                    .short(CmdLineArgs::QUIET_SHORT)
                    .help(CmdLineArgs::QUIET_HELP)
                    .action(ArgAction::SetTrue),
            );

        if cfg!(feature = "extra") {
            cmd_line = cmd_line
                .arg(
                    Arg::new(CmdLineArgs::WIZARDRY_LONG)
                        .short(CmdLineArgs::WIZARDRY_SHORT)
                        .long(CmdLineArgs::WIZARDRY_LONG)
                        .help(CmdLineArgs::WIZARDRY_HELP)
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    Arg::new(CmdLineArgs::EPD_TEST_LONG)
                        .short(CmdLineArgs::EPD_TEST_SHORT)
                        .long(CmdLineArgs::EPD_TEST_LONG)
                        .help(CmdLineArgs::EPD_TEST_HELP)
                        .action(ArgAction::SetTrue),
                );
        }

        cmd_line.get_matches()
    }
}
