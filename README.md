# DF2

A tool to tell what's taking up so much disk space.

## Supported platforms

- MacOS

## Usage

### Examples

### Simple

```
df2 .
```

![example1](https://private-user-images.githubusercontent.com/51764318/460443627-19f4f8ba-0352-424d-96e6-77758cfba6d5.png?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NTEyNjcwMzEsIm5iZiI6MTc1MTI2NjczMSwicGF0aCI6Ii81MTc2NDMxOC80NjA0NDM2MjctMTlmNGY4YmEtMDM1Mi00MjRkLTk2ZTYtNzc3NThjZmJhNmQ1LnBuZz9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNTA2MzAlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjUwNjMwVDA2NTg1MVomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPTZjMWU0MWI5NGU0ODFhNmJkZDc3ZDBmMTYxYjQ3NWI2MWIwODVkZDRkZmRiMjE2MjIyYmM5MGRkYzQ0Y2Y4NmUmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.byKxamqwqjH4qb6cicK4iJ2GC11Sm4GbfIdFZFVfTPk)

### Interactive Mode

```
df2 -i .
```

![example2](https://private-user-images.githubusercontent.com/51764318/460444922-5faa47cb-4b1a-4e2b-9cbf-a54175b5f897.mov?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3NTEyNjcwMzEsIm5iZiI6MTc1MTI2NjczMSwicGF0aCI6Ii81MTc2NDMxOC80NjA0NDQ5MjItNWZhYTQ3Y2ItNGIxYS00ZTJiLTljYmYtYTU0MTc1YjVmODk3Lm1vdj9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNTA2MzAlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjUwNjMwVDA2NTg1MVomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWUyYzAxZDBlY2IxNzhlNDFlODBhMzU1Mzg4MTk2ZjEzNmEwNTM3YjY1MDIwMDE1MmZjYzJiYmMxN2ZmN2FlNjkmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0In0.KtHFFiqnGxFegLBiMKZUa7HZuokOknLfjPoB59bfPkw)

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
