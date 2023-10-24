[![Quick check](https://github.com/chevdor/submig/actions/workflows/quick-check.yml/badge.svg?branch=master)](https://github.com/chevdor/submig/actions/workflows/quick-check.yml)

# submig

`submig` is an experimental project aiming at finding the list of active Migrations for a Polkadot/Substrate runtime. The cli is mainly a an interface on top of the [`submig-lib crate`](https://crates.io/crates/submig-lib).

`submig` parses the Rust code in order to find the Migrations and is thus subject to breaking if the structure of the code would happen to regarding how the Migrations are defined.

This tool is provided as an experiment. You should probably not rely on it too heavily.

## Usage

### Install

You can install from crates.io:
```
cargo install submig
```

Or from the repo:
```
cargo install --git https://github.com/chevdor/submig
```

### Help
```source,bash
submig --help
```

### List

```source,bash
submig list /path/to/repo
```

Alternatively, if you defined the `REPO_POLKADOT` ENV in your system, you can simply call:
```source,bash
submig list
```

Here is a sample output:
```
Checking migrations in repo: /projects/polkadot
/projects/polkadot/runtime/polkadot/src/lib.rs:
  - V0940
  - V0941
  - V0942
  - Unreleased
/projects/polkadot/runtime/rococo/src/lib.rs:
  - V0940
  - V0941
  - V0942
  - Unreleased
/projects/polkadot/runtime/kusama/src/lib.rs:
  - V0940
  - V0941
  - V0942
  - Unreleased
/projects/polkadot/runtime/westend/src/lib.rs:
  - V0940
  - V0941
  - V0942
  - Unreleased
  ```

You can also get the result for a single runtime using:
```source,bash
submig list -p runtime/polka
```

The output would then become only:
```
Checking migrations in repo: /projects/polkadot
/projects/polkadot/runtime/polkadot/src/lib.rs:
  - V0940
  - V0941
  - V0942
  - Unreleased
```
