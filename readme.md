[https://rustic-chess.org/](https://rustic-chess.org/)

# Rustic Chess Engine

Rustic is a chess engine written from scratch in the Rust programming
language.

This repo has been extended to change the engine focus to moves which
the opponent has limited replies to in order to gain or maintain an
advantage.

I do not take credit for any code other than in this fork.

# Noted Improvements

### July 5
- Implemented mobility evaluation in position assessment.
- ELO +200
- 1950 on Lichess

### June 30 
- Fixed critical bug causing evaluation to send "0000" when time slices ran out. Approx. 300 ELO jump.
- ELO ~ 1800 on Lichess BOTS


# Kinda to do / known problems / I'm tired and don't have my pc / I'm in work and I'm looking at games 

it's inherently bad to search for opponents moves and chose better ones for them, so it's never gonna hold up in engine tournaments (unless enter with margin=0?)

eval is quite bad regardless so sometimes it does make mistakes 

it seems in low time, the eval drops out of UCI logs but it still keeps good moves? 

it's officially better than me, consistently 

depth for sharp move in a good spot, when eval gets faster we can try look further 


# User interface

The engine does not provide its own user interface. It uses the UCI and
XBoard protocols to communicate with graphical user interfaces. (XBoard is
not yet completely implemented at the time of writing. For now, use the
engine in UCI mode, which is also the default.) It is recommended that you
use a GUI to play games against the engine. Rustic is tested with these
user interfaces:

- [Arena Chess GUI](http://www.playwitharena.de/)
- [XBoard/Winboard](https://www.gnu.org/software/xboard/FAQ.html)
- [CuteChess](https://cutechess.com/)
- [Tarrasch](https://www.triplehappy.com/)
- [The Shredder GUI](https://www.shredderchess.com/)
- [Fritz / Chessbase series](https://en.chessbase.com/)
- [Scid vs PC (database)](http://scidvspc.sourceforge.net/)
- [Banksia GUI](https://banksiagui.com/)

There are many other user interfaces that will probably work just fine, but its
obviously impossible to test all of them. If you have problems with a user
interface, open an issue so I can see if it can be fixed. (Assuming the
user interface is either free or open source, as I can't go and buy GUI's
just for testing purposes.)

# Features

The current feature-set for Rustic Alpha 1.0.0 is:

- Engine:
  - Bitboard board representation
  - Fancy Magic bitboard move generator
  - Transposition Table
  - UCI-protocol
  - Multi-threaded search
- Search
  - Alpha/Beta search
  - Quiescence search
  - Check extension
  - PVS
- Move ordering
  - TT Move priority
  - MVV-LVA
  - Killer moves
- Evaluation
  - Material counting
  - Piece-Square Tables

(See changelog.md for more information.)

# Included binaries, supported platforms

Each release contains several binaries; these are compiled for different
types of CPU's. The following binaries are supplied:

- bmi2: Intel Haswell (2013), AMD Zen 3 (2020)
- popcnt: Intel Nehalem (2008), AMD Bulldozer (2011), AMD Zen 1 and 2 (2017)
- old: Intel Core2 Duo (2006) and AMD CPU's between 2003 and 2011
- ancient: For the very first 64-bit CPU's (2003)
- i686: Intel Pentium II (1998) (Windows only), 32-bit
- arm: Raspberry Pi 3 and 4 (not tested on RPi 1 and 2)

Windows still supports a 32-bit executable. Note that this executable is at
50% slower (half the speed) as compared to the 64-bit one and not as well
tested. Many Linux-distributions have dropped support for 32-bit native
Linux software, so no 32-bit executable for Linux is provided. The same is
true for MacOS. The executable for the Raspberry Pi will be 32-bit as long
as the 32-bit version of Raspbian OS is the default and 64-bit is
experimental.

You can use the binary which runs fastest on your particular system for
maximum playing strength. Start a terminal, and run each Rustic version:

```
$ ./<executable_name> -p7 -h512
```

This will run perft 7 from the starting position, using a 512 MB
transposition table. Pick the version that runs perft 7 the fastest. If a
binary crashes, your CPU does not support the required instructions to run
it. Try a different binary.

If you wish to run Rustic on a system for which no binary is supplied, you
can try to compile the engine yourself using the compilation tips below.
Make sure to install at least Rust version 1.46.

# Quick compiling tips

The engine includes a Makefile since Rustic Alpha 3.0.0, which makes
building the engine easier. If you wish to build the engine yourself, you
can find some quick tips below. These are meant for people who have some
experience with setting up build environments, or have them installed
already. If more information is required, see the file "build.md", or
Rustic's documentation.

## Build environment

- [Install Rust for your platform](https://www.rust-lang.org/tools/install)
- Windows: [Install MSYS2](https://www.msys2.org/)
  - Make sure you install the following parts:
    - CoreUtils
    - BinUtils
    - GCC
    - Make
- MacOS:
  - [Install HomeBrew](https://brew.sh/)
  - Install GNU Make
  - The command is "gmake" instead of "make", as MacOS includes its own
    (very old) version of "make".
- Make sure you install the correct Rust target for your platform, using
  Rustup:
  - Linux:
    - __stable-x86_64-unknown-linux-gnu__
  - Windows:
    - __stable-x86_64-pc-windows-gnu__ (This toolchain creates compiles that
      are compatible with the GNU GDB-debugger.)
    - __stable-x86_64-pc-windows-msvc__ (This toolchain creates compiles that
      are compatible with Windows/Visual Studio's debugger. This will
      require the Microsoft Visual C++ Build Tools, because it uses the
      Microsoft Linker.)
- Install Git for your platform.

## Building Rustic

  - Start the terminal for your platform: MSYS2 (MinGW64 or MinGW32
    version) for Windows, Terminal on Mac, and for Linux, your favorite
    terminal emulator.
  - Clone Rustic: "git clone https://github.com/mvanthoor/rustic.git"
  - Switch to the "rustic" folder.
  - Run "make" (Windows, Linux) or "gmake" (MacOS).
  - A ./bin folder should be created. The Makefile will build all versions
    of Rustic for the operating system and CPU you're running on.

# Extra module

There is a module called "Extra", which copmiles some extra capabilities
into the Rustic executable.

- Command-line option -e: Rustic can run a perft suite containing 172
  tests, to see if its move generator, make, and unmake are working as
  intended. This is mainly useful for developers.
- Command-line option -w: Using this option, Rustic can perform Wizardry:
  it runs a function that generates magic numbers for use in a magic
  bitboard engine which has square A1 = 0, or LSB, and square H8 = 63. This
  is mainly useful if one wants to write their own chess engine, bus has no
  interest in writing a function to compute the magic numbers. (Though,
  doing so, will make understanding of magic bitboards much more complete.)

This module can be included by using the --features option of cargo:

```
cargo build --release --features "extra"
```

# All command-line options

```
USAGE:
    rustic.exe [FLAGS] [OPTIONS]

FLAGS:
        --help        Prints help information
    -k, --kiwipete    Set up KiwiPete position (ignore --fen)
    -q, --quiet       No intermediate search stats updates
    -V, --version     Prints version information

OPTIONS:
    -c, --comm <comm>          Select communication protocol to use [default: uci]  [possible values: uci, xboard]
    -f, --fen <fen>            Set up the given position [default: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -
                               0 1]
    -h, --hash <hash>          Transposition Table size in MB [default: 32]
    -p, --perft <perft>        Run perft to the given depth [default: 0]
    -t, --threads <threads>    Number of CPU-threads to use [default: 1]
```
The thread count determines how many search workers Sharp-Rustic will start. Each
worker runs on a clone of the current board and shares the transposition table.
For example, running `./rustic -t 4` spawns four worker threads.

Please note that the -e (--epdtest) and -w (--wizardry) options are only
available if the "extra" module is compiled into the engine.

# Credits

Firstly, the creator of Rustic. This fork has allowed me to enjoy
problem solving again. 

The OG Creator's credits can be found in "credits.md", or in [Rustic's
documentation](https://rustic-chess.org/back_matter/credits.html).


