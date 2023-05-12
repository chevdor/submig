# submig

This project is an experimental cli on top of the the `submig-lib` crate.
It helps finding Polkadot Migrations.

Sample output:
```
Checking migrations in repo: /projects/polkadot
/projects/polkadot/runtime/westend/src/lib.rs:
  - ✅ V0940
  - ✅ V0941
  - ✅ V0942
  - ✅ Unreleased
/projects/polkadot/runtime/rococo/src/lib.rs:
  - ✅ V0940
  - ✅ V0941
  - ✅ V0942
  - ✅ Unreleased
/projects/polkadot/runtime/polkadot/src/lib.rs:
  - ✅ V0940
  - ✅ V0941
  - ✅ V0942
  - ✅ Unreleased
/projects/polkadot/runtime/kusama/src/lib.rs:
  - ✅ V0940
  - ✅ V0941
  - ✅ V0942
  - ✅ Unreleased
  - ❌ InitMigrate
```
