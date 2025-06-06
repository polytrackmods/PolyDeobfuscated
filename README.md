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

To rebase your changes with another branch, you can run the following command:

```bash
git rebase origin/main
```

To merge your changes with another branch, you can run:

```bash
git merge origin/main
```

By default, `mergiraf` will automatically attempt to resolve merge conflicts. To temporarily disable this feature, you can set the `mergiraf` environment variable to `0`:

```bash
mergiraf=0 git (rebase/merge) origin/main
```

You can abort the merge process at any time by running:

```bash
git merge --abort
```

You can review the changes made by `mergiraf` by running:

```bash
mergiraf review (review_name)
```

If you encounter a bug, please run:

```bash
mergiraf report (review_name/file_path)
```

You can manually resolve merge conflicts by running:

```bash
mergiraf solve (file_path)
```

## Confused? Have an Issue?

Join the [PolyTrack Discord](https://discord.gg/kzSNuh4ZTu) for help and support in the `# modding` channel. The community is active and can assist with any questions or issues you may encounter while working with the deobfuscated code.

## License

The deobfuscated code is licensed under the original PolyTrack license.
