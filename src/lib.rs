// Library interface for rustic-sharp chess engine
// This allows the crate to be used as both a binary and library

pub mod board;
pub mod comm;
pub mod defs;
pub mod engine;
pub mod evaluation;
#[cfg(feature = "extra")]
pub mod extra;
pub mod misc;
pub mod movegen;
pub mod search;

// Re-export commonly used items
pub use board::Board;
pub use defs::FEN_START_POSITION;
pub use evaluation::evaluate_position;
pub use movegen::MoveGenerator; 