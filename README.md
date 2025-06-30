# DF2

A tool to tell what's taking up so much disk space.

## Supported platforms

- MacOS

## Usage

### Simple

```
df2 .
```

<img width="720" alt="Image" src="https://github.com/user-attachments/assets/19f4f8ba-0352-424d-96e6-77758cfba6d5" />

### Interactive Mode

```
df2 -i .
```

https://github.com/user-attachments/assets/d5ce0703-2339-4e71-8655-b6a96fd3bafa

### Help

```
$> df2 --help

Calculate the size of a directory

Usage: df2 [OPTIONS] [DIRECTORY]

Arguments:
[DIRECTORY] Directory to scan [default: .]

Options:
-l, --list-items List all directories and files in the directory after scanning
-i, --interactive-mode Cache the scan results and allow further traversal
-w, --width <WIDTH> Max chart width [default: 100]
-f, --full Use full width of the terminal
-v, --verbose Log all errors
-h, --help Print help
-V, --version Print version

```

## Building

```
cargo run -- [args]
```

## Installation

Build from source: (run in repo root)

```
cargo install --path .
```
