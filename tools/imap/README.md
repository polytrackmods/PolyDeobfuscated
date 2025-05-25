# IMap (Identifier Mapper)

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

## Usage

1. Edit the obfuscated code in the `PolyTrack` folder as needed.
2. Run the `imap create` command to generate the source maps:

    ```bash
    imap create
    ```

3. The generated source maps will be placed in the `source_maps` folder.

## The Source Map

The source map is a JSON file that contains the mapping between the original code and the deobfuscated code. It includes the following fields:

```rust
struct Mapping {
    original: String, // The original identifier from the obfuscated code
    modified: String, // The deobfuscated identifier
    scope_id: u32, // The scope ID of the identifier
    id: usize, // The unique ID of the mapping
    declaration_type: String, // The type of declaration (e.g., "function", "variable", etc.)
}
```

## TODO

- Add support for other identifier declarations.
- Clean up the code.

## License

IMap is licensed under the [MIT License](https://opensource.org/licenses/MIT). See the [LICENSE](LICENSE) file for more information.
