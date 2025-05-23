# IMap

A tool for creating source maps for deobfuscated files.

## Usage

```bash
imap <file> <line> <original> <new>
```

- `<file>`: The file to create a source map for.
- `<line>`: The line number in the file to create a source map for.
- `<original>`: The original identifier to map.
- `<new>`: The new identifier to map to.

Creates/Modifies the file at `(cwd)/source_maps/(file).json` with the following format:

```json
[
    {
        "file": "<file>",
        "line": <line>,
        "original": "<original>",
        "new": "<new>"
    }
]
```

## Installation

```bash
pnpm install
pnpm run build # for windows
pnpm run build:unix # for unix
npm link
```
