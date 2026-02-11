# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


<!-- next-header -->
## [3.1.1] - 2026-02-11

### 🏭 Refactor
- Remove auth caching, improve test config

### 📝 Documentation
- Described differences from upstreaam fork. by @TralahM

### :gear: Miscellaneous
- Update ci workflow to use new environment variables

## [3.1.0] - 2026-02-07

### :rocket: New Features
- Move from Mutex to RwLock for multiple readers of initiator password or one writer. by @TralahM

## [3.0.0] - 2026-02-07

### :rocket: New Features
- Thread-safety made the Mpesa client Send+Sync. by @TralahM

### :bug: Fixed
- Make Mpesa client Clone by @TralahM

### :gear: Miscellaneous
- Update release badge and replacements in docs by @TralahM

## [2.2.1] - 2026-02-07

### Added
- Add musl support and env setup for CI builds by @TralahM

## [2.2.0] - 2026-02-07

### :rocket: New Features
- Gate features and types with conditional compilation by @TralahM

## [2.1.1] - 2026-02-07

### Added
- Add secrets env for tests in CI workflow by @TralahM

## [2.1.0] - 2026-02-07

### Added
- Add secrets to CI and update README by @TralahM

## [2.0.2] - 2026-02-06

### :bug: Fixed
- Applied clippy fix suggestions by @TralahM

### 📝 Documentation
- Clarify API docs and update code examples by @TralahM

### :gear: Miscellaneous
- Ignore .envrc by @TralahM

## [2.0.1] - 2026-02-06

### :bug: Fixed
- Use correct macro path in test module by @TralahM

## [2.0.0] - 2026-02-06

### :rocket: New Features
- Make openssl dependency optional via feature flag by @TralahM
- Update rust edition by @itsyaasir
- Add M-Pesa Express Query service by @itsyaasir

### :bug: Fixed
- Update builder methods to include lifetime annotations by @itsyaasir

### Removed
- Remove CODECOV_TOKEN from GitHub workflow by @itsyaasir

### 📝 Documentation
- Current release. by @TralahM

### :gear: Miscellaneous
- Changelog. by @TralahM
- Cliff config. by @TralahM
- Release config. by @TralahM
- Clear ignored advisories in audit configuration by @itsyaasir
- Update GitHub workflow for code coverage by @itsyaasir
- Enhance GitHub workflows with concurrency settings by @itsyaasir
- Update GitHub workflows and add cargo audit configuration by @itsyaasir
- Follow API variable naming as per Daraja by @itsyaasir
- Move express-request and transaction reversal to builder pattern by @itsyaasir
- Generic `send` implementation by @itsyaasir

### New Contributors
* @TralahM made their first contribution
* @Tevinthuku made their first contribution

## [1.1.0] - 2023-11-14

### Added
- Add dynamic qr code by @itsyaasir

### :gear: Miscellaneous
- Implement auth caching by @itsyaasir

### New Contributors
* @Borwe made their first contribution

## [1.0.0] - 2023-08-26

### :bug: Fixed
- Run rust fmt ci error by @dxphilo
- Review changes by @dxphilo
- Rustfmt ci errors by @dxphilo
- Extract an api struct to store error details by @dxphilo

### New Contributors
* @crispinkoech made their first contribution
* @dxphilo made their first contribution
* @itsyaasir made their first contribution

## [0.3.5] - 2021-05-16

### :bug: Fixed
- Explicitly set env variables at runtime by @c12i
- Move CI env variables to global scope by @c12i
- Fix ci yaml by @c12i
- Fix ci workflow by @c12i

### :gear: Miscellaneous
- Update ci and release workflows by @c12i
- Update ci by @c12i
- Update release workflow by @c12i

### 🌀 Other
- Bump version to 0.3.5 by @c12i
- Modify access to the client auth method by @c12i

## [0.3.4] - 2021-04-06

### :rocket: New Features
- Update set_initiator_password to employ interior mutability pattern by @c12i
- Implement TryFrom for Environment by @c12i
- Update docs, and doc tests and bump version to 0.3.0 by @c12i
- Extract a MpesaExpressRequestResponse struct + getters and make it the return value for express_request service by @c12i
- Extract a AccountBalanceResponse struct + getters and make it the return value of account_balance service by @c12i
- Extract a C2bSimulateResponse struct + getters and make it the return value for c2b_simulate service by @c12i
- Extract a C2bRegisterResponse + getters and make it the return value for c2b_register service by @c12i
- Derive a B2bResponse struct + getters and make it the return value of b2b service by @c12i
- Add B2cResponse getters by @c12i
- Extract a custom B2cResponse type and make it the return value for b2c by @c12i
- Update docs by @c12i
- Implement mpesa express request/ stk push by @c12i
- Improve docs by @c12i
- Initiator password no longer a requirement, now created from within client + services docs improvement by @c12i
- Bump version to 0.2.0 by @c12i
- Complete Account Balance builder by @c12i
- Create B2B builder[untested] and add missing docs to B2C builder by @c12i
- Replace previous b2c method with B2cBuilder by @c12i

### :bug: Fixed
- Implement serde::derive on all payloads and cease usage of the json macro to construct a serialized value; instead we now send the payload, more clean code by @c12i
- Make is_connected method return boolean by @c12i
- Update mpesa_derive crate and bump version to 0.2.0 by @c12i
- Update .travis.yml tests by @c12i
- Update doc tests and docs by @c12i
- Add updated integration tests by @c12i
- Use response.status().is_succes() to check for successful requests by @c12i
- Make auth method private and extract a MpesaResult type alias by @c12i
- Rename payloads to services by @c12i

### Added
- Add optional set_initiator_password method for production usage by @c12i
- Add C2B Simulate builder by @c12i
- Add C2bRegister builder by @c12i
- Add b2b docs by @c12i

### Removed
- Remove redundant auth module from services and limit access to certain modules by @c12i
- Remove unused dependencies from mpesa_derive: TODO update once main crate is revamped by @c12i

### 📝 Documentation
- Update usage notes by @c12i
- Improve description of intiator_name by @c12i
- Update c2b_register docs by @c12i
- Update b2c docs by @c12i
- Add B2cBuilder, auth and b2c method docs by @c12i

### :gear: Miscellaneous
- Update express_request getters to return string slices by @c12i
- Update release workflow by @c12i
- Bump version by @c12i
- Update docs and bump version to 0.3.1 by @c12i
- Derive Clone trait for response types and anotate lifetimes of getter return values by @c12i
- Update docs by @c12i
- Re-enable b2c tests by @c12i
- Cleanup on account_balance tests by @c12i
- Code cleanup with cargo fmt by @c12i
- Use version 0.2.1 of mpesa_derive and update docs by @c12i
- Clean up and docs by @c12i
- Clean up unused imports by @c12i
- Cargo fmt by @c12i
- Format codebase with cargo by @c12i

### 🌀 Other
- Depreciate .urls and .parties methods and replace with individual method calls to add urls and party a/b by @c12i
- Intoduce MpesaError enum and implement on auth and FromStr implementation of the Environment enum by @c12i

### New Contributors
* @c12i made their first contribution

<!-- next-url -->
[3.1.1]: https://github.com/tralahm/mpesa-rust/compare/v3.1.0...v3.1.1
[3.1.0]: https://github.com/tralahm/mpesa-rust/compare/v3.0.0...v3.1.0
[3.0.0]: https://github.com/tralahm/mpesa-rust/compare/v2.2.1...v3.0.0
[2.2.1]: https://github.com/tralahm/mpesa-rust/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/tralahm/mpesa-rust/compare/v2.1.1...v2.2.0
[2.1.1]: https://github.com/tralahm/mpesa-rust/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/tralahm/mpesa-rust/compare/v2.0.2...v2.1.0
[2.0.2]: https://github.com/tralahm/mpesa-rust/compare/v2.0.1...v2.0.2
[2.0.1]: https://github.com/tralahm/mpesa-rust/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/tralahm/mpesa-rust/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/tralahm/mpesa-rust/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/tralahm/mpesa-rust/compare/0.4.2...v1.0.0
[0.3.5]: https://github.com/tralahm/mpesa-rust/compare/0.3.4...v0.3.5

<!-- generated by git-cliff -->
