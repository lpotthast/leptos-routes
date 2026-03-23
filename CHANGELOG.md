# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.4.1] - 2026-03-23

### Fixed

- `layout`, `index` and `page` method's return type is now just `impl ::leptos_router::ChooseView`. `ChooseView` already
  specifies required bounds (`Send + Clone + 'static`). Re-specifying them ourselves would lead to
  [clippy:implied_bounds_in_impls](https://rust-lang.github.io/rust-clippy/rust-1.94.0/index.html#implied_bounds_in_impls)
  errors being emitted by clippy.
- Compatibility message for old code still specifying `#[routes(with_views)]`

## [0.4.0] - 2026-03-23

### Added

- `layout!()`, `index!()`, `page!()`, and `fallback!()` macros for declaring view components inside route
  modules with full IDE support (go-to-definition, autocomplete). Replaces the old attribute-based syntax
  (`#[route("/path", layout = "...", view = "...", fallback = "...")]`).
- `layout!()`, `index!()`, `page!()`, and `fallback!()` can be used directly in the `#[routes]` module body, creating a
  synthetic root route without needing an explicit `#[route("/")]` wrapper module.
- `path = "/..."` argument on `#[routes]` for mounting an entire route tree under a common prefix.
- `path_pattern()` method on route structs returning the full concatenated path from root (e.g., `/users/:id/details`).
- `router()` convenience function wrapping `route_tree()` inside `<Router>`.
- Implicit `<Outlet/>` passthrough for parent routes without `layout!()`.
- Parameter name collision detection across the route hierarchy with suggestions for prefixed alternatives.
- Path validation for `#[routes(path = "...")]` argument (no trailing slash, no double slashes).
- Additional trybuild tests covering struct generation, view rendering, and error diagnostics.
- Example projects (`examples/basic/`, `examples/routes-only/`).
- Architecture documentation (`documentation/architecture.md`).
- Justfile for common development tasks.
- CLAUDE.md project guidance.
- CHANGELOG.md.

### Changed

- **Breaking:** View components are now declared via `layout!()`, `index!()`, `page!()`, and `fallback!()` macros in the
  module body instead of `layout`, `view`, and `fallback` arguments on `#[route(...)]` attributes.
- **Breaking:** The `fallback` argument moved from `#[routes(fallback = ...)]` to a `fallback!()` macro in the module
  body.
- **Breaking:** View generation is now the default; opt out with `#[routes(without_views)]`. A `fallback!()` is required
  when views are generated. The old `#[routes(with_views)]` opt-in argument has been removed.
- **Breaking:** Upgrade to Leptos 0.8 and `leptos_router` 0.8.
- **Breaking:** MSRV bumped to 1.88.
- Rename `generated_routes()` to `route_tree()`.
- Improved error diagnostics with migration guidance for deprecated attribute arguments.

### Removed

- `darling` and `uuid` dependencies.
- `leptos_router` runtime dependency from the proc-macro crate.

## [0.3.1] - 2025-02-23

### Changed

- Move shared package settings to workspace `Cargo.toml`.

### Fixed

- Allow `module_inception` lint on any `#[routes]` annotated module, allowing an inline `routes` mod to be placed in a
  `routes.rs` without experiencing clippy warnings.

## [0.3.0] - 2025-02-23

### Changed

- **Breaking:** Update to Rust edition 2024.
- **Breaking:** Bump MSRV to 1.85.0.
- Remove lockfile from repository.

## [0.2.0] - 2025-02-14

### Added

- Router view generation: `generated_routes()` function producing `<Route>`/`<Outlet/>` Leptos router markup.
- Compile-time path validation checks.

### Fixed

- Root route `"/"` materialization now correctly yields `"/"` instead of `""`, ensuring absolute paths.

### Changed

- Drop `itertools` dependency.

## [0.1.0] - 2025-01-04

### Added

- Initial release.
- Declarative route definitions via `#[routes]` and `#[route]` proc macros.
- Route struct generation with `path()` and `materialize()` methods.
- `Route` enum with a variant per route.
- Recursive `materialize()` building full URLs through the parent chain.

[Unreleased]: https://github.com/lpotthast/leptos-routes/compare/v0.4.1...HEAD

[0.4.1]: https://github.com/lpotthast/leptos-routes/compare/v0.4.0..v0.4.1

[0.4.0]: https://github.com/lpotthast/leptos-routes/compare/v0.3.1...v0.4.0

[0.3.1]: https://github.com/lpotthast/leptos-routes/compare/v0.3.0...v0.3.1

[0.3.0]: https://github.com/lpotthast/leptos-routes/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/lpotthast/leptos-routes/compare/v0.1.0...v0.2.0

[0.1.0]: https://github.com/lpotthast/leptos-routes/releases/tag/v0.1.0
