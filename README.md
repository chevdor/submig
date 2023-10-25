[![Quick check](https://github.com/chevdor/submig/actions/workflows/quick-check.yml/badge.svg?branch=master)](https://github.com/chevdor/submig/actions/workflows/quick-check.yml)

# submig

`submig` is an experimental project aiming at finding the list of active Migrations for a Polkadot/Substrate runtime. The cli is mainly a an interface on top of the [`submig-lib crate`](https://crates.io/crates/submig-lib).

`submig` parses the Rust code in order to find the Migrations and is thus subject to breaking if the structure of the code would happen to regarding how the Migrations are defined.

This tool is provided as an experiment. You should probably not rely on it too heavily.
The rules related to what makes a Migration "valid" are also very opinionated and may not suit every project.

## Rules

Since migrations need to be managed in a rolling manner (ie added and removed over time), having a simple list of
migrations such as the following is not ideal:

```
/// Migrations to apply on runtime upgrade.
pub type Migrations = (
	// v9420
	pallet_nfts::migration::v1::MigrateToV1<Runtime>,
	// unreleased
	pallet_collator_selection::migration::v1::MigrateToV1<Runtime>,
	// unreleased
	migrations::NativeAssetParents0ToParents1Migration<Runtime>,
	// unreleased
	pallet_multisig::migrations::v1::MigrateToV1<Runtime>,
	// unreleased
	InitStorageVersions,
);
```

Instead, the following makes it much easier to track migrations down through each release:
```
pub type Migrations = (
  migrations::Unreleased,
  migrations::V9430,
  migrations::V9420,
);
```

Say the next release is V9440, the current `migrations::Unreleased` can be renamed to `migrations::V9440` and added to
the list while a new empty tuple can now be created to hold the future migrations: `pub type migrations::Unreleased =
()`.

The new migration plan becomes:
```
pub type Migrations = (
  migrations::Unreleased,
  migrations::V9430,
  migrations::V9440,
  migrations::V9420,
);
```

and it becomes easy now to remove all the `migrations::V9420` migrations after N releases.

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

Alternatively, if you defined the `REPO_POLKADOT_SDK` ENV in your system, you can simply call:
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
