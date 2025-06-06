# PolyDeobfuscated

Clean, readable, and deobfuscated source of PolyTrack, rebuilt for analysis, modding, and understanding.

## Prerequisites

- [Rustup](https://rustup.rs/) - Install Rust and Cargo
- [Mergiraf](https://mergiraf.org/installation.html#from-source) - For resolving merge conflicts

## How to deobfuscate

1. Clone the PolyTrack repository:

   ```bash
   git clone
   ```

2. Use the VSCode renaming tool to rename the obfuscated variables and functions. This tool will help you systematically rename the identifiers in the codebase.
    - Please do this in the polytrack-deobfuscated directory.
    - Try not to modify the structure of the code, as it may lead to merge conflicts later.
3. After renaming, run the following command to format the code:

    ```bash
    cd polytrack-deobfuscated
    npx prettier --write .
    ```

4. Create/Update your branch and create a pull request (You can also make your own fork of the repository and push your changes there):

    ```bash
    git checkout -b (your-branch-name)
    git add .
    git commit -m "(your commit message, try to follow https://www.conventionalcommits.org/en/v1.0.0/ and be descriptive)"
    git push origin deobfuscated
    ```

## Resolving Merge Conflicts

First, setup the `Mergiraf` tool:

```bash
git config --global merge.mergiraf.name mergiraf
git config --global merge.mergiraf.driver 'mergiraf merge --git %O %A %B -s %S -x %X -y %Y -p %P -l %L'
git config --global core.attributesfile .gitattributes
```

`Mergiraf` will automatically resolve merge conflicts in the deobfuscated code... for more information, refer to the [Mergiraf documentation](https://mergiraf.org/usage.html).

## Confused? Have an Issue?

Join the [PolyTrack Discord](https://discord.gg/kzSNuh4ZTu) for help and support in the `# modding` channel. The community is active and can assist with any questions or issues you may encounter while working with the deobfuscated code.

## License

The deobfuscated code is licensed under the original PolyTrack license.
