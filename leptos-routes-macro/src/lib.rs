//! Proc-macro implementation crate for [`leptos-routes`](https://crates.io/crates/leptos-routes).
//!
//! This crate is not intended to be used as a direct dependency. Users should depend on
//! `leptos-routes` instead, which re-exports all macros defined here.

mod generate;
mod module_path;
mod path;
mod route_def;
mod route_macro_args;
mod util;

use crate::module_path::ModulePath;
use crate::path::{PathSegment, PathSegments};
use crate::route_def::{
    RouteDef, collect_route_definitions, extract_body_macros, extract_root_fallback,
    validate_body_macro_conflicts,
};
use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use quote::quote;
use syn::{Expr, Item, ItemMod, Visibility, parse_macro_input};

/// Marks a module as a route definition inside a [`routes`]-annotated module.
///
/// # Path syntax
///
/// The first argument is a path string that must start with `/`:
///
/// | Syntax    | Example       | Segment type    | `materialize()` parameter       |
/// |-----------|---------------|-----------------|----------------------------------|
/// | `/foo`    | `"/users"`    | `Static`        | *(none)*                         |
/// | `/:name`  | `"/:id"`      | `Param`         | `id: impl Display`               |
/// | `/:name?` | `"/:id?"`     | `OptionalParam` | `id: Option<impl Display>`       |
/// | `/*name`  | `"/*rest"`    | `Wildcard`      | `rest: impl Display`             |
///
/// Paths must not end with `/` (except the root `"/"`), and must not contain `//`.
///
/// # Component declarations
///
/// Use `layout!()`, `index!()`, and `page!()` body macros inside the module body:
///
/// ```ignore
/// #[route("/users")]
/// mod users {
///     layout!(UsersLayout); // layout (provided view must render <Outlet/> to see child routes!)
///     index!(Home);         // shown when "/users" is matched directly
///
///     #[route("/:id")]
///     mod user {
///         page!(UserPage);  // shown when some user route, e.g. "/users/42", is matched
///     }
/// }
/// ```
///
/// # Visibility
///
/// Route modules must be declared without a visibility modifier (`mod`, not `pub mod`).
/// The `#[routes]` macro automatically rewrites them to `pub` so that the generated types are accessible from
/// outside the `routes` module.
#[proc_macro_attribute]
#[proc_macro_error]
pub fn route(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // When `#[routes]` expands the outer module, it consumes `#[route]` attributes directly
    // and this proc macro is never called at compile time. The IDE may expand `#[route]`
    // independently for source-level analysis; in that case we simply pass through the input.
    // IDE name resolution for component identifiers is provided by layout!() and page!() macros.
    input
}

/// Declares the fallback (404 / not-found) component.
///
/// This macro is designed to be used inside the top-level `#[routes]`-annotated module.
/// It is **required** when view generation is enabled (the default). Use
/// `#[routes(without_views)]` to opt out of view generation and skip the fallback requirement.
///
/// # Usage
///
/// ```ignore
/// #[routes]
/// pub mod routes {
///     fallback!(NotFound);
///
///     #[route("/dashboard")]
///     mod dashboard {
///         page!(Dashboard);
///     }
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn fallback(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn __leptos_routes_fallback() {
            let _ = #expr;
        }
    }
    .into()
}

/// Declares the layout component for a parent route.
///
/// This macro can be used inside `#[route]`-annotated modules or directly in the
/// `#[routes]` module (creating an implicit root route).
/// The layout component must render `<Outlet/>` for child routes to appear.
///
/// # Optional
///
/// Parent routes without `layout!()` become pure path-prefix groups.
/// Internally, an implicit `<Outlet/>` passthrough is generated so that
/// Leptos Router's `<ParentRoute>` still wraps the children correctly.
///
/// # Usage
///
/// ```ignore
/// #[routes]
/// pub mod routes {
///     fallback!(NotFound);
///     layout!(MainLayout);      // main layout (provided view must render <Outlet/> to see child routes!)
///     index!(Home);             // shown when "/" is matched directly (#[routes] does not define a different root path.)
///
///     #[route("/users")]
///     mod users {
///         layout!(UsersLayout); // sub-layout for the users area (provided view must render <Outlet/> to see child routes!)
///         index!(Home);         // shown when "/users" is matched directly
///
///         #[route("/:id")]
///         mod user {
///             page!(UserPage);  // shown when some user route, e.g. "/users/42", is matched
///         }
///     }
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn layout(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn __leptos_routes_layout() {
            let _ = #expr;
        }
    }
    .into()
}

/// Declares the index view for a parent route.
///
/// The index view is shown when the parent route's path is matched directly,
/// without matching any child route. For example, `/users` would show the
/// index view while `/users/42` would show a child route.
///
/// This macro can be used inside `#[route]`-annotated modules or directly in the
/// `#[routes]` module (creating an implicit root route), alongside or instead of `layout!()`.
///
/// # Usage
///
/// ```ignore
/// #[route("/users")]
/// mod users {
///     layout!(UsersLayout);
///     index!(UsersList);   // shown at /users
///
///     #[route("/:id")]
///     mod user {
///         page!(UserPage); // shown at /users/:id
///     }
/// }
/// ```
///
/// # Alternative
///
/// The same effect can be achieved with an explicit child route:
///
/// ```ignore
/// #[route("/users")]
/// mod users {
///     layout!(UsersLayout);
///
///     #[route("/")]
///     mod index {
///         page!(UsersList);
///     }
///
///     #[route("/:id")]
///     mod user {
///         page!(UserPage);
///     }
/// }
/// ```
///
/// `index!()` is preferred over the explicit child route. The parent route struct
/// already provides `materialize()` for URL construction (e.g.,
/// `routes::Users.materialize()` returns `"/users"`), and the view is
/// accessible via the `index()` method. The explicit `#[route("/")]` child would
/// generate a redundant `users::Index` struct, a `Route::UsersIndex` enum
/// variant, and its own `path()` / `materialize()` methods — all producing the
/// same URL that the parent struct already covers.
#[proc_macro]
#[proc_macro_error]
pub fn index(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn __leptos_routes_index() {
            let _ = #expr;
        }
    }
    .into()
}

/// Declares the view component for a leaf route.
///
/// This macro can be used inside `#[route]`-annotated modules or directly in the
/// `#[routes]` module (creating an implicit leaf root route, when no child `#[route]`
/// modules are present).
///
/// # Usage
///
/// ```ignore
/// #[route("/:id")]
/// mod user {
///     page!(UserPage);
/// }
/// ```
#[proc_macro]
#[proc_macro_error]
pub fn page(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    quote! {
        #[doc(hidden)]
        #[allow(dead_code)]
        fn __leptos_routes_page() {
            let _ = #expr;
        }
    }
    .into()
}

/// Parsed arguments for the `#[routes(...)]` attribute macro.
///
/// Controls whether view-generation code (`route_tree()`, component methods)
/// is emitted. When `without_views` is `false` (the default), a `fallback!()` body
/// macro is required in the module.
///
/// An optional `path = "/..."` argument sets the root path prefix. When specified,
/// a synthetic root route is created at that path.
struct RoutesMacroArgs {
    without_views: bool,
    path: Option<String>,
}

impl RoutesMacroArgs {
    /// Returns `true` when view generation is enabled (i.e. `without_views` was not set).
    fn with_views(&self) -> bool {
        !self.without_views
    }
}

impl syn::parse::Parse for RoutesMacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut without_views = false;
        let mut path = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "without_views" {
                without_views = true;
            } else if ident == "path" {
                let _ = input.parse::<syn::Token![=]>()?;
                let lit: syn::LitStr = input.parse()?;
                let value = lit.value();
                if !value.starts_with('/') {
                    return Err(syn::Error::new(lit.span(), "path must start with \"/\"."));
                }
                if value.ends_with('/') && value.len() > 1 {
                    return Err(syn::Error::new(
                        lit.span(),
                        "path must not end with \"/\". Remove the trailing \"/\".",
                    ));
                }
                if value.contains("//") {
                    return Err(syn::Error::new(
                        lit.span(),
                        "path must not contain \"//\". Coalesce consecutive slashes into one.",
                    ));
                }
                path = Some(value);
            } else if ident == "fallback" {
                return Err(syn::Error::new(
                    ident.span(),
                    "\"fallback\" is no longer an attribute argument. Use the fallback!() body macro inside the module instead. Example:\n\n#[routes]\npub mod routes {\n    fallback!(YourFallback);\n    // ...\n}",
                ));
            } else if ident == "with_views" {
                return Err(syn::Error::new(
                    ident.span(),
                    "\"with_views\" is no longer needed. Views are generated by default. Simply use #[routes]. To disable view generation, use #[routes(without_views)].",
                ));
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!(
                        "Unknown attribute: \"{ident}\". Expected \"without_views\" or \"path\"."
                    ),
                ));
            }

            if !input.is_empty() {
                let _ = input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(RoutesMacroArgs {
            without_views,
            path,
        })
    }
}

/// Entry point for route declarations. Applied to a module containing `#[route]`-annotated
/// child modules.
///
/// # Root-level macros
///
/// `layout!()`, `index!()`, and `page!()` can be used directly in the `#[routes]` module.
/// This creates an implicit root route, generating a `Root` struct and wrapping all
/// child routes. This eliminates the need for an explicit `#[route("/")]` wrapper module.
///
/// # Arguments
///
/// - `without_views` — disables view/router generation (no `route_tree()` function).
/// - `path = "/..."` — sets a custom root path prefix. Creates a synthetic root route at
///   the specified path.
///
/// # Example (with root-level layout)
///
/// ```
/// use assertr::prelude::*;
/// use leptos::prelude::*;
/// use leptos_router::components::{Outlet, Router};
/// use leptos_router::location::RequestUrl;
/// use leptos_routes::routes;
///
/// #[routes]
/// pub mod routes {
///     fallback!(|| view! { <Err404/> });
///     layout!(MainLayout);
///     index!(Dashboard);
///
///     #[route("/users")]
///     mod users {
///         layout!(UsersLayout);
///         index!(NoUser);
///
///         #[route("/:id")]
///         mod user {
///             page!(UserDetails);
///         }
///     }
/// }
///
/// #[component]
/// fn Err404() -> impl IntoView { view! { "Err404" } }
/// #[component]
/// fn MainLayout() -> impl IntoView { view! { <div id="main-layout"> <Outlet/> </div> } }
/// #[component]
/// fn UsersLayout() -> impl IntoView { view! { <div id="users-layout"> <Outlet/> </div> } }
/// #[component]
/// fn Dashboard() -> impl IntoView { view! { "Dashboard" } }
/// #[component]
/// fn NoUser() -> impl IntoView { view! { "NoUser" } }
/// #[component]
/// fn UserDetails() -> impl IntoView { view! { "UserDetails" } }
///
/// fn main() {
///     fn app() -> impl IntoView {
///         view! {
///             <Router>
///                 { routes::route_tree() }
///             </Router>
///         }
///     }
///
///     let _owner = Owner::new_root(None);
///
///     provide_context::<RequestUrl>(RequestUrl::new(
///         routes::users::User.materialize("42").as_str(),
///     ));
///     assert_that!(app().to_html()).is_equal_to(r#"<div id="main-layout"><div id="users-layout">UserDetails</div></div>"#);
/// }
/// ```
///
/// # Example (without views)
///
/// ```
/// use leptos_routes::routes;
///
/// #[routes(without_views)]
/// pub mod routes {
///     #[route("/users")]
///     mod users {
///         #[route("/:id")]
///         mod user {
///             #[route("/details")]
///             mod details {}
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn routes(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as RoutesMacroArgs);

    let mut root_mod: ItemMod = parse_macro_input!(input as ItemMod);

    // Make sure we have module contents to work with.
    let Some((_brace, ref mut content)) = root_mod.content else {
        abort!(root_mod.ident, "routes macro requires a module with a body");
    };

    // Extract fallback!() from the root module body.
    let fallback = extract_root_fallback(content);

    // Validate fallback/without_views consistency.
    if args.without_views && fallback.is_some() {
        abort!(
            root_mod.ident,
            "fallback!() cannot be used with \"without_views\", as no views are generated."
        );
    }
    if args.with_views() && fallback.is_none() {
        abort!(
            root_mod.ident,
            "A fallback!() is required when views are generated. Add `fallback!(YourFallback)` inside the module body, or use #[routes(without_views)] to disable view generation."
        );
    }

    // Extract layout!/index!/page! from the root module body (new: supports root-level views).
    let root_body = extract_body_macros(content, true);

    // Validate mutual exclusivity at root level (same rules as child routes).
    validate_body_macro_conflicts(&root_body);

    // Determine if a synthetic root is needed (must check before collection to compute prefix).
    let has_root_body_macros =
        root_body.layout.is_some() || root_body.index.is_some() || root_body.page.is_some();
    let needs_synthetic_root = has_root_body_macros || args.path.is_some();

    // Compute the initial accumulated path prefix and params for child collection.
    let (initial_path_prefix, initial_params) = if needs_synthetic_root {
        let root_path = args.path.as_deref().unwrap_or("/");
        let root_segments = PathSegments::parse(root_path);

        // Validate wildcard placement in root path prefix.
        if let Some(pos) = root_segments
            .segments
            .iter()
            .position(|seg| matches!(seg, PathSegment::Wildcard(_)))
            && pos != root_segments.segments.len() - 1
        {
            abort!(
                root_mod.ident,
                "Wildcard segments (e.g., `*name`) must be the last segment in a path. \
                 Move the wildcard to the end of \"{}\".",
                root_path,
            );
        }

        let (path_pattern, params) =
            route_def::compute_accumulated_context(root_path, &root_segments, "", &[]);

        // Convert path_pattern to child prefix: "/" becomes "" to avoid double slashes.
        let prefix = if path_pattern == "/" {
            String::new()
        } else {
            path_pattern
        };
        (prefix, params)
    } else {
        (String::new(), vec![])
    };

    // Collect child route definitions from #[route]-annotated modules.
    let mut route_defs: Vec<RouteDef> = Vec::new();
    let root_module_path = ModulePath::root(root_mod.ident.clone());
    for item in content.iter_mut() {
        if let Item::Mod(child_module) = item {
            collect_route_definitions(
                child_module,
                None,
                &mut route_defs,
                &root_module_path,
                &initial_path_prefix,
                &initial_params,
            );
        }
    }

    if needs_synthetic_root {
        // Validate: page!() at root is only valid without children.
        if root_body.page.is_some() && !route_defs.is_empty() {
            abort!(
                root_body
                    .page_span
                    .expect("page_span must be Some when page is Some"),
                "page!() at root level requires no child #[route] modules. Use layout!() and/or index!() for parent routes, or remove child routes to make this a leaf."
            );
        }

        // Validate: no #[route("/")] child when synthetic root is present.
        if let Some(conflict) = route_defs.iter().find(|r| r.path == "/") {
            abort!(
                conflict.route_ident_span,
                "Cannot use #[route(\"/\")] when the #[routes] module has root-level macros (layout!/index!/page!) or a path= argument. \
                 Move your layout!/index! into the #[route(\"/\")] module, or remove #[route(\"/\")] and use root-level macros instead."
            );
        }

        let root_path = args.path.as_deref().unwrap_or("/");
        let root_name = syn::Ident::new("Root", root_mod.ident.span());

        // Re-parent all children under the synthetic root.
        for child in &mut route_defs {
            child.parent_struct = Some((root_path.to_owned(), root_name.clone()));
        }

        // The module path needs 2 elements so that `intermediate_segments()` returns `[]`,
        // placing the Root struct directly in the routes module.
        let mut root_module_path = ModulePath::root(root_mod.ident.clone());
        root_module_path.push(root_name.clone());

        // Placeholder name; `module_name` is #[expect(unused)] and never surfaces to users.
        let synthetic_root = RouteDef {
            module_name: syn::Ident::new("__root__", root_mod.ident.span()),
            module_span: root_mod.ident.span(),
            route_ident_span: root_mod.ident.span(),
            path: root_path.to_owned(),
            path_segments: PathSegments::parse(root_path),
            layout: root_body.layout,
            layout_span: root_body.layout_span,
            index: root_body.index,
            index_span: root_body.index_span,
            page: root_body.page,
            page_span: root_body.page_span,
            name: root_name,
            parent_struct: None,
            vis: Visibility::Public(syn::token::Pub::default()),
            found_in_module_path: root_module_path,
            children: route_defs,
            path_pattern: root_path.to_owned(),
            all_params: initial_params,
        };

        route_defs = vec![synthetic_root];
    }

    generate::impls(&mut root_mod, &args, fallback, &route_defs);

    Into::into(quote! { #root_mod })
}
