use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustic_sharp::{
    board::Board,
    evaluation::evaluate_position,
    movegen::{MoveGenerator, defs::{MoveList, MoveType}},
    defs::FEN_START_POSITION,
};
use std::sync::Arc;

// Test positions with different characteristics
const TEST_POSITIONS: &[(&str, &str)] = &[
    (FEN_START_POSITION, "Starting Position"),
    ("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", "After 1.e4"),
    ("rnbqkb1r/pppp1ppp/5n2/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 2 3", "Petrov Defense"),
    ("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4", "Italian Game"),
    ("rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R b KQkq - 0 5", "Italian Game - Developed"),
    ("r1bq1rk1/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQ1RK1 w - - 6 7", "Castled Position"),
    ("r2q1rk1/ppp2ppp/2np1n2/2b1p1B1/2B1P3/3P1N2/PPP2PPP/RN1Q1RK1 w - - 0 9", "Complex Middlegame"),
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", "King and Pawn Endgame"),
    ("8/8/8/8/8/8/8/K7 w - - 0 1", "Lone King"),
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", "Starting Position (duplicate for baseline)"),
];

/// Set up a board from FEN and return it with a move generator
fn setup_position(fen: &str) -> (Board, Arc<MoveGenerator>) {
    let mut board = Board::new();
    board.fen_read(Some(fen)).expect("Valid FEN");
    let mg = Arc::new(MoveGenerator::new());
    
    // Initialize evaluation caches
    board.init_evaluation_caches(&mg);
    
    (board, mg)
}

/// Benchmark evaluation function across different positions
fn bench_evaluation_positions(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_positions");
    
    for (fen, name) in TEST_POSITIONS {
        let (mut board, mg) = setup_position(fen);
        
        group.bench_with_input(
            BenchmarkId::new("evaluate_position", name),
            fen,
            |b, _| {
                b.iter(|| {
                    black_box(evaluate_position(&mut board, &mg))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark the effect of caching on evaluation speed
fn bench_evaluation_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_caching");
    
    // Use a complex middlegame position for this test
    let (mut board, mg) = setup_position("r2q1rk1/ppp2ppp/2np1n2/2b1p1B1/2B1P3/3P1N2/PPP2PPP/RN1Q1RK1 w - - 0 9");
    
    // Test with cache hits (calling evaluate multiple times without position changes)
    group.bench_function("cached_evaluation", |b| {
        b.iter(|| {
            // These calls should benefit from caching
            for _ in 0..10 {
                black_box(evaluate_position(&mut board, &mg));
            }
        });
    });
    
    // Test cache invalidation by making and unmaking moves
    group.bench_function("cache_invalidation", |b| {
        b.iter(|| {
            // This simulates the cost of cache invalidation during search
            let mut moves = MoveList::new();
            mg.generate_moves(&board, &mut moves, MoveType::All);
            if moves.len() > 0 {
                let first_move = moves.get_move(0);
                if board.make(first_move, &mg) {
                    black_box(evaluate_position(&mut board, &mg));
                    board.unmake();
                    black_box(evaluate_position(&mut board, &mg));
                }
            }
        });
    });
    
    group.finish();
}

/// Benchmark individual evaluation components
fn bench_evaluation_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_components");
    
    let (mut board, mg) = setup_position("r2q1rk1/ppp2ppp/2np1n2/2b1p1B1/2B1P3/3P1N2/PPP2PPP/RN1Q1RK1 w - - 0 9");
    
    // Benchmark pawn structure evaluation
    group.bench_function("pawn_structure", |b| {
        b.iter(|| {
            black_box(board.get_cached_pawn_structure_score())
        });
    });
    
    // Benchmark mobility evaluation
    group.bench_function("mobility", |b| {
        b.iter(|| {
            black_box(board.get_cached_mobility_score(&mg))
        });
    });
    
    // Benchmark king safety evaluation
    group.bench_function("king_safety", |b| {
        b.iter(|| {
            black_box(rustic_sharp::evaluation::kingsafety::evaluate_king_safety(&board, &mg))
        });
    });
    
    group.finish();
}

/// Benchmark evaluation under time pressure (simulating search conditions)
fn bench_evaluation_search_simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_search_simulation");
    
    let (mut board, mg) = setup_position("r2q1rk1/ppp2ppp/2np1n2/2b1p1B1/2B1P3/3P1N2/PPP2PPP/RN1Q1RK1 w - - 0 9");
    
    // Simulate rapid evaluation calls as would happen during search
    group.bench_function("rapid_evaluations", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(evaluate_position(&mut board, &mg));
            }
        });
    });
    
    // Simulate search tree exploration with move make/unmake
    group.bench_function("search_tree_simulation", |b| {
        b.iter(|| {
            let mut moves = MoveList::new();
            mg.generate_moves(&board, &mut moves, MoveType::All);
            for i in 0..std::cmp::min(5, moves.len()) { // Simulate exploring top 5 moves
                let mv = moves.get_move(i);
                if board.make(mv, &mg) {
                    black_box(evaluate_position(&mut board, &mg));
                    
                    // Simulate one level deeper
                    let mut counter_moves = MoveList::new();
                    mg.generate_moves(&board, &mut counter_moves, MoveType::All);
                    for j in 0..std::cmp::min(3, counter_moves.len()) {
                        let counter_mv = counter_moves.get_move(j);
                        if board.make(counter_mv, &mg) {
                            black_box(evaluate_position(&mut board, &mg));
                            board.unmake();
                        }
                    }
                    
                    board.unmake();
                }
            }
        });
    });
    
    group.finish();
}

/// Benchmark different game phases
fn bench_evaluation_game_phases(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_game_phases");
    
    let phases = [
        ("Opening", "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2"),
        ("Middlegame", "r2q1rk1/ppp2ppp/2np1n2/2b1p1B1/2B1P3/3P1N2/PPP2PPP/RN1Q1RK1 w - - 0 9"),
        ("Endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        ("Late Endgame", "8/8/8/8/8/3k4/8/3K4 w - - 0 1"),
    ];
    
    for (phase_name, fen) in &phases {
        let (mut board, mg) = setup_position(fen);
        
        group.bench_with_input(
            BenchmarkId::new("evaluation", phase_name),
            phase_name,
            |b, _| {
                b.iter(|| {
                    black_box(evaluate_position(&mut board, &mg))
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_evaluation_positions,
    bench_evaluation_caching,
    bench_evaluation_components,
    bench_evaluation_search_simulation,
    bench_evaluation_game_phases
);
criterion_main!(benches); 