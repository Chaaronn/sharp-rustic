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

use super::{
    defs::{ErrFatal, ErrNormal},
    Engine,
};
use crate::{
    comm::{uci::UciReport, CommControl, CommReport},
    defs::FEN_START_POSITION,
    engine::defs::EngineOptionName,
    evaluation::evaluate_position,
    search::{defs::{SearchControl, SearchMode, SearchParams, OVERHEAD}, SearchManager},
};
use std::sync::Arc;
use crossbeam_channel;

// This block implements handling of incoming information, which will be in
// the form of either Comm or Search reports.
impl Engine {
    pub fn comm_reports(&mut self, comm_report: &CommReport) {
        // Split out the comm reports according to their source.
        match comm_report {
            CommReport::Uci(u) => self.comm_reports_uci(u),
        }
    }

    // Handles "Uci" Comm reports sent by the UCI-module.
    fn comm_reports_uci(&mut self, u: &UciReport) {
        // Setup default variables.
        let mut sp = SearchParams::new();
        sp.quiet = self.settings.quiet;
        sp.sharp_margin = self.settings.sharp_margin;

        match u {
            UciReport::Uci => self.comm.send(CommControl::Identify),

            UciReport::UciNewGame => {
                self.board
                    .lock()
                    .expect(ErrFatal::LOCK)
                    .fen_read(Some(FEN_START_POSITION))
                    .expect(ErrFatal::NEW_GAME);
                self.tt_search.write().expect(ErrFatal::LOCK).clear();
            }

            UciReport::IsReady => self.comm.send(CommControl::Ready),

            UciReport::SetOption(option) => {
                match option {
                    EngineOptionName::Hash(value) => {
                        if let Ok(v) = value.parse::<usize>() {
                            self.tt_search.write().expect(ErrFatal::LOCK).resize(v);
                        } else {
                            let msg = String::from(ErrNormal::NOT_INT);
                            self.comm.send(CommControl::InfoString(msg));
                        }
                    }

                    EngineOptionName::ClearHash => {
                        self.tt_search.write().expect(ErrFatal::LOCK).clear()
                    }

                    EngineOptionName::SharpMargin(value) => {
                        if let Ok(v) = value.parse::<i16>() {
                            self.settings.sharp_margin = v;
                        } else {
                            let msg = String::from(ErrNormal::NOT_INT);
                            self.comm.send(CommControl::InfoString(msg));
                        }
                    }

                    EngineOptionName::Threads(value) => {
                        if let Ok(v) = value.parse::<usize>() {
                            if v >= 1 && v <= 64 {
                                self.settings.threads = v;
                                // Recreate search manager with new thread count
                                self.search = SearchManager::new(v);
                                // Reinitialize the search manager with a new channel
                                let (info_tx, info_rx) = crossbeam_channel::unbounded::<crate::engine::defs::Information>();
                                self.info_rx = Some(info_rx);
                                self.search.init(
                                    info_tx,
                                    Arc::clone(&self.board),
                                    Arc::clone(&self.mg),
                                    Arc::clone(&self.tt_search),
                                    self.settings.tt_size > 0,
                                );
                            } else {
                                let msg = format!("Thread count must be between 1 and 64, got {}", v);
                                self.comm.send(CommControl::InfoString(msg));
                            }
                        } else {
                            let msg = String::from(ErrNormal::NOT_INT);
                            self.comm.send(CommControl::InfoString(msg));
                        }
                    }

                    EngineOptionName::Nothing => (),
                };
            }

            UciReport::Position(fen, moves) => {
                let fen_result = self.board.lock().expect(ErrFatal::LOCK).fen_read(Some(fen));

                if fen_result.is_ok() {
                    for m in moves.iter() {
                        let ok = self.execute_move(m.clone());
                        if !ok {
                            let msg = format!("{}: {}", m, ErrNormal::NOT_LEGAL);
                            self.comm.send(CommControl::InfoString(msg));
                            break;
                        }
                    }
                }

                if fen_result.is_err() {
                    let msg = ErrNormal::FEN_FAILED.to_string();
                    self.comm.send(CommControl::InfoString(msg));
                }
            }

            UciReport::GoInfinite => {
                sp.search_mode = SearchMode::Infinite;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoDepth(depth) => {
                sp.depth = *depth;
                sp.search_mode = SearchMode::Depth;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoMoveTime(msecs) => {
                sp.move_time = *msecs - (OVERHEAD as u128);
                sp.search_mode = SearchMode::MoveTime;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoNodes(nodes) => {
                sp.nodes = *nodes;
                sp.search_mode = SearchMode::Nodes;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::GoGameTime(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::GameTime;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::Stop => {
                self.search.stop_search();
                self.search.send(SearchControl::Stop);
            }
            UciReport::Quit => self.quit(),

            UciReport::GoPonder(gt) => {
                sp.game_time = *gt;
                sp.search_mode = SearchMode::Ponder;
                self.pondering = true;
                self.search.start_search();
                self.search.send(SearchControl::Start(sp));
            }

            UciReport::PonderHit => {
                self.pondering = false;
                if let Some(m) = self.delayed_bestmove.take() {
                    self.comm.send(CommControl::BestMove(m));
                    self.comm.send(CommControl::Update);
                }
            }

            // Custom commands
            UciReport::Board => self.comm.send(CommControl::PrintBoard),
            UciReport::History => self.comm.send(CommControl::PrintHistory),
            UciReport::Eval => {
                let mut board_guard = self.board.lock().expect(ErrFatal::LOCK);
                let e = evaluate_position(&mut *board_guard, &self.mg);
                let msg = format!("Evaluation: {e} centipawns");
                self.comm.send(CommControl::InfoString(msg));
            }
            UciReport::Help => self.comm.send(CommControl::PrintHelp),
            UciReport::Unknown => (),
        }
    }
}
