# PolyDeobfuscated

Clean, readable, and deobfuscated source of PolyTrack, rebuilt for analysis, modding, and understanding.

## Tools

### [IMap](./tools/imap/)

IMap is a tool used for generating the source maps for PolyTrack. It can be used to deobfuscate the code and make it more readable. This is the main tool you will be using to work with the source code.

#### Usage

1. Follow the build instructions in the [IMap README](./tools/imap/README.md) to set up the tool.
2. Run the `imap update` command (from the project root) to generate your `temp` folder:

    ```bash
    imap update
    ```

3. Edit the obfuscated code in the `PolyTrack` folder as needed.
4. Run the `imap create` command to generate the source maps:

    ```bash
    imap create
    ```

5. The generated source maps will be placed in the `source_maps` folder.

#### Reminders

- Any time you pull changes from the upstream repository, you will need to run `imap update` again to update your `temp` folder.
- Do not commit the `temp` folder.

## General Reminders

- You are not modifying the structure of the code, just the obfuscated names.
- Do not push changes to the upstream repository. Instead, create a pull request to the repo.

## Confused? Have an Issue?

Join the [PolyTrack Discord](https://discord.gg/kzSNuh4ZTu) for help and support in the `# modding` channel. The community is active and can assist with any questions or issues you may encounter while working with the deobfuscated code.

## License

The deobfuscated code is licensed under the original PolyTrack license. Tools and scripts have their own licenses as specified in their respective directories.
