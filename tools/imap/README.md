# IMap

A tool to create source maps from deobfuscated JavaScript/TypeScript code.

## Pre-requisites

- Rust via [rustup](https://rustup.rs/)

## Building

To build the tool, run the following command in the directory containing the `Cargo.toml` file:

```bash
cargo build --release
```

This will create a release build of the tool in the `target/release` directory.
The binary will be located at `target/release/imap`.

## Running

To use the tool, from the project's root, run the following command:

```bash
./tools/imap/target/release/imap --help
```

This will display the help message, which includes information on how to use the tool and its available options.

You can optionally add the build directory to your `PATH` environment variable to make it easier to run the tool without specifying the full path, and then run it like this:

```bash
imap --help
```

### Subcommands

- `create`: Create a source map from deobfuscated JavaScript/TypeScript code.
- `update`: Update the temp directory with the latest code.

### Options

- `create`:
  - `-c, --code-dir <CODE_DIR>`: The directory containing the code. Defaults to `./PolyTrack`.
  - `-t, --temp-dir <TEMP_DIR>`: The directory which stores the temporary files. Defaults to `./temp`.
  - `-s, --source-map-dir <SOURCE_MAP_DIR>`: The directory which stores the source maps. Defaults to `./source_maps`.
- `update`:
  - `-c, --code-dir <CODE_DIR>`: The directory containing the code. Defaults to `./PolyTrack`.
  - `-t, --temp-dir <TEMP_DIR>`: The directory which stores the temporary files. Defaults to `./temp`.

## Usage

First, you must create the temp directory based on your current code.

```bash
./tools/imap/target/release/imap update
```

After that, rename an identifier declaration in the code, and run the following command to create a source map:

```bash
./tools/imap/target/release/imap create
```

**Reminder**: If you add a comment or anything that is not renaming an identifier, you must run the `update` command again to update the temp directory with the latest code. Do not update two identifiers at the same time.

The source map for the file will be created in the `source_maps` directory, and the temp directory will be updated with the latest code.

### The Source Map

The source map is a JSON file that contains the mapping between the original code and the deobfuscated code. It includes the following fields:

- `original`: It is in the format of `(original_identifier):(scope_id):(unique_id)`.
- `new`: It is in the format of `(new_identifier):(scope_id):(unique_id)`.

## Extra Notes

Some identifier declarations are not tracked. We are trying to add tracking to every possible way an identifier declaration can be renamed.

## License

IMap is licensed under the [MIT License](https://opensource.org/licenses/MIT). See the [LICENSE](LICENSE) file for more information. This does not mean that the actual deobfuscated code is licensed under the MIT license.
