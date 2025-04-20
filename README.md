# Ambition Tracker

A simple web application to track your daily motivation and productivity levels.

## Description

Ambition Tracker is a Rust-based web application that allows you to record whether you're feeling productive or non-productive each day. It stores this information in a SQLite database and provides a simple web interface to view and update your mood.

Key features:
- Record your daily mood (Motivated or Unmotivated)
- Only one entry per day (updates if you change your mind)
- Automatic tracking of missing days (adds "Unmotivated" entry if you miss a day)
- Simple and clean web interface

## Installation

### Prerequisites

- Rust and Cargo (https://www.rust-lang.org/tools/install)
- For MUSL builds: `rustup target add x86_64-unknown-linux-musl`

### Building from Source

Clone the repository and build the application:

```bash
git clone <repository-url>
cd ambition-rs
cargo build --release
```

The executable will be available at `target/release/ambition`.

### Using the Makefile

The project includes a Makefile with several useful targets:

```bash
# Build in debug mode
make build

# Build in release mode
make release

# Build with MUSL target (statically linked)
make musl

# Run the application
make run

# Clean build artifacts
make clean
```

## Usage

Run the application:

```bash
./target/release/ambition
```

Then open your browser and navigate to:

```
http://localhost:3000/ambition/
```

The web interface allows you to:
- See your current mood
- Set your mood for today (Productive or Non-Productive)
- Get feedback when you update your mood

## Technical Details

- Built with Rust and the Axum web framework
- Uses SQLite for data storage
- Implements a scheduler to automatically track missing days
- Stores dates in YYYY-MM-DD format
- Provides a responsive web interface

## License

MIT