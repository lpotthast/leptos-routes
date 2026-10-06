//! Generates the `route_tree()` and `router()` functions that return Leptos Router
//! view markup.
//!
//! This module is only active when view generation is enabled (i.e., `without_views`
//! is not set and a `fallback!()` is provided). The generated `route_tree()` function
//! contains `<Routes>`, `<ParentRoute>`, and `<Route>` components matching the route
//! tree. The generated `router()` function wraps `route_tree()` in a `<Router>` for
//! convenience.

use crate::route_def::RouteDef;
use proc_macro_error2::abort;
use quote::quote;
use syn::Expr;

/// Returns the `route_tree()` function token stream, or an empty stream when
/// `without_views` is set.
pub fn maybe_generate_routes_component(
    with_views: bool,
    fallback: Option<Expr>,
    route_defs: &[RouteDef],
) -> proc_macro2::TokenStream {
    if with_views {
        generate_routes_component(route_defs, fallback)
    } else {
        // Don't generate `route_tree()` at all.
        // If the user tries to call it, they'll get a clear compile error:
        //   "cannot find function `route_tree` in module `routes`"
        // To enable it, add `fallback!(YourFallbackComponent)` in the module body.
        quote! {}
    }
}

/// Generates the `route_tree()` function body.
///
/// The view expressions of `layout!()`, `index!()`, `page!()` and `fallback!()` resolve names in
/// the scope enclosing the `#[routes]` module first, just like in the generated struct methods
/// (see `documentation/architecture.md`): `page!(Users)` means the component `Users`, not the
/// route struct `Users`. Everything this function generates itself is therefore referenced by
/// paths no name of that scope can shadow: Route structs as `self::...`, everything else by
/// absolute paths or the reserved `__leptos_router_components` alias.
///
/// Recursively walks the route tree via the inner `process_route_def()`:
/// - **Parent routes** (with children) become `<ParentRoute>`. If `layout!()` is
///   present, its expression is used as the view; otherwise an implicit `<Outlet/>`
///   passthrough is generated. An `index!()` expression becomes a nested
///   `<Route path="">` for the parent's own path.
/// - **Leaf routes** (no children) become `<Route>`, requiring `page!()`.
/// - Using `page!()` on a parent route is an error.
pub fn generate_routes_component(
    route_defs: &[RouteDef],
    fallback: Option<Expr>,
) -> proc_macro2::TokenStream {
    fn process_route_def(route_def: &RouteDef, ts: &mut proc_macro2::TokenStream) {
        let struct_path = route_def.full_module_path_to_struct_def();
        let full_path = quote! { self::#struct_path };

        if route_def.children.is_empty() {
            let view = if let Some(v) = &route_def.page {
                quote! { view=#v }
            } else {
                abort! {
                    route_def.route_ident_span,
                    "Leaf routes (without children) require a page!() declaration.";
                    help = "Add `page!(YourPage)` inside the module body."
                }
            };

            ts.extend([quote! {
                <__leptos_router_components::Route path=#full_path.path() #view/>
            }]);
        } else {
            let layout = if let Some(v) = &route_def.layout {
                quote! { view=#v }
            } else {
                quote! { view=move || ::leptos::view! { <__leptos_router_components::Outlet/> } }
            };

            ts.extend([quote! {
                <__leptos_router_components::ParentRoute path=#full_path.path() #layout>
            }]);
            {
                for child in &route_def.children {
                    process_route_def(child, ts);
                }

                if let Some(v) = &route_def.index {
                    let index = quote! { view=#v };
                    ts.extend([quote! {
                        <__leptos_router_components::Route path=::leptos_router::path!("") #index/>
                    }]);
                }
            }
            ts.extend([quote! {
                </__leptos_router_components::ParentRoute>
            }]);
        }
    }

    // SAFETY: Guaranteed by validation in routes() before calling generate.
    let fallback = fallback.expect("guaranteed by routes() validation");

    let mut ts = quote! {};
    for route_def in route_defs {
        process_route_def(route_def, &mut ts);
    }

    quote! {
        pub fn route_tree() -> impl ::leptos::IntoView {
            // The `fallback!()` expression is only compiled here, and may rely on this.
            #[allow(unused_imports)]
            use ::leptos::prelude::*;
            // Lets the view expressions resolve names in the enclosing scope first.
            #[allow(unused_imports, clippy::wildcard_imports)]
            use super::*;
            // View tags can't start with `::`. A name no scope uses instead.
            use ::leptos_router::components as __leptos_router_components;

            ::leptos::view! {
                <__leptos_router_components::Routes fallback=#fallback>
                    #ts
                </__leptos_router_components::Routes>
            }
        }
    }
}

/// Returns the `router()` convenience function token stream, or an empty
/// stream when `without_views` is set.
///
/// `router()` wraps `route_tree()` in a `<Router>`, covering the common case
/// where no custom `<Router>` props are needed.
pub fn maybe_generate_router_fn(with_views: bool) -> proc_macro2::TokenStream {
    if with_views {
        quote! {
            pub fn router() -> impl ::leptos::IntoView {
                use ::leptos_router::components::Router;
                use ::leptos::prelude::*;

                view! {
                    <Router>{ route_tree() }</Router>
                }
            }
        }
    } else {
        quote! {}
    }
}
