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

The source map for the file will be created in the `source_maps` directory, and the temp directory will be updated with the latest code.

## Reminders

- If you add a comment or anything that is not renaming an identifier, you must run the `update` command again to update the temp directory with the latest code.
- Do not modify the structure of the obfuscated code. The tool relies on the structure of the code to create accurate source maps.

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

IMap is licensed under the [MIT License](https://opensource.org/licenses/MIT). See the [LICENSE](LICENSE) file for more information. This does not mean that the actual deobfuscated code is licensed under the MIT license.
