# Evaluation Benchmarks

This document describes the evaluation benchmarking suite for Sharp-Rustic, which measures the performance of the chess evaluation function across different scenarios.

## Overview

The benchmark suite is designed to:
- Track evaluation performance across different chess positions
- Measure the effectiveness of caching strategies
- Profile individual evaluation components
- Simulate real search conditions
- Compare performance across different game phases

## Running Benchmarks

### Prerequisites
```bash
# Ensure you have the required dependencies
cargo install criterion
```

### Running All Benchmarks
```bash
cargo bench --bench evaluation --features extra
```

### Running Specific Benchmark Groups
```bash
# Position-specific benchmarks
cargo bench --bench evaluation --features extra -- evaluation_positions

# Caching performance
cargo bench --bench evaluation --features extra -- evaluation_caching

# Component benchmarks
cargo bench --bench evaluation --features extra -- evaluation_components

# Search simulation
cargo bench --bench evaluation --features extra -- evaluation_search_simulation

# Game phase comparison
cargo bench --bench evaluation --features extra -- evaluation_game_phases
```

## Benchmark Categories

### 1. Position-Specific Benchmarks (`evaluation_positions`)

**Purpose**: Measure evaluation performance across different chess positions with varying complexity.

**Test Positions**:
- Starting Position
- After 1.e4 (early opening)
- Petrov Defense
- Italian Game
- Italian Game - Developed
- Castled Position
- Complex Middlegame
- King and Pawn Endgame
- Lone King (minimal position)

**Expected Performance**:
- Regular positions: ~130-140 ns
- Endgame positions: ~75-85 ns
- Lone king: ~35-40 ns

### 2. Caching Benchmarks (`evaluation_caching`)

**Purpose**: Measure the effectiveness of evaluation caching strategies.

**Tests**:
- **Cached Evaluation**: Multiple evaluations without position changes
- **Cache Invalidation**: Cost of cache invalidation during move make/unmake

**Key Metrics**:
- Cache hit performance: ~130 ns per evaluation
- Cache invalidation overhead: ~400-450 ns
- Pawn structure cache: ~465 ps (extremely fast)

### 3. Component Benchmarks (`evaluation_components`)

**Purpose**: Profile individual evaluation components to identify bottlenecks.

**Components Measured**:
- **Pawn Structure**: Cached evaluation of pawn formations
- **Mobility**: Piece mobility calculations
- **King Safety**: King safety evaluation including pawn shields and attacks

**Performance Targets**:
- Pawn structure: ~465 ps (cached)
- Mobility: ~45-50 ns
- King safety: ~75-85 ns

### 4. Search Simulation (`evaluation_search_simulation`)

**Purpose**: Simulate real search conditions with rapid evaluations and tree exploration.

**Tests**:
- **Rapid Evaluations**: 100 consecutive evaluation calls
- **Search Tree Simulation**: Multi-level move exploration with make/unmake

**Performance Expectations**:
- Rapid evaluations: ~12-15 μs for 100 calls
- Search tree simulation: ~3.5-4.5 μs per iteration

### 5. Game Phase Benchmarks (`evaluation_game_phases`)

**Purpose**: Compare evaluation performance across different game phases.

**Game Phases**:
- **Opening**: Early game with many pieces
- **Middlegame**: Complex tactical positions
- **Endgame**: Fewer pieces, different evaluation focus
- **Late Endgame**: Minimal pieces, king activity important

**Performance Patterns**:
- Opening/Middlegame: ~130-140 ns (most complex)
- Endgame: ~75-85 ns (fewer pieces)
- Late Endgame: ~65-75 ns (minimal evaluation)

## Interpreting Results

### Performance Metrics

**Time Measurements**:
- **ns (nanoseconds)**: 1 billionth of a second
- **μs (microseconds)**: 1 millionth of a second
- **ps (picoseconds)**: 1 trillionth of a second

**Statistical Information**:
- **Mean**: Average execution time
- **Standard deviation**: Consistency of performance
- **Outliers**: Measurements outside expected range

### Performance Regression Detection

The benchmark suite automatically detects performance changes:
- **Performance regression**: Slower than previous runs
- **Performance improvement**: Faster than previous runs
- **No change detected**: Within noise threshold

### Caching Effectiveness

**Indicators of Good Caching**:
- Pawn structure evaluation in picoseconds
- Consistent performance across repeated evaluations
- Reasonable cache invalidation overhead

## Optimization Guidelines

### Performance Targets

| Component | Target Performance | Current Performance |
|-----------|-------------------|-------------------|
| Full Evaluation | <150 ns | ~130-140 ns ✓ |
| Pawn Structure | <1 ns | ~465 ps ✓ |
| Mobility | <60 ns | ~45-50 ns ✓ |
| King Safety | <100 ns | ~75-85 ns ✓ |

### Optimization Priorities

1. **Maintain caching effectiveness**: Ensure pawn structure remains cached
2. **Optimize hot paths**: Focus on components taking >50 ns
3. **Minimize allocation**: Avoid heap allocations in evaluation
4. **Vectorize operations**: Use bitboard operations where possible

## Troubleshooting

### Common Issues

**Benchmark Fails to Compile**:
```bash
# Ensure extra feature is enabled
cargo bench --bench evaluation --features extra
```

**Performance Regression**:
- Check if caching is working (pawn structure should be ~465 ps)
- Verify no new allocations in hot paths
- Compare component timings to identify bottlenecks

**Inconsistent Results**:
- Run benchmarks multiple times
- Check for system load during benchmarking
- Ensure consistent CPU frequency (disable power management)

### Debugging Performance Issues

**Profiling Individual Components**:
```bash
# Run component benchmarks specifically
cargo bench --bench evaluation --features extra -- evaluation_components
```

**Analyzing Cache Performance**:
```bash
# Focus on caching benchmarks
cargo bench --bench evaluation --features extra -- evaluation_caching
```

## Historical Performance

### Optimization Milestones

**Initial Implementation**:
- Full evaluation: ~200+ ns
- No caching: All components computed each time

**Pawn Structure Caching**:
- Pawn evaluation: From ~50 ns to ~465 ps
- Overall speedup: ~15-20%

**Mobility Caching**:
- Mobility evaluation: Consistent ~45-50 ns
- Search simulation: Improved by ~10-15%

**King Safety Optimization**:
- King safety: From ~120 ns to ~75-85 ns
- Balanced evaluation: Maintained accuracy while improving speed

## Future Improvements

### Planned Optimizations

1. **Incremental Mobility**: Update mobility incrementally during search
2. **Evaluation Pruning**: Skip expensive evaluations in certain positions
3. **SIMD Operations**: Vectorize bitboard operations
4. **Lazy Evaluation**: Compute only necessary components

### Benchmark Enhancements

1. **Memory Usage**: Track memory allocation patterns
2. **Cache Miss Analysis**: Detailed cache performance metrics
3. **Position Database**: Expand test positions for better coverage
4. **Regression Testing**: Automated performance regression detection

## Contributing

When making changes to the evaluation function:

1. **Run benchmarks before and after changes**
2. **Document performance impact**
3. **Ensure no regressions in accuracy**
4. **Add new test positions for edge cases**

### Benchmark Best Practices

- Run benchmarks on consistent hardware
- Close unnecessary applications during benchmarking
- Run multiple iterations to ensure consistency
- Document significant performance changes

## Conclusion

The benchmarking suite provides comprehensive coverage of evaluation performance, enabling continuous optimization while maintaining chess playing strength. Regular benchmark runs help detect regressions and guide optimization efforts.

For questions or improvements to the benchmark suite, please refer to the project's contribution guidelines. 