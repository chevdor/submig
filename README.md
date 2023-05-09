# submig

`submig` is an experimental project aiming at finding the list of active Migrations for a Polkadot/Substrate runtime. The cli is mainly a an interface on top of the [`submig-lib crate`](https://crates.io/crates/submig-lib).

## Usage

### Help
```source,bash
submig --help
```

### List

```source,bash
submig list /path/to/repo
```

Alternatively, if you defined the `POLKADOT_REPO` ENV in your system, you can simply call:
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
