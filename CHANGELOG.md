# Changelog

All notable changes to this project _should_ be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

This project also attempts to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(for whatever that's actually worth).

---

## [Planned]
- Cross-platform testing and compilation

---

## [2.0.0-beta.7] - 2023-03-03

### Added
- Added doc comments to all public modules, types, and functions
- Basic instructions for using `jrny` as a library instead of an executable

### Changed
- `crate::config` and `crate::environment` moved inside of `crate::context` with all types re-exported
- `crate::client` and `crate::executor` moved inside of `crate::db` and re-exported
- `jrny::plan` has a new `contents: Option<&str>` parameter to allow programmatically
writing a new, non-empty revision file

### Removed
- `Logger` moved from `crate::logger` to `main`

## [2.0.0-beta.6] - 2023-03-01

### Changed
- Converted `&PathBuf` parameters to `&Path` per clippy

## [2.0.0-beta.5] - 2021-10-25

### Changed
- Disabled now-default colorized output from clap

## [2.0.0-beta.4] - 2021-10-25

### Changed
- Updated all dependencies to latest, including clap 3 beta

## [2.0.0-beta.3] - 2021-10-25

### Added
- Can specify custom revisions directory in config file
- Can specify database url via environment file
- Can specify database url via command-line option

### Changed
- `jrny` now looks for database url in either environment file or command-line flag
- Library command functions (plan, etc.) now accept `Config` and `Environment` objects
where appropriate, rather than `&str` and `PathBuf` fields and making assumptions about
file and directory paths

### Removed
- Can no longer connect to database via environment variable
- Database strategy removed from config file

## [2.0.0-beta.2] - 2021-08-11

### Changed
- Help message printed instead of error when `jrny` is run without subcommand

## [2.0.0-beta.1] - 2021-04-16

### Changed
- **Breaking:** Use an explicit revision sequence, prepending revisions with an id before the timestamp

## [1.3.0] - 2020-12-18

### Added
- Changelog!
- Revision SQL template [includes begin & commit](https://github.com/kevlarr/jrny/pull/16/files#diff-402d559eb0a3ae778c2280bf3daddd645de5ee18fc9044396ca11bbe7035e981R8) by default

### Changed
- Transaction commands (`begin`, etc) no longer generate errors
- Whole revision file now batch executed via simpla query protocol

### Removed
- State-machine parser to split revision file into distinct statements to prepare
- Programmatic transactions
- `--commit` flag for `jrny embark`
- `unicode-segmentation` dependency

---

## [1.2.0] - 2020-12-16

### Added
- Dual licensing via MIT and Apache-2.0

### Changed
- All non-CLI logic moved from `main` to `lib`
- All dependencies updated

---

## [1.1.0] - 2020-12-15

### Added
-  Actual error types via `crate::error::Error` enum, rather than `Result<_, String>` everywhere

---

## [1.0.1] - 2020-12-05

### Fixed
- Revisions can have periods in the name portion (`<timestamp>.<name>.<sql`>) of the filename

---

## [1.0.0] - 2020-12-04

### Added
- All commands complete and operable on macOS
- Basic Github workflow to run unit tests
- Database connection via [connection string](https://github.com/kevlarr/jrny/pull/13), eg. `postgres://user:pass@host:port/name`
