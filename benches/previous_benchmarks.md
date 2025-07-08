# Previous Benchmark results 

### 08/07/25

Though it says improved, compared to yday it hasn't. Must of ruined the bench somewhere.

```
evaluation_positions/evaluate_position/Starting Position
                        time:   [162.48 ns 166.40 ns 171.20 ns]
                        change: [-12.174% -10.113% -7.7849%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) high mild
  9 (9.00%) high severe
evaluation_positions/evaluate_position/After 1.e4
                        time:   [86.999 ns 88.049 ns 89.547 ns]
                        change: [-20.817% -18.576% -16.205%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) high mild
  9 (9.00%) high severe
evaluation_positions/evaluate_position/Petrov Defense
                        time:   [90.009 ns 90.844 ns 91.908 ns]
                        change: [-17.203% -14.660% -11.830%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
evaluation_positions/evaluate_position/Italian Game
                        time:   [89.463 ns 90.327 ns 91.433 ns]
                        change: [-20.266% -18.229% -16.106%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
evaluation_positions/evaluate_position/Italian Game - Developed
                        time:   [88.721 ns 89.532 ns 90.518 ns]
                        change: [-18.813% -17.059% -15.357%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
evaluation_positions/evaluate_position/Castled Position
                        time:   [89.854 ns 90.851 ns 92.046 ns]
                        change: [-19.121% -16.962% -14.622%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
evaluation_positions/evaluate_position/Complex Middlegame
                        time:   [86.318 ns 86.989 ns 87.791 ns]
                        change: [-18.675% -16.596% -14.472%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) high mild
  8 (8.00%) high severe
evaluation_positions/evaluate_position/King and Pawn Endgame
                        time:   [63.205 ns 63.947 ns 64.871 ns]
                        change: [-20.905% -19.162% -17.491%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
evaluation_positions/evaluate_position/Lone King
                        time:   [44.188 ns 44.518 ns 44.922 ns]
                        change: [-22.518% -20.428% -18.390%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
evaluation_positions/evaluate_position/Starting Position (duplicate for baseline)
                        time:   [164.30 ns 167.02 ns 170.46 ns]
                        change: [-20.503% -18.434% -16.437%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

evaluation_caching/cached_evaluation
                        time:   [864.51 ns 878.78 ns 897.10 ns]
                        change: [-17.505% -15.339% -13.256%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe
evaluation_caching/cache_invalidation
                        time:   [506.57 ns 522.83 ns 541.34 ns]
                        change: [-17.587% -14.842% -11.914%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe

evaluation_components/pawn_structure
                        time:   [469.34 ps 476.24 ps 484.79 ps]
                        change: [-18.567% -16.701% -14.845%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
evaluation_components/mobility
                        time:   [469.00 ps 472.73 ps 477.33 ps]
                        change: [-19.033% -17.291% -15.595%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
evaluation_components/king_safety
                        time:   [76.382 ns 76.762 ns 77.231 ns]
                        change: [-21.301% -19.715% -18.210%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

evaluation_search_simulation/rapid_evaluations
                        time:   [8.3934 µs 8.4395 µs 8.4957 µs]
                        change: [-24.431% -22.081% -19.682%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  4 (4.00%) high mild
  10 (10.00%) high severe
evaluation_search_simulation/search_tree_simulation
                        time:   [4.4181 µs 4.4767 µs 4.5501 µs]
                        change: [-16.396% -14.110% -11.773%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe

evaluation_game_phases/evaluation/Opening
                        time:   [162.50 ns 164.38 ns 166.99 ns]
                        change: [-21.330% -19.355% -17.336%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
evaluation_game_phases/evaluation/Middlegame
                        time:   [93.693 ns 96.818 ns 100.43 ns]
                        change: [-17.620% -13.896% -9.6444%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe
evaluation_game_phases/evaluation/Endgame
                        time:   [65.119 ns 65.968 ns 66.973 ns]
                        change: [-20.768% -18.569% -16.427%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
evaluation_game_phases/evaluation/Late Endgame
                        time:   [73.948 ns 74.583 ns 75.412 ns]
                        change: [-21.077% -18.931% -16.859%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe


Perft 1: 20 (0 ms, inf leaves/sec, hash full: 0%)
Perft 2: 400 (0 ms, inf leaves/sec, hash full: 0%)
Perft 3: 8902 (0 ms, inf leaves/sec, hash full: 0%)
Perft 4: 197281 (8 ms, 24660125 leaves/sec, hash full: 0%)
Perft 5: 4865609 (135 ms, 36041548 leaves/sec, hash full: 0.2%)
Perft 6: 119060324 (1614 ms, 73767239 leaves/sec, hash full: 2.9%)
Perft 7: 3195901860 (19962 ms, 160099281 leaves/sec, hash full: 33.1%)
Total time spent: 21719 ms
Execution speed: 152863133 leaves/second


```


### 07/07/25

```
evaluation_positions/evaluate_position/Starting Position
                        time:   [156.33 ns 157.24 ns 158.42 ns]
                        change: [-0.3983% +0.7013% +1.7265%] (p = 0.22 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
evaluation_positions/evaluate_position/After 1.e4
                        time:   [83.067 ns 83.313 ns 83.628 ns]
                        change: [-1.9912% -1.0462% -0.1272%] (p = 0.03 < 0.05)
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
evaluation_positions/evaluate_position/Petrov Defense
                        time:   [86.331 ns 87.212 ns 88.181 ns]
                        change: [+0.2955% +1.9743% +4.0201%] (p = 0.03 < 0.05)
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) high mild
  10 (10.00%) high severe
evaluation_positions/evaluate_position/Italian Game
                        time:   [85.067 ns 85.620 ns 86.305 ns]
                        change: [-3.7679% -1.7820% -0.0625%] (p = 0.06 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
evaluation_positions/evaluate_position/Italian Game - Developed
                        time:   [85.326 ns 85.858 ns 86.526 ns]
                        change: [-2.3498% -1.3290% -0.4449%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
evaluation_positions/evaluate_position/Castled Position
                        time:   [85.736 ns 86.836 ns 88.301 ns]
                        change: [-0.5770% +1.0292% +2.8031%] (p = 0.24 > 0.05)
                        No change in performance detected.
Found 21 outliers among 100 measurements (21.00%)
  4 (4.00%) low mild
  4 (4.00%) high mild
  13 (13.00%) high severe
evaluation_positions/evaluate_position/Complex Middlegame
                        time:   [83.196 ns 83.389 ns 83.585 ns]
                        change: [-1.2319% -0.5416% +0.0400%] (p = 0.10 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
evaluation_positions/evaluate_position/King and Pawn Endgame
                        time:   [62.849 ns 63.618 ns 64.497 ns]
                        change: [-0.0002% +1.5956% +3.2044%] (p = 0.06 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  3 (3.00%) low mild
  13 (13.00%) high severe
evaluation_positions/evaluate_position/Lone King
                        time:   [44.369 ns 44.726 ns 45.134 ns]
                        change: [-3.9632% -1.5174% +0.4177%] (p = 0.21 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
evaluation_positions/evaluate_position/Starting Position (duplicate for baseline)
                        time:   [156.62 ns 157.59 ns 158.88 ns]
                        change: [-3.4296% -2.3795% -1.4481%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

evaluation_caching/cached_evaluation
                        time:   [836.22 ns 842.77 ns 850.72 ns]
                        change: [+1.4267% +2.4196% +3.5575%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
evaluation_caching/cache_invalidation
                        time:   [471.01 ns 474.02 ns 477.78 ns]
                        change: [-1.2839% +0.1282% +1.4053%] (p = 0.85 > 0.05)
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

evaluation_components/pawn_structure
                        time:   [450.00 ps 452.13 ps 454.70 ps]
                        change: [-5.3408% -3.5838% -2.0661%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe
evaluation_components/mobility
                        time:   [462.19 ps 468.55 ps 476.38 ps]
                        change: [-1.5942% +0.7439% +2.8282%] (p = 0.52 > 0.05)
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) high mild
  10 (10.00%) high severe
evaluation_components/king_safety
                        time:   [75.466 ns 75.689 ns 75.943 ns]
                        change: [-0.1205% +0.4904% +1.2289%] (p = 0.15 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

evaluation_search_simulation/rapid_evaluations
                        time:   [8.5689 µs 8.6726 µs 8.7771 µs]
                        change: [+0.0096% +0.9785% +1.8741%] (p = 0.05 < 0.05)
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe
evaluation_search_simulation/search_tree_simulation
                        time:   [4.6216 µs 4.7157 µs 4.8404 µs]
                        change: [+5.0272% +7.0117% +9.0818%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

evaluation_game_phases/evaluation/Opening
                        time:   [167.30 ns 169.77 ns 172.72 ns]
                        change: [+24.631% +35.580% +48.006%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
evaluation_game_phases/evaluation/Middlegame
                        time:   [86.796 ns 87.776 ns 88.874 ns]
                        change: [+1.2478% +2.4153% +3.7780%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  8 (8.00%) high mild
  9 (9.00%) high severe
evaluation_game_phases/evaluation/Endgame
                        time:   [63.668 ns 64.004 ns 64.456 ns]
                        change: [+0.7510% +2.3038% +4.3798%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
evaluation_game_phases/evaluation/Late Endgame
                        time:   [75.407 ns 76.446 ns 77.729 ns]
                        change: [+1.9255% +2.8754% +3.8001%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe



Perft 1: 20 (0 ms, inf leaves/sec, hash full: 0%)
Perft 2: 400 (0 ms, inf leaves/sec, hash full: 0%)
Perft 3: 8902 (0 ms, inf leaves/sec, hash full: 0%)
Perft 4: 197281 (8 ms, 24660125 leaves/sec, hash full: 0.5%)
Perft 5: 4865609 (123 ms, 39557796 leaves/sec, hash full: 7.8%)
Perft 6: 119060324 (1599 ms, 74459239 leaves/sec, hash full: 79%)
Perft 7: 3195901860 (29388 ms, 108748532 leaves/sec, hash full: 100%)
```