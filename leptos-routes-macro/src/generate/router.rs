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
        let full_path = &route_def.full_module_path_to_struct_def();

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
                <Route path=#full_path.path() #view/>
            }]);
        } else {
            let layout = if let Some(v) = &route_def.layout {
                quote! { view=#v }
            } else {
                quote! { view=move || ::leptos::prelude::view! { <Outlet/> } }
            };

            ts.extend([quote! {
                <ParentRoute path=#full_path.path() #layout>
            }]);
            {
                for child in &route_def.children {
                    process_route_def(child, ts);
                }

                if let Some(v) = &route_def.index {
                    let index = quote! { view=#v };
                    ts.extend([quote! {
                        <Route path=::leptos_router::path!("") #index/>
                    }]);
                }
            }
            ts.extend([quote! {
                </ParentRoute>
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
            use ::leptos_router::components::Routes;
            use ::leptos_router::components::ParentRoute;
            use ::leptos_router::components::Route;
            use ::leptos_router::components::Outlet;
            use ::leptos::prelude::*;
            use super::*;

            view! {
                <Routes fallback=#fallback>
                    #ts
                </Routes>
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
