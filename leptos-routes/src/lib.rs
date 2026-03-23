//! Declarative, type-safe route definitions for the [Leptos](https://leptos.dev) web framework.
//!
//! Define routes as nested modules annotated with `#[route("/path")]` inside a
//! `#[routes]`-annotated root module. The macros generate route structs with `path()`,
//! `path_pattern()`, and `materialize()` methods, a `Route` enum covering all routes,
//! a `route_tree()` function that produces the Leptos Router view tree, and a `router()`
//! convenience function that wraps `route_tree()` in a `<Router>`.
//!
//! Use `layout!()`, `index!()`, and `page!()` macros in route modules to declare view components
//! with full IDE support (go-to-definition, autocomplete). Use `fallback!()` in the
//! root module to set the 404 handler. To generate only route structs without view
//! code, use `#[routes(without_views)]`.
//!
//! See [routes] for more information.

/// Re-exports all procedural macros from the implementation crate.
///
/// Users should depend on `leptos-routes`, not `leptos-routes-macro` directly.
pub use leptos_routes_macro::*;
