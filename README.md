# leptos-routes

[![crates.io](https://img.shields.io/crates/v/leptos-routes.svg)](https://crates.io/crates/leptos-routes)
[![docs.rs](https://docs.rs/leptos-routes/badge.svg)](https://docs.rs/leptos-routes)
[![CI](https://github.com/lpotthast/leptos-routes/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/leptos-routes/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/rust-1.88%2B-blue.svg)](https://www.rust-lang.org/)

Declaratively define the routes for your Leptos project.

## Why?

Standard Leptos routing uses untyped path strings — there's no way to **refer** to a route from elsewhere in your app
without hand-writing URLs. `leptos-routes` generates a struct per route so you get type-safe `path()` for `<Route>`
declarations and `materialize()` for building links, plus optional full router generation.

## Example

Define the routes of your app in their own `routes.rs`:

```rust
// src/routes.rs
use leptos_routes::routes;

#[routes]
mod defs {
    use crate::pages;

    // Everywhere a `fallback!`, `layout!`, `index!` or `page!` macro is used,
    // you can pass any view, so also `|| view! { ... }` closures!
    fallback!(pages::Err404);

    layout!(pages::MainLayout);
    index!(pages::Dashboard);

    #[route("/welcome")]
    mod welcome {
        page!(pages::Welcome);
    }

    #[route("/users")]
    mod users {
        layout!(pages::UsersLayout);
        index!(pages::NoUser);

        #[route("/:id")]
        mod user {
            layout!(pages::UserLayout);
            index!(pages::User);

            #[route("/details")]
            mod details {
                page!(pages::UserDetails);
            }
        }
    }
}
pub use defs::*;
```

Then render the router and link to your routes from anywhere:

```rust
// src/main.rs
mod pages;
mod routes;

fn app() -> impl IntoView {
    routes::router()
}

// Somewhere in a component:
view! { <a href=routes::users::User.materialize(42)>"User 42"</a> }
```

### Why this layout?

- Rust does not (yet) allow `#![routes]` on the `routes.rs` file module itself. So `#[routes]` annotates the private
  `defs` module, and `pub use defs::*;` re-exports everything it generates into `routes`.
- Keep the file scope free of other imports: Names there compete with the re-exported route structs (an imported
  component `Users` would hide `routes::Users`). Import what your views need inside `defs`.
- Refer to components through their module (e.g. in `pages::Users`), as a component imported under a route struct's
  name could conflict with it.

Alternatively, annotate a `pub mod routes { ... }` in an existing file, e.g. next to your `App`. View expressions then
resolve names in that file's scope first: `page!(Users)` refers to the component, not the route struct.

To generate only route structs without views, use `#[routes(without_views)]`.
See [`examples/routes-only`](examples/routes-only) for an example.

## Generated API

The macro generates a struct per route, accessible through the module hierarchy:

```rust
routes::Root
routes::Welcome
routes::Users
routes::users::User
routes::users::user::Details
```

Each struct provides:

- **`path()`** — local path segments (matching `leptos_router::path!()` output) for use in `<Route>` declarations.
  ```rust
  assert_that!(routes::users::User.path()).is_equal_to((ParamSegment("id"),));
  ```
- **`path_pattern() -> &'static str`** — the full path pattern from root to this route.
  ```rust
  assert_that!(routes::users::user::Details.path_pattern()).is_equal_to("/users/:id/details");
  ```
- **`materialize(...) -> String`** — builds a concrete URL, replacing dynamic segments with provided values (any
  `impl Display`). Usable anywhere a `ToHref` is expected. Use `materialize(...)` to build links anywhere in your app!
  ```rust
  assert_that!(routes::users::user::Details.materialize(42u32)).is_equal_to("/users/42/details");
  ```

When views are enabled (the default), two additional functions are generated at the root module level:

- **`route_tree()`** — returns the `<Routes>` view tree. Use when you need a custom `<Router>` component declaration.
- **`router()`** — wraps `route_tree()` in `<Router>` for the common case. Prefer this instead.

A `Route` enum is also generated with a variant per route. It derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`,
`Hash`, implements `Display`, and provides `path_pattern()` and `all() -> &'static [Route]`.

## Path Syntax

Each `#[route("...")]` path must start with `/`, must not end with `/` (except the root `"/"`), and must not contain
`//`.

| Syntax    | Example    | Segment type  | `materialize()` parameter  |
|-----------|------------|---------------|----------------------------|
| `/foo`    | `"/users"` | Static        | *(none)*                   |
| `/:name`  | `"/:id"`   | Param         | `id: impl Display`         |
| `/:name?` | `"/:id?"`  | OptionalParam | `id: Option<impl Display>` |
| `/*name`  | `"/*rest"` | Wildcard      | `rest: impl Display`       |

Parameter names that are Rust keywords get an underscore appended (e.g. `:type` → `type_: impl Display`).

## View Macros

When view generation is enabled (the default):

- **`fallback!(/* view */)`** — required at the routes module root. The 404 / path-not-found view.
- **`layout!(/* view */)`** — optional on parent routes. Wraps children. The view must render `<Outlet/>`. Parents
  without a layout get an implicit `<Outlet/>` passthrough.
- **`index!(/* view */)`** — optional on parent routes. Shown when the parent is matched directly without matching any
  child.
- **`page!(/* view */)`** — required on leaf routes. The view rendered for this route.

## Naming Conventions

Module names map to `PascalCase` struct names: `mod user_details` → `struct UserDetails`.

`Route` enum variants concatenate the PascalCase names of all intermediate modules (between the root and the struct):
`routes::users::user::Details` → `Route::UsersUserDetails`.

## Examples

## basic

A tiny `leptos_axum` driven SSR web server demonstrating the default mode with full router generation:
`fallback!()`, `layout!()`, `index!()`, `page!()`, and the generated `router()` function. It uses the recommended
layout, with the routes in [`src/routes.rs`](examples/basic/src/routes.rs).

```sh
cd examples/basic && cargo run
```

Then open [http://127.0.0.1:3000](http://127.0.0.1:3000) in your browser. Navigate to `/`, `/users`, and `/users/42` to
see the different views.

## routes-only

A CLI program demonstrating `#[routes(without_views)]` for generating only route structs,
`path()`, `path_pattern()`, `materialize()`, and the `Route` enum — without any view or router
generation. Use this mode when you want type-safe route definitions but manage the router yourself.

```sh
cd examples/routes-only && cargo run
```

## Testing

    cargo test --all

## MSRV

- As of `0.4.0` the MSRV is `1.88.0`

## See also

[`leptos-routable`](https://crates.io/crates/leptos-routable) is another community crate for type-safe routing in
Leptos, using a `#[derive(Routable)]` enum approach. Both crates eliminate string-typed route paths but differ in style:
`leptos-routes` uses a module hierarchy that mirrors the URL tree, while `leptos-routable` uses enum variants with route
attributes.

We believe the module-based approach scales better: the nesting is visible at a glance, component declarations live
right next to their route, IDE autocompletion follows the module path, and `materialize()` gives you a unique,
compiler-checked function signature per route so you can never forget a parameter.
