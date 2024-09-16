# Rust Memory Allocator Visualizer

This program provides a real-time, interactive visualization of memory usage in Rust, focusing on stack and heap allocations.

## Features

- Visualizes stack and heap memory usage
- Allows dynamic allocation and deallocation of memory
- Shows stack growth through recursive function calls
- Real-time updates of memory usage

## Usage

Run the program and use the following commands:

- `a <size>`: Allocate memory of specified size (in bytes)
- `d`: Deallocate the most recently allocated memory
- `v`: Force update of the visualization
- `q`: Quit the program

## Visualization

The program displays:
- A bar graph representing stack and heap usage
- Detailed byte counts for stack and heap
- A visualization of stack growth

## Implementation

- Uses a custom global allocator to track heap usage
- Employs atomic operations for thread-safe memory tracking
- Utilizes ANSI escape codes for console-based visualization

## Requirements

- Rust (stable)
- Colored crate for terminal coloring

## Building and Running

```
cargo build --release
cargo run --release
```
