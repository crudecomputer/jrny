# Changelog

All notable changes to this project _should_ be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project _attempts_ to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Planned]
- Cross-platform testing and compilation
- Enforcement of explicit revision sequence

---

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
