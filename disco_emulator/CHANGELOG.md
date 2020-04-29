# Changelog (disco-emulator)

## 1.2.1 - 2020-04-29

### Fixed

- Only accept first audio handler, so multiple `bl init` calls don't break & sample plotter works

## 1.2.0 - 2020-04-29

### Changed

- Version info now sourced from Cargo.toml file. Fixes inconsistent `--version` results.
- Audio now dynamically spawned when `init` or `audio_init` is called.
- Scratch registers (r0--r3, r12) given random (but still consistent) values after call to audio init and play functions. This minimises emulator-only solutions that don't respect the calling convention.

### Fixed

- Fix positive-or-zero condition check

## 1.1.2 - 2020-04-03

### Fixed

- Don't crash when audio sent to observer fails.


## 1.1.1 - 2020-03-15

### Added

- Change log started.
