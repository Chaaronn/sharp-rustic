/* =======================================================================
Search Definitions Module

This module contains all the constants, enums, structs, and type definitions
used throughout the chess engine's search functionality. It serves as the
central configuration point for search algorithms including alpha-beta,
Late Move Reduction (LMR), null move pruning, and time management.

The definitions here control the behaviour of:
- Alpha-beta search parameters
- Move reduction strategies (LMR)
- Transposition table batching
- Time management and game phase detection
- Search termination conditions
- Thread-local optimisations
======================================================================= */

use crate::{
    board::{Board, defs::ZobristKey},
    defs::{MAX_PLY, NrOf, Sides},
    engine::defs::{Information, SearchData, TT, LocalTTCache},
    movegen::{
        defs::{Move, ShortMove},
        MoveGenerator,
    },
};
use crossbeam_channel::{Receiver, Sender};
use std::{
    sync::{Arc, Mutex, RwLock},
    time::Instant,
};

// Import time management overhead constant from time module
pub use super::time::OVERHEAD;

// =======================================================================
// CORE SEARCH CONSTANTS
// =======================================================================

/// Infinity value for alpha-beta search bounds. Set below checkmate scores
/// to allow proper mate detection and scoring.
pub const INF: i16 = 25_000;

/// Window size for aspiration search optimisation. Starts with a narrow window
/// around the previous iteration's score and widens if the search fails.
pub const ASPIRATION_WINDOW: i16 = 50;

/// Base checkmate score. Actual mate scores are calculated as CHECKMATE - distance_to_mate
/// to prefer shorter mates over longer ones.
pub const CHECKMATE: i16 = 24_000;

/// Threshold for detecting mate scores. Any score above this is considered a mate.
pub const CHECKMATE_THRESHOLD: i16 = 23_900;

/// Score returned for stalemate positions (drawn game).
pub const STALEMATE: i16 = 0;

/// Score returned for drawn positions.
pub const DRAW: i16 = 0;

/// Margin for "sharp" move analysis - moves within this evaluation range
/// are considered roughly equivalent for tactical sequence analysis.
pub const SHARP_MARGIN: i16 = 30;

/// Maximum depth for analysing sharp tactical sequences to prevent excessive computation.
pub const SHARP_SEQUENCE_DEPTH_CAP: i8 = 3;

// =======================================================================
// SEARCH TIMING AND STATISTICS
// =======================================================================

/// Bitmask for checking search termination. Check every 2048 nodes (0x7FF + 1).
/// This balances responsiveness with performance overhead.
pub const CHECK_TERMINATION: usize = 0x7FF;

/// Bitmask for sending statistics to GUI. Send stats every 524,288 nodes (0x7FFFF + 1).
/// Less frequent than termination checks to reduce communication overhead.
pub const SEND_STATS: usize = 0x7FFFF;

/// Minimum time (in milliseconds) between sending statistics to avoid spamming the GUI.
pub const MIN_TIME_STATS: u128 = 2_000;

/// Minimum time (in milliseconds) between sending current move updates.
pub const MIN_TIME_CURR_MOVE: u128 = 1_000;

// =======================================================================
// MOVE ORDERING AND HEURISTICS
// =======================================================================

/// Maximum number of killer moves stored per ply. Killer moves are quiet moves
/// that caused beta cutoffs and are likely to be good in similar positions.
pub const MAX_KILLER_MOVES: usize = 2;

// =======================================================================
// SEARCH PRUNING TECHNIQUES
// =======================================================================

/// Depth reduction for null move pruning. When making a "null move" (passing the turn),
/// search 3 plies shallower to detect if the position is still good enough for a cutoff.
pub const NULL_MOVE_REDUCTION: i8 = 3;

/// Standard Late Move Reduction amount for early moves. Reduces search depth by 1 ply
/// for moves that are likely to be inferior (conservative reduction).
pub const LMR_REDUCTION: i8 = 1;

/// Move number threshold for starting Late Move Reduction. Only begin reducing moves
/// from the 4th move onwards, as early moves are more likely to be important.
pub const LMR_MOVE_THRESHOLD: u8 = 4;

/// Move number threshold for applying more aggressive Late Move Reduction.
/// Moves beyond the 8th position get larger reductions as they're less likely to be best.
pub const LMR_LATE_THRESHOLD: u8 = 8;

/// Reduction amount for very late moves (beyond LMR_LATE_THRESHOLD).
/// Conservative 1-ply reduction to avoid missing important defensive resources.
pub const LMR_LATE_REDUCTION: i8 = 1;

/// Minimum depth required before applying Late Move Reduction. Only use LMR
/// in deeper searches where the time savings are worthwhile.
pub const LMR_MIN_DEPTH: i8 = 4;

/// Minimum depth for applying Multi-Cut pruning. This aggressive technique
/// tries multiple moves at reduced depth to detect early cutoffs.
pub const MULTICUT_DEPTH: i8 = 4;

/// Depth reduction for Multi-Cut search attempts.
pub const MULTICUT_REDUCTION: i8 = 3;

/// Number of cutoffs required before Multi-Cut triggers a beta cutoff.
pub const MULTICUT_CUTOFFS: u8 = 2;

/// Maximum number of moves to try in Multi-Cut before giving up.
pub const MULTICUT_MOVES: u8 = 4;

/// Depth extension for recapture moves. Recaptures are tactically important
/// and deserve extra search attention.
pub const RECAPTURE_EXTENSION: i8 = 1;

// =======================================================================
// TIME MANAGEMENT CONSTANTS
// =======================================================================

/// Emergency time threshold in milliseconds. When remaining time drops below this,
/// the engine switches to emergency mode with reduced search depth and time allocation.
pub const EMERGENCY_TIME_THRESHOLD: u128 = 2_000;

/// Maximum search depth allowed in emergency time mode to prevent time losses.
pub const EMERGENCY_MAX_DEPTH: i8 = 8;

/// Factor for reducing time allocation in emergency mode (50% of normal time).
pub const EMERGENCY_TIME_FACTOR: f64 = 0.5;

// =======================================================================
// GAME PHASE DETECTION CONSTANTS
// =======================================================================

/// Ply threshold for considering a game to be in the opening phase.
/// Used for adaptive time management and move selection strategies.
pub const OPENING_PLY_THRESHOLD: usize = 25;

/// Ply threshold for transitioning from opening to early middlegame.
pub const EARLY_MIDDLEGAME_PLY_THRESHOLD: usize = 30;

/// Ply threshold for transitioning to late middlegame phase.
pub const LATE_MIDDLEGAME_PLY_THRESHOLD: usize = 40;

/// Piece count threshold for detecting endgame phase. When total pieces
/// on board drop below this number, endgame time management kicks in.
pub const ENDGAME_PIECE_THRESHOLD: usize = 12;

// =======================================================================
// TIME MANAGEMENT ENUMERATIONS
// =======================================================================

/// Represents the current phase of the chess game for adaptive time management.
/// Different phases require different time allocation strategies.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum GamePhase {
    /// Opening phase: Book moves, rapid development
    Opening,
    /// Early middlegame: Tactical complications begin
    EarlyMiddlegame,
    /// Late middlegame: Complex tactical and positional play
    LateMiddlegame,
    /// Endgame: Precise calculation required, fewer pieces
    Endgame,
}

/// Time control categories for different playing speeds.
/// Used to adjust search behaviour and time allocation.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TimeControl {
    /// Bullet games: Less than 3 minutes per side
    Bullet,
    /// Blitz games: 3-15 minutes per side
    Blitz,
    /// Rapid games: 15-60 minutes per side
    Rapid,
    /// Classical games: More than 60 minutes per side
    Classical,
}

/// Quality assessment of the current move situation.
/// Used to determine if extra time should be allocated.
#[derive(PartialEq, Copy, Clone)]
pub enum MoveQuality {
    /// Clear best move identified, can move quickly
    Excellent,
    /// Good move available, some alternatives worth considering
    Good,
    /// Multiple reasonable moves, need standard time
    Acceptable,
    /// Difficult position, limited good options
    Poor,
    /// Critical position requiring extensive analysis
    Critical,
}

// =======================================================================
// TIME MANAGEMENT STATISTICS
// =======================================================================

/// Comprehensive statistics tracking for time management performance.
/// This data is used to improve time allocation decisions during games.
#[derive(Clone, PartialEq)]
pub struct TimeStats {
    /// Total number of moves played with time management
    pub total_moves: usize,
    
    /// Number of moves where time allocation was successful (didn't exceed limit)
    pub successful_allocations: usize,
    
    /// Number of moves that resulted in time loss (exceeded allocation)
    pub time_losses: usize,
    
    /// Running average of time spent per move in milliseconds
    pub average_time_per_move: u128,
    
    /// Time usage statistics broken down by game phase
    /// Helps optimise phase-specific time allocation
    pub time_usage_by_phase: std::collections::HashMap<GamePhase, u128>,
    
    /// Timestamp of the last statistics update
    pub last_update: std::time::Instant,
}

impl TimeStats {
    /// Creates a new TimeStats instance with all counters initialised to zero.
    /// Used at the start of a new game or when resetting statistics.
    pub fn new() -> Self {
        Self {
            total_moves: 0,
            successful_allocations: 0,
            time_losses: 0,
            average_time_per_move: 0,
            time_usage_by_phase: std::collections::HashMap::new(),
            last_update: std::time::Instant::now(),
        }
    }

    /// Updates the time statistics with data from a completed move.
    /// 
    /// # Arguments
    /// * `time_used` - Time spent on the move in milliseconds
    /// * `success` - Whether the move completed within the allocated time
    /// * `phase` - The game phase during which this move was played
    pub fn update(&mut self, time_used: u128, success: bool, phase: GamePhase) {
        self.total_moves += 1;
        
        // Track success/failure of time allocations
        if success {
            self.successful_allocations += 1;
        } else {
            self.time_losses += 1;
        }

        // Update running average time per move using weighted calculation
        // This maintains accuracy without storing all historical values
        let total_time = self.average_time_per_move * (self.total_moves - 1) as u128 + time_used;
        self.average_time_per_move = total_time / self.total_moves as u128;

        // Update phase-specific statistics using exponential moving average
        // This gives more weight to recent data whilst maintaining historical context
        let phase_time = self.time_usage_by_phase.entry(phase).or_insert(0);
        *phase_time = (*phase_time + time_used) / 2;

        self.last_update = std::time::Instant::now();
    }

    /// Calculates the success rate of time allocations as a percentage.
    /// Returns 0.0 if no moves have been tracked yet.
    /// 
    /// # Returns
    /// Success rate as a value between 0.0 and 1.0
    pub fn success_rate(&self) -> f64 {
        if self.total_moves == 0 {
            0.0
        } else {
            self.successful_allocations as f64 / self.total_moves as f64
        }
    }
}

// =======================================================================
// TYPE DEFINITIONS
// =======================================================================

/// Result type returned by search functions containing the best move and termination reason.
pub type SearchResult = (Move, SearchTerminate);

/// Unique identifier for search threads in multi-threaded search.
pub type ThreadId = u32;

/// Killer moves storage: [ply][killer_slot] -> move
/// Stores the best quiet moves that caused cutoffs at each ply level.
type KillerMoves = [[ShortMove; MAX_KILLER_MOVES]; MAX_PLY as usize];

// =======================================================================
// TRANSPOSITION TABLE BATCHING
// =======================================================================

/// Size of transposition table update batches. Batching reduces contention
/// on the global TT write lock by accumulating updates before applying them.
const TT_BATCH_SIZE: usize = 16;

/// Single transposition table update entry containing the position key and search data.
#[derive(Clone)]
pub struct TTUpdate {
    /// Zobrist hash key uniquely identifying the position
    pub zobrist_key: ZobristKey,
    /// Search data to store (depth, score, best move, etc.)
    pub data: SearchData,
}

/// Batch container for transposition table updates. This optimisation reduces
/// the frequency of expensive write lock acquisitions on the global TT.
pub struct TTBatch {
    /// Vector of pending TT updates to be applied
    pub updates: Vec<TTUpdate>,
    /// Maximum batch size before forced flush
    pub size: usize,
}

impl TTBatch {
    /// Creates a new empty transposition table batch with pre-allocated capacity.
    /// The vector is sized to avoid reallocations during normal operation.
    pub fn new() -> Self {
        Self {
            updates: Vec::with_capacity(TT_BATCH_SIZE),
            size: TT_BATCH_SIZE,
        }
    }

    /// Adds a new transposition table update to the batch.
    /// Does not check if the batch is full - caller should check with is_full().
    /// 
    /// # Arguments
    /// * `zobrist_key` - Position hash key
    /// * `data` - Search data to store for this position
    pub fn add(&mut self, zobrist_key: ZobristKey, data: SearchData) {
        self.updates.push(TTUpdate { zobrist_key, data });
    }

    /// Checks if the batch has reached its maximum size and should be flushed.
    /// When true, the batch should be applied to the global TT and cleared.
    pub fn is_full(&self) -> bool {
        self.updates.len() >= self.size
    }

    /// Clears all pending updates from the batch.
    /// Called after the batch has been successfully applied to the global TT.
    pub fn clear(&mut self) {
        self.updates.clear();
    }

    /// Returns the current number of pending updates in the batch.
    pub fn len(&self) -> usize {
        self.updates.len()
    }
}

/// Equality comparison for TTBatch based on size and number of updates.
/// Used primarily for testing and debugging purposes.
impl PartialEq for TTBatch {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.updates.len() == other.updates.len()
    }
}

// =======================================================================
// THREAD-LOCAL DATA STRUCTURES
// =======================================================================

/// Thread-local data structure containing search state and optimisations
/// specific to individual search threads. This reduces contention on shared
/// resources and improves multi-threaded search performance.
pub struct ThreadLocalData {
    /// Unique identifier for this search thread
    pub thread_id: ThreadId,
    
    /// Thread-local cache of transposition table entries to reduce global TT access.
    /// Provides faster lookup for recently accessed positions.
    pub local_tt_cache: LocalTTCache<SearchData>,
    
    /// Batch container for pending transposition table updates.
    /// Reduces write lock contention on the global TT.
    pub tt_batch: TTBatch,
    
    /// Timestamp when the current search iteration began.
    /// Used for time management and search termination.
    pub search_start_time: Option<Instant>,
    
    /// Number of nodes searched by this thread in the current iteration.
    /// Used for performance statistics and load balancing.
    pub nodes_searched: usize,
    
    /// Best move found by this thread so far.
    /// Updated as better moves are discovered during search.
    pub best_move_found: Option<Move>,
    
    /// Current search depth reached by this thread.
    /// Used for iterative deepening and depth-based termination.
    pub search_depth: i8,
}

impl ThreadLocalData {
    /// Creates a new ThreadLocalData instance for the specified thread.
    /// Initialises all caches and counters to their default values.
    /// 
    /// # Arguments
    /// * `thread_id` - Unique identifier for this search thread
    pub fn new(thread_id: ThreadId) -> Self {
        Self {
            thread_id,
            local_tt_cache: LocalTTCache::new(),
            tt_batch: TTBatch::new(),
            search_start_time: None,
            nodes_searched: 0,
            best_move_found: None,
            search_depth: 0,
        }
    }

    /// Initialises the thread-local data for a new search iteration.
    /// Clears all caches and resets counters to prepare for fresh search.
    pub fn start_search(&mut self) {
        self.search_start_time = Some(Instant::now());
        self.nodes_searched = 0;
        self.best_move_found = None;
        self.search_depth = 0;
        
        // Clear caches to avoid stale data from previous searches
        self.local_tt_cache.clear();
        self.tt_batch.clear();
    }

    /// Returns the elapsed time since the current search iteration began.
    /// Used for time management and determining when to stop searching.
    /// 
    /// # Returns
    /// Elapsed time in milliseconds, or 0 if no search is active
    pub fn elapsed_time(&self) -> u128 {
        if let Some(start_time) = self.search_start_time {
            start_time.elapsed().as_millis()
        } else {
            0
        }
    }

    /// Updates the best move found by this thread.
    /// Called when a new best move is discovered during search.
    /// 
    /// # Arguments
    /// * `mv` - The new best move to store
    pub fn update_best_move(&mut self, mv: Move) {
        self.best_move_found = Some(mv);
    }

    /// Increments the node counter for this thread.
    /// Called after each position evaluation to track search progress.
    pub fn increment_nodes(&mut self) {
        self.nodes_searched += 1;
    }
}

// =======================================================================
// SEARCH CONTROL ENUMERATIONS
// =======================================================================

/// Commands sent to search threads to control their execution.
/// Used for inter-thread communication in multi-threaded search.
#[derive(PartialEq, Clone)]
pub enum SearchControl {
    /// Begin a new search with the specified parameters
    Start(SearchParams),
    /// Stop the current search and return the best move found so far
    Stop,
    /// Terminate the search thread permanently
    Quit,
    /// No action required (placeholder value)
    Nothing,
}

/// Reasons for search termination, used to coordinate thread shutdown
/// and communicate why a search ended.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchTerminate {
    /// Search stopped by user command or time limit
    Stop,
    /// Engine shutdown requested
    Quit,
    /// Search is still active
    Nothing,
}

/// Different search modes supported by the engine.
/// Determines how the search termination criteria are evaluated.
#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    /// Search to a fixed depth (e.g., "go depth 10")
    Depth,
    /// Search for a fixed amount of time (e.g., "go movetime 5000")
    MoveTime,
    /// Search a fixed number of nodes (e.g., "go nodes 1000000")
    Nodes,
    /// Time-controlled game with time management (e.g., "go wtime 300000 btime 300000")
    GameTime,
    /// Pondering mode - search whilst opponent is thinking
    Ponder,
    /// Search until manually stopped (e.g., "go infinite")
    Infinite,
    /// No search mode specified
    Nothing,
}

/// Time control parameters for game-time searches.
/// Contains all timing information needed for proper time management.
#[derive(PartialEq, Copy, Clone)]
pub struct GameTime {
    /// White's remaining time in milliseconds
    pub wtime: u128,
    /// Black's remaining time in milliseconds  
    pub btime: u128,
    /// White's time increment per move in milliseconds
    pub winc: u128,
    /// Black's time increment per move in milliseconds
    pub binc: u128,
    /// Number of moves until next time control, if applicable
    pub moves_to_go: Option<usize>,
}

impl GameTime {
    /// Creates a new GameTime instance with the specified time control parameters.
    /// 
    /// # Arguments
    /// * `wtime` - White's remaining time in milliseconds
    /// * `btime` - Black's remaining time in milliseconds  
    /// * `winc` - White's increment per move in milliseconds
    /// * `binc` - Black's increment per move in milliseconds
    /// * `moves_to_go` - Optional number of moves to next time control
    pub fn new(
        wtime: u128,
        btime: u128,
        winc: u128,
        binc: u128,
        moves_to_go: Option<usize>,
    ) -> Self {
        Self {
            wtime,
            btime,
            winc,
            binc,
            moves_to_go,
        }
    }
}

/// Complete set of search parameters and configuration options.
/// Contains all information needed to configure a search iteration.
#[derive(PartialEq, Copy, Clone)]
pub struct SearchParams {
    /// Maximum depth to search (for depth-limited searches)
    pub depth: i8,
    /// Fixed time per move in milliseconds (for movetime searches)
    pub move_time: u128,
    /// Maximum nodes to search (for node-limited searches) 
    pub nodes: usize,
    /// Time control parameters (for game-time searches)
    pub game_time: GameTime,
    /// Search mode determining termination criteria
    pub search_mode: SearchMode,
    /// Whether to suppress output during search (for background analysis)
    pub quiet: bool,
    /// Evaluation margin for sharp move analysis
    pub sharp_margin: i16,
}

impl SearchParams {
    /// Creates a new SearchParams instance with default values.
    /// Sets reasonable defaults for all search parameters.
    pub fn new() -> Self {
        Self {
            depth: MAX_PLY,
            move_time: 0,
            nodes: 0,
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
            sharp_margin: SHARP_MARGIN,
        }
    }

    /// Checks if this search is using game-time mode with time management.
    /// 
    /// # Returns
    /// True if search should use time management, false for other modes
    pub fn is_game_time(&self) -> bool {
        matches!(self.search_mode, SearchMode::GameTime)
    }
}

/// Comprehensive search state and statistics tracking.
/// Maintains all information needed during search execution and for reporting progress.
#[derive(PartialEq)]
pub struct SearchInfo {
    /// Timestamp when the current search began (private for controlled access)
    start_time: Option<Instant>,
    
    /// Current search depth in the main search
    pub depth: i8,
    
    /// Maximum depth reached in any search branch (selective depth)
    pub seldepth: i8,
    
    /// Total number of nodes searched in current iteration
    pub nodes: usize,
    
    /// Current ply (half-moves) from the root position  
    pub ply: i8,
    
    /// Killer moves table: [ply][slot] -> move
    /// Stores quiet moves that caused beta cutoffs for move ordering
    pub killer_moves: KillerMoves,
    
    /// Timestamp of last statistics report to GUI (to avoid spam)
    pub last_stats_sent: u128,
    
    /// History heuristic scores: [side][piece][target_square] -> score
    /// Tracks success of quiet moves for better move ordering
    pub history_heuristic: [[[u32; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
    
    /// Counter moves table: [side][piece][square] -> move
    /// Stores best replies to opponent moves for move ordering
    pub counter_moves: [[[ShortMove; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
    
    /// Timestamp of last current move report to GUI
    pub last_curr_move_sent: u128,
    
    /// Time allocated for the current move in milliseconds
    pub allocated_time: u128,
    
    /// Current search termination status
    pub terminate: SearchTerminate,
    
    /// Analysis of all legal moves at the root position
    pub root_analysis: Vec<RootMoveAnalysis>,
    
    /// Thread-local transposition table cache for performance
    pub local_tt_cache: LocalTTCache<SearchData>,
    
    /// Batch container for pending TT updates
    pub tt_batch: TTBatch,
    
    // =======================================================================
    // TIME MANAGEMENT FIELDS
    // =======================================================================
    
    /// Whether the engine is in emergency time mode (low time remaining)
    pub emergency_mode: bool,
    
    /// Maximum search depth allowed (may be limited by time pressure)
    pub max_depth: i8,
    
    /// Comprehensive time management statistics and tracking
    pub time_stats: TimeStats,
}

impl SearchInfo {
    /// Creates a new SearchInfo instance with all fields initialised to default values.
    /// Sets up empty tables for killer moves, history heuristic, and counter moves.
    pub fn new() -> Self {
        Self {
            start_time: None,
            depth: 0,
            seldepth: 0,
            nodes: 0,
            ply: 0,
            killer_moves: [[ShortMove::new(0); MAX_KILLER_MOVES]; MAX_PLY as usize],
            history_heuristic: [[[0u32; NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
            counter_moves: [[[ShortMove::new(0); NrOf::SQUARES]; NrOf::PIECE_TYPES]; Sides::BOTH],
            last_stats_sent: 0,
            last_curr_move_sent: 0,
            allocated_time: 0,
            terminate: SearchTerminate::Nothing,
            root_analysis: Vec::new(),
            local_tt_cache: LocalTTCache::new(),
            tt_batch: TTBatch::new(),
            emergency_mode: false,
            max_depth: 0,
            time_stats: TimeStats::new(),
        }
    }

    /// Starts the search timer for the current iteration.
    /// Should be called at the beginning of each search to enable time tracking.
    pub fn timer_start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Returns the elapsed time since the search timer was started.
    /// Used for time management and progress reporting.
    /// 
    /// # Returns
    /// Elapsed time in milliseconds, or 0 if timer hasn't been started
    pub fn timer_elapsed(&self) -> u128 {
        if let Some(x) = self.start_time {
            x.elapsed().as_millis()
        } else {
            0
        }
    }

    /// Checks if the search has been interrupted by external command.
    /// 
    /// # Returns
    /// True if search should be terminated, false if it can continue
    pub fn interrupted(&self) -> bool {
        self.terminate != SearchTerminate::Nothing
    }

    /// Reinitialises the SearchInfo whilst preserving time management statistics.
    /// Useful when starting a new game but wanting to keep historical time data
    /// for improved time allocation decisions.
    pub fn preserve_time_stats(&mut self) {
        // Keep the existing time statistics across reinitialisation
        let preserved_stats = self.time_stats.clone();
        *self = Self::new();
        self.time_stats = preserved_stats;
    }
}

// =======================================================================
// SEARCH REPORTING STRUCTURES
// =======================================================================

/// Complete search results summary for GUI reporting.
/// Contains all information typically sent in UCI "info" messages.
#[derive(PartialEq, Clone)]
pub struct SearchSummary {
    /// Depth searched to completion
    pub depth: i8,
    /// Maximum depth reached in any variation (selective depth)
    pub seldepth: i8,
    /// Time spent searching in milliseconds
    pub time: u128,
    /// Evaluation score in centipawns (100 = 1 pawn)
    pub cp: i16,
    /// Mate distance if mate found (0 if no mate)
    pub mate: u8,
    /// Total nodes searched
    pub nodes: usize,
    /// Search speed in nodes per second
    pub nps: usize,
    /// Transposition table fullness (per mille - parts per 1000)
    pub hash_full: u16,
    /// Principal variation (best line of play found)
    pub pv: Vec<Move>,
}

impl SearchSummary {
    /// Converts the principal variation to a human-readable string.
    /// Used for UCI output and debugging.
    /// 
    /// # Returns
    /// Space-separated string of moves in algebraic notation
    pub fn pv_as_string(&self) -> String {
        let mut pv = String::from("");
        for next_move in self.pv.iter() {
            let m = format!(" {}", next_move.as_string());
            pv.push_str(&m[..]);
        }
        pv
    }
}

/// Information about the move currently being searched.
/// Used for UCI "info currmove" reporting during long searches.
#[derive(PartialEq, Clone)]
pub struct SearchCurrentMove {
    /// The move currently being analysed
    pub curr_move: Move,
    /// Position of this move in the search order (1-based)
    pub curr_move_number: u8,
}

impl SearchCurrentMove {
    /// Creates a new SearchCurrentMove report.
    /// 
    /// # Arguments
    /// * `curr_move` - The move being searched
    /// * `curr_move_number` - Its position in the move list (1-based)
    pub fn new(curr_move: Move, curr_move_number: u8) -> Self {
        Self {
            curr_move,
            curr_move_number,
        }
    }
}

/// Basic search statistics for progress reporting.
/// Lighter-weight version of SearchSummary for frequent updates.
#[derive(PartialEq, Clone)]
pub struct SearchStats {
    /// Time elapsed in current search (milliseconds)
    pub time: u128,
    /// Nodes searched so far
    pub nodes: usize,
    /// Current search speed (nodes per second)
    pub nps: usize,
    /// Transposition table fullness (per mille)
    pub hash_full: u16,
}

impl SearchStats {
    /// Creates a new SearchStats instance.
    /// 
    /// # Arguments
    /// * `time` - Elapsed time in milliseconds
    /// * `nodes` - Number of nodes searched
    /// * `nps` - Nodes per second calculation
    /// * `hash_full` - TT fullness per mille
    pub fn new(time: u128, nodes: usize, nps: usize, hash_full: u16) -> Self {
        Self {
            time,
            nodes,
            nps,
            hash_full,
        }
    }
}

/// Analysis of a single root move including tactical sequences.
/// Used for sharp line detection and move quality assessment.
#[derive(PartialEq, Clone)]
pub struct RootMoveAnalysis {
    /// The move being analysed
    pub mv: Move,
    /// Evaluation score for this move
    pub eval: i16,
    /// Number of good replies available to the opponent
    pub good_replies: usize,
    /// Best reply found (if forced/limited options)
    pub reply: Option<Move>,
    /// Sequence of moves in sharp tactical lines
    pub reply_sequence: Vec<Move>,
}

// =======================================================================
// SEARCH CONTEXT STRUCTURE
// =======================================================================

/// Reference structure providing access to all search-related data.
/// Used to pass context to search functions without excessive parameter lists.
/// Lifetime parameter ensures references remain valid during search.
pub struct SearchRefs<'a> {
    /// Mutable reference to the chess board position
    pub board: &'a mut Board,
    /// Shared reference to the move generator
    pub mg: &'a Arc<MoveGenerator>,
    /// Shared reference to the transposition table
    pub tt: &'a Arc<RwLock<TT<SearchData>>>,
    /// Whether transposition table is enabled for this search
    pub tt_enabled: bool,
    /// Mutable reference to search parameters
    pub search_params: &'a mut SearchParams,
    /// Mutable reference to search state and statistics
    pub search_info: &'a mut SearchInfo,
    /// Channel for receiving search control commands
    pub control_rx: &'a Receiver<SearchControl>,
    /// Channel for sending information to the engine
    pub report_tx: &'a Sender<Information>,
    /// Thread-local data for optimisations
    pub thread_local_data: &'a mut ThreadLocalData,
}

// =======================================================================
// SEARCH REPORTING ENUMERATION
// =======================================================================

/// Different types of reports that can be sent from search threads.
/// Used for communication between search logic and the main engine.
#[derive(PartialEq, Clone)]
pub enum SearchReport {
    /// Search completed with the best move found
    Finished(Move),
    /// Comprehensive search results summary
    SearchSummary(SearchSummary),
    /// Information about current move being searched
    SearchCurrentMove(SearchCurrentMove),
    /// Basic search progress statistics
    SearchStats(SearchStats),
    /// Arbitrary information string for debugging/logging
    InfoString(String),
}