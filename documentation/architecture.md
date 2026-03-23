# Architecture

This document explains the non-obvious design decisions in `leptos-routes`: the two-phase macro
expansion, the import chain mechanism, and the struct shadowing problem that shapes how type checking
works.

## Crate structure

- **`leptos-routes`** — Public crate. Re-exports all proc macros via `pub use leptos_routes_macro::*;`.
- **`leptos-routes-macro`** — Proc-macro crate with all implementation logic.
  Exports six proc macros: `routes`, `route`, `fallback`, `layout`, `index`, `page`.

Users only need `use leptos_routes::routes;`. All other macros are injected automatically inside
a `#[routes]`-annotated module.

## Two-phase expansion

### Phase 1: `#[routes]` expands

The compiler processes `#[routes]` first because it is an attribute macro on the outermost module.
It receives the **raw token stream** of the entire module hierarchy, including unexpanded `layout!()`,
`index!()`, `page!()`, and `fallback!()` invocations.

`#[routes]` performs these steps:

1. **Extracts the fallback expression** — Scans the root module for `fallback!(...)`, parses it,
   and stores it. The invocation itself is left in place for Phase 2.

2. **Walks the module tree** — Recursively visits each `#[route]`-annotated child module:
    - Parses and validates the route path.
    - Injects `use super::*;` for import propagation (see below).
    - Extracts `layout!()`, `index!()`, `page!()` expressions (left in place for Phase 2).
    - Builds an internal `RouteDef` tree.

3. **Validates** — Path syntax, body macro conflicts, parameter collisions, etc.

4. **Generates code** — Per-route structs + impls, `Route` enum, `route_tree()`/`router()`.

5. **Emits** — The modified module with original items (body macros still unexpanded) plus
   generated code.

### Phase 2: Body macros expand

After `#[routes]` finishes, the compiler expands the remaining macros. Each body macro generates
a hidden function:

```rust
#[doc(hidden)]
#[allow(dead_code)]
fn __leptos_routes_layout() {
    let _ = MainLayout;
}
```

These serve **one purpose: IDE support**. The `let _ = MainLayout;` expression gives rust-analyzer
go-to-definition and autocomplete. The functions are never called and are stripped as dead code.

The body macros do **not** verify `ChooseView` — that happens in Phase 1's generated struct methods
(see "Struct shadowing problem" below).

## Import chain

`#[routes]` injects imports at the top of the root module:

```rust
use super::*;                    // component functions from the enclosing scope
use ::leptos_routes::route;      // so preserved #[route] attributes resolve
use ::leptos_routes::layout;     // so layout!() invocations resolve
use ::leptos_routes::index;      // so index!() invocations resolve
use ::leptos_routes::page;       // so page!() invocations resolve
use ::leptos_routes::fallback;   // so fallback!() invocations resolve
```

Each child `#[route]` module also gets `use super::*;` injected. This creates a chain
that propagates all imports downward:

```
outer scope (component functions defined here)
    mod routes
        use super::*;              <- brings in component functions
        use ::leptos_routes::layout; (etc.)
        mod users
            use super::*;          <- chains from routes module
            mod user
                use super::*;      <- chains from users module
```

## Generated code

For each route, a unit struct is generated in the route's parent module with `path()`,
`path_pattern()`, `materialize()`, and (when views are enabled) component accessor methods.
Child routes call their parent's `materialize()` to build full URLs recursively.

A `Route` enum at the root module level has one variant per route with `path_pattern()`,
`all()`, and `Display`. When views are enabled, `route_tree()` and `router()` functions
produce the Leptos Router view tree.

## Struct shadowing problem

This is the central design constraint that explains why type checking is split across the two phases.

The generated struct methods enforce `ChooseView + Clone + 'static`:

```rust
pub fn layout(&self) -> impl ::leptos_router::ChooseView + Clone + 'static {
    #[allow(clippy::wildcard_imports)]
    use super::super::super::*;
    MainLayout
}
```

The deep `use super::super::...::*;` chain escapes the module tree entirely, reaching the outer
scope where component functions are defined. The number of `super`s equals the module nesting
depth plus one.

**Why body macros cannot do this instead:**

1. `#[routes]` generates `struct User;` inside `mod user` (PascalCase of the module name).
2. `mod user` has `use super::*;` which imports the component function `User`.
3. The struct definition takes precedence over the glob import — inside `mod user`, `User`
   resolves to the struct, not the component function.
4. When `index!(User)` expands in Phase 2, the `User` token resolves in `mod user`'s scope
   where the struct shadows the function.

Due to proc macro span hygiene, the token's name resolution is tied to its original span context.
Neither a function-level import nor a qualified path in the body macro expansion can override this.

The struct methods avoid this because both the `use super::...::*;` import and the expression
token are part of the same `#[routes]` expansion context, and the deep super chain reaches past
the shadowing module.

| Concern                        | Where it happens                              |
|--------------------------------|-----------------------------------------------|
| IDE support (go-to-definition) | Body macro hidden functions (`let _ = expr;`) |
| `ChooseView` type enforcement  | Struct methods generated by `#[routes]`       |
| Router markup generation       | `route_tree()` in `generate/router.rs`        |
