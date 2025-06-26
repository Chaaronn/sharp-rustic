<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [Changelog](#changelog)
  - [December 2024 - Sharp Rustic 0.0.2](#december-2024---sharp-rustic--002)
  - [March 22, 2024 - Sharp Rustic 0.0.1](#march-22-2024---sharp-rustic--001)
  - [January 21, 2024 - Rustic Alpha 3.0.5](#january-21-2024---rustic-alpha-305)
  - [December 28, 2023 - Rustic Alpha 3.0.4](#december-28-2023---rustic-alpha-304)
  - [March 28, 2023 - Rustic Alpha 3.0.3](#march-28-2023---rustic-alpha-303)
  - [June 11, 2022 - Rustic Alpha 3.0.2](#june-11-2022---rustic-alpha-302)
  - [November 6, 2021 - Rustic Alpha 3.0.1](#november-6-2021---rustic-alpha-301)
  - [June 18, 2021 - Rustic Alpha 3.0.0](#june-18-2021---rustic-alpha-300)
  - [December 28, 2023 - Rustic Alpha 2.3](#december-28-2023---rustic-alpha-23)
  - [March 28, 2023 - Rustic Alpha 2.2](#march-28-2023---rustic-alpha-22)
  - [June 11, 2022 - Rustic Alpha 2.1](#june-11-2022---rustic-alpha-21)
  - [March 17, 2021 - Rustic Alpha 2](#march-17-2021---rustic-alpha-2)
  - [December 28, 2023 - Rustic Alpha 1.4](#december-28-2023---rustic-alpha-14)
  - [March 28, 2023 - Rustic Alpha 1.3](#march-28-2023---rustic-alpha-13)
  - [June 11, 2021 - Rustic Alpha 1.2](#june-11-2021---rustic-alpha-12)
  - [March 15, 2021 - Rustic Alpha 1.1](#march-15-2021---rustic-alpha-11)
  - [January 24, 2021 - Rustic Alpha 1](#january-24-2021---rustic-alpha-1)

<!-- /code_chunk_output -->

# Changelog

## December 2024 - Sharp Rustic 0.0.2

**Major Time Management Overhaul**

This release addresses critical time management issues that were causing random timeouts. The engine now features a comprehensive, intelligent time management system that adapts to different game situations and time constraints.

### Time Management Improvements

- **Unified Time Checking Logic**: Replaced inconsistent time checking throughout the engine with a single, reliable `time_up()` function that uses `out_of_time()` consistently everywhere.

- **Progressive Time Management**: Implemented intelligent overshoot factors based on available time:
  - Critical time (≤1000ms): No overshoot (1.0x)
  - Very low time (≤500ms): Minimal overshoot (1.1x) 
  - Low time (≤2000ms): Small overshoot (1.2x)
  - OK time (≤5000ms): Moderate overshoot (1.3x)
  - Comfortable time (>5000ms): Larger overshoot (1.5x)

- **Position-Based Time Allocation**: Engine now considers position complexity when allocating time:
  - Check positions receive 20% time bonus
  - Tactical positions (high move count) get more time
  - Simple endgames receive time reduction
  - Game phase adjustments (opening/middlegame/endgame)

- **Early Termination Heuristics**: Prevents timeouts in critical situations with progressive thresholds:
  - Critical time: 70% of allocated time
  - Very low time: 80% of allocated time
  - Low time: 85% of allocated time
  - OK time: 90% of allocated time
  - Comfortable time: 95% of allocated time

- **Safety Buffers**: Added 15% safety buffer in time allocation and minimum 100ms buffer to prevent timeouts from unexpected scenarios.

- **Improved Moves-to-Go Estimation**: Enhanced game phase detection for better time allocation:
  - Opening: Full game length estimation
  - Middlegame: 75% of game length
  - Endgame: 50% of game length
  - Late endgame: 25% of game length

- **Time Management Debugging**: Added comprehensive logging of time management decisions for analysis and optimisation.

### Technical Improvements

- **Enhanced Search Termination**: Added early termination checks in both alpha-beta and quiescence search
- **Game Phase Detection**: Intelligent piece counting and position analysis
- **Comprehensive Test Suite**: Added tests to verify time management functionality
- **Detailed Documentation**: Extensive UK English comments throughout the codebase

### Expected Results

- **Eliminated**: Random timeouts due to inconsistent time checking
- **Improved**: Time allocation based on position complexity and game phase
- **Enhanced**: Safety in time-critical situations
- **Better**: Progressive time management that adapts to available time
- **Reliable**: Consistent time management across all search modes

## March 22, 2024 - Sharp Rustic 0.0.1

- Engine considers the evaluation of opponent move sequences when chosing the move.
  Root analysis now records the entire forced reply sequence.
- Iterative deepening output lists full sharp lines.
- Added UCI support for pondering

## January 21, 2024 - Rustic Alpha 3.0.5

Version 3.0.5 is a maintenance release. Its main 'feature' is having
dropped separate material counting, by moving the material values directly
into the PSQT's. This is done so that these PSQT's can be used as a basis
for the upcoming Rustic 4. That way, it can be determined how much playing
playing strength is added in Rustic 4 by tapering and tuning these tables.

- Unified the material values with the PSQT values the same way Rustic
  4-beta does. This does not impact playing strength of Alpha 3.
- Update 'clap' to 4.4.18
- Update 'crossbeam-channel' to 0.5.11
- Remove 'crossbeam-utils' (automatically pulled in)

## December 28, 2023 - Rustic Alpha 3.0.4

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

## March 28, 2023 - Rustic Alpha 3.0.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Update About banner layout
- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

## June 11, 2022 - Rustic Alpha 3.0.2

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade to Rust Edition 2021
- Upgrade 'rand' to 0.8.5
- Upgrade 'rand_chacha' to 0.3.1
- Upgrade 'if_chain' to 1.0.2
- Upgrade 'clap' to 3.2.8
- Upgrade 'crossbeam-channel' to 0.5.5
- Upgrade 'crossbeam-utils' to 0.8.10 (security fix)
- Upgrade 'rand_core' to 0.6.3 (security fix)

## November 6, 2021 - Rustic Alpha 3.0.1

Bugfix upgrade. There is no functional difference to the previous version.
For normal playing and testing, the binary of version 3.0.0 can be used.

- Fixed a variable having the wrong type. This caused the "extra" module
  failing to compile.

## June 18, 2021 - Rustic Alpha 3.0.0

- New features:
  - Killer Moves
  - Principal Variation Search (PVS)
- Changes:
  - Switch versioning scheme to SemVer. Versions are going to be in the
    form "a.b.c" from now on, with the following meaning:
    - Increment **a**: A new strength-gaining feature was added.
    - Increment **b**: A bug was fixed that gained strength.
    - Increment **c**: A feature was added or a bug was fixed that did not
      gain stregnth. It is not necessary to test this version for a rating
      change.
- Misc:
  - Updated crossbeam-channel to version 0.5.1
  - A Makefile was added, so Rustic can be built using "GNU Make". When
    typing "make" (or "gmake" in MacOS), the Makefile will build all Rustic
    versions for the platform it's being compiled on.
  - Re-add showing the size of the TT and number of threads in About.
  - Fairly large update of the book on https://rustic-chess.org/.

## December 28, 2023 - Rustic Alpha 2.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18

## March 28, 2023 - Rustic Alpha 2.2

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used. (Unless you run the engine on the command-line and REALLY want to see
the settings in the About banner. Then you will need to compile a new
version.)

- Update About banner with Hash and Threads settings
- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

## June 11, 2022 - Rustic Alpha 2.1

Maintenance upgrade. There is no functional difference to the previous
version. For normal playing and testing, the existing binaries can be used.

- Upgrade to Rust Edition 2021
- Upgrade 'rand' to 0.8.5
- Upgrade 'rand_chacha' to 0.3.1
- Upgrade 'if_chain' to 1.0.2
- Upgrade 'clap' to 3.2.8
- Upgrade 'crossbeam-channel' to 0.5.5
- Upgrade 'crossbeam-utils' to 0.8.10 (security fix)
- Upgrade 'rand_core' to 0.6.3 (security fix)

## March 17, 2021 - Rustic Alpha 2

[CCRL Blitz rating: +/- 1815 Elo](https://ccrl.chessdom.com/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%202%2064-bit#Rustic_Alpha_2_64-bit)

- New Features:
  - Transposition table for search and perft.
  - Ordering on transposition table move.
  - Set TT size through --hash option or UCI parameter.
- Improvement:
  - Move check extension higher up in the search routine, to prevent
    quiescence search while in check.
- Changes:
  - seldepth: report max ply reached during the search, instead of
    selective depth at last completed iteration.
  - Count all nodes visited, instead of only nodes which generated moves.
  - Change random number generator from SmallRng to ChaChaRng for
    reproducible behavior between platforms/OS's/architectures/versions.
- Cleanup
  - Change Root PV handling to remove redundant code.
  - Miscellaneous small renames, refactors, and cleanups.
  - Add rand_chacha and remove SmallRng number generators.
  - Update Rand library to 0.8.3.

## December 28, 2023 - Rustic Alpha 1.4

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'clap' to 4.4.11
- Upgrade 'crossbeam-channel' to 0.5.10
- Upgrade 'crossbeam-utils' to 0.8.18


## March 28, 2023 - Rustic Alpha 1.3

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade 'rand_core' to 0.6.4
- Upgrade 'clap' to 4.1.14
- Upgrade 'crossbeam-channel' to 0.5.7
- Upgrade 'crossbeam-utils' to 0.8.15

## June 11, 2021 - Rustic Alpha 1.2

Maintenance upgrades. There is no functional difference to the previous
versions. For normal playing and testing, the existing binaries can be
used.

- Upgrade to Rust Edition 2021
- Upgrade 'rand' to 0.8.5
- Upgrade 'rand_chacha' to 0.3.1
- Upgrade 'if_chain' to 1.0.2
- Upgrade 'clap' to 3.2.8
- Upgrade 'crossbeam-channel' to 0.5.5
- Upgrade 'crossbeam-utils' to 0.8.10 (security fix)
- Upgrade 'rand_core' to 0.6.3 (security fix)

## March 15, 2021 - Rustic Alpha 1.1

This is a bugfix release. Alpha 1 lost all of its games on time forfeit
when playing in MoveTime mode (for example, when playing seconds/move).

Bugfixes:
- Do not exceed alotted time in MoveTime mode.

## January 24, 2021 - Rustic Alpha 1

This is the initial release.

[CCRL Blitz rating: +/- 1677 Elo](https://www.computerchess.org.uk/ccrl/404/cgi/engine_details.cgi?print=Details&each_game=1&eng=Rustic%20Alpha%201%2064-bit#Rustic_Alpha_1_64-bit)

Below are the features included in this version.

- Engine:
  - Bitboard board representation
  - Magic bitboard move generator
  - UCI-protocol
- Search
  - Alpha/Beta search
  - Quiescence search
  - MVV-LVA move ordering
  - Check extension
- Evaluation
  - Material counting
  - Piece-Square Tables