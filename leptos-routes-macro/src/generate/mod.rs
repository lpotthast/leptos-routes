//! Orchestrates all code generation for a `#[routes]`-annotated module.
//!
//! After route definitions have been collected, [`impls`] drives the generation
//! pipeline: it injects helper imports, generates per-route structs, the `Route`
//! enum, and optionally the `route_tree()` and `router()` functions.

use crate::RoutesMacroArgs;
use crate::generate::all_routes_enum::generate_route_enum;
use crate::generate::route_struct::generate_route_struct;
use crate::generate::router::{maybe_generate_router_fn, maybe_generate_routes_component};
use crate::route_def::{RouteDef, flatten};
use proc_macro_error2::abort_call_site;
use syn::{Attribute, Expr, Item, ItemMod, parse_quote};

pub mod all_routes_enum;
pub mod route_struct;
pub mod router;

/// Top-level generation entry point. Mutates `root_mod` in place to inject all
/// generated code.
///
/// Pipeline:
/// 1. Adds `#[allow(clippy::module_inception)]` to the root module.
/// 2. Injects `use super::*;` and macro imports (`route`, `layout`, `index`,
///    `page`, `fallback`) so that preserved attributes and body macros resolve
///    throughout the module tree.
/// 3. Generates a struct + impl block per route and inserts them into the
///    corresponding source modules.
/// 4. Generates the `Route` enum and `Display` impl at the root module level.
/// 5. Generates `route_tree()` (when views are enabled) at the root.
pub fn impls(
    root_mod: &mut ItemMod,
    args: &RoutesMacroArgs,
    fallback: Option<Expr>,
    route_defs: &[RouteDef],
) {
    // A common pattern could be to add a root-level `routes.rs` file containing the `#[routes]`
    // annotated inline-defined `routes` module.
    // Clippy does not like this nesting of similarly named modules. As it generally should!
    // For this special case, not letting the lint pop up for many users, we explicitly allow it.
    let allow_module_inception: Attribute = parse_quote!(#[allow(clippy::module_inception)]);
    root_mod.attrs.push(allow_module_inception);

    // Inject scoping helpers at the top of the root module (before any child modules).
    // `use super::*;` brings in items from the enclosing scope (e.g. component functions).
    // `route`, `layout`, `index`, `page`, and `fallback` imports make preserved `#[route]` attributes
    // and `layout!()`/`index!()`/`page!()`/`fallback!()` invocations resolve. Child route modules inject
    // their own `use super::*;` which chains all of these imports downward through the module tree.
    if let Some((_, items)) = &mut root_mod.content {
        let helpers: Vec<Item> = vec![
            // Bring in items from the enclosing scope (e.g. component functions) so that
            // the `use super::*;` chain in child modules can resolve them.
            parse_quote! {
                #[allow(unused_imports, clippy::wildcard_imports)]
                use super::*;
            },
            // Import `route`, `layout`, `index`, `page`, and `fallback` so that preserved `#[route]`
            // attributes and body macro invocations resolve. Child route modules inject
            // `use super::*;` which chains these imports downward through the module tree.
            parse_quote! {
                #[allow(unused_imports)]
                use ::leptos_routes::route;
            },
            parse_quote! {
                #[allow(unused_imports)]
                use ::leptos_routes::layout;
            },
            parse_quote! {
                #[allow(unused_imports)]
                use ::leptos_routes::index;
            },
            parse_quote! {
                #[allow(unused_imports)]
                use ::leptos_routes::page;
            },
            parse_quote! {
                #[allow(unused_imports)]
                use ::leptos_routes::fallback;
            },
        ];
        // Insert at position 0 so they appear before child modules.
        for (i, item) in helpers.into_iter().enumerate() {
            items.insert(i, item);
        }
    }

    // Generate the individual route structs.
    for route_def in flatten(route_defs) {
        let (struct_def, struct_impl) = generate_route_struct(route_def, args.with_views());

        let src_mod = find_src_module(
            root_mod,
            route_def.found_in_module_path.intermediate_segments(),
        )
        .expect("source module must exist in tree for route found at this module path");

        insert_into_module(src_mod, struct_def);
        insert_into_module(src_mod, struct_impl);
    }

    // Generate a "Route" enum listing all possible routes.
    for item in generate_route_enum(route_defs) {
        insert_into_module(root_mod, item);
    }

    // Generate `route_tree()` — the `<Routes>` view tree.
    insert_into_module(
        root_mod,
        maybe_generate_routes_component(args.with_views(), fallback, route_defs),
    );

    // Generate `router()` — convenience wrapper that puts `route_tree()` inside `<Router>`.
    insert_into_module(root_mod, maybe_generate_router_fn(args.with_views()));
}

/// Navigates the `ItemMod` tree to find the module at the given path.
///
/// Used to locate the source module where a generated route struct should be
/// inserted. Returns `None` if any segment of `path` does not match a child module.
pub fn find_src_module<'a>(
    module: &'a mut ItemMod,
    path: &[syn::Ident],
) -> Option<&'a mut ItemMod> {
    if path.is_empty() {
        return Some(module);
    }

    if let Some((_, items)) = &mut module.content {
        for item in items.iter_mut() {
            if let Item::Mod(child_module) = item
                && child_module.ident == path[0]
            {
                return find_src_module(child_module, &path[1..]);
            }
        }
    }

    None
}

/// Parses a [`proc_macro2::TokenStream`] into a [`syn::Item`] and appends it
/// to the module's content. No-ops if the token stream is empty.
pub fn insert_into_module(module: &mut ItemMod, ts: proc_macro2::TokenStream) {
    if ts.is_empty() {
        return;
    }
    match syn::parse2::<Item>(ts) {
        Ok(item) => {
            if let Some((_, items)) = &mut module.content {
                items.push(item);
            } else {
                abort_call_site!("Expected module to have content");
            }
        }
        Err(e) => abort_call_site!(e),
    }
}
