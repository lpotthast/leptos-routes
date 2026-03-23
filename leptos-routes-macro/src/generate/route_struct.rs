//! Generates a route struct and its `impl` block for each `#[route]`-annotated module.
//!
//! Each route gets a unit struct (e.g., `pub struct Users;`) with methods:
//! - `path()` — returns the local path segment tuple for use in `<Route>`/`<ParentRoute>`.
//! - `path_pattern()` — returns the full concatenated path pattern from root to this route.
//! - `materialize(...)` — builds a concrete URL string, accepting dynamic parameters.
//! - When views are enabled: `layout()`, `page()`, and/or `index()` component accessors.

use crate::path::{PathSegment, PathSegments};
use crate::route_def::RouteDef;
use crate::util::sanitize_identifier;
use quote::{format_ident, quote};

/// Builds a `format!()` string and its arguments for URL materialization.
///
/// Static segments use `AsPath` to extract the path string from their `StaticSegment`
/// wrapper. Dynamic segments (`:param`, `*wildcard`) become direct `format!()` arguments.
/// Optional parameters (`:param?`) produce conditional formatting: present values get
/// `/{val}`, absent values produce an empty string.
///
/// The `has_parent_with_empty_path` flag suppresses the leading `/` separator for the
/// first segment when the parent's path is `"/"` or empty, avoiding double slashes.
fn create_format(
    segments: &PathSegments,
    format_str: &mut String,
    format_args: &mut Vec<proc_macro2::TokenStream>,
    has_parent_with_empty_path: bool,
) {
    if segments.segments.is_empty() {
        format_str.push('/');
        return;
    }
    for (i, seg) in segments.segments.iter().enumerate() {
        let segment_var = format_ident!("segment_{}", i);
        match seg {
            PathSegment::Static(_) => {
                // When the parent's materialized path already ends with "/" (or is empty),
                // omit the leading "/" on the first segment to prevent "//".
                if i == 0 && has_parent_with_empty_path {
                    format_str.push_str("{}");
                } else {
                    format_str.push_str("/{}");
                }
                format_args.push(quote! { ::leptos_router::AsPath::as_path(&(#segment_var).0) });
            }
            PathSegment::Param(name) | PathSegment::Wildcard(name) => {
                if i == 0 && has_parent_with_empty_path {
                    format_str.push_str("{}");
                } else {
                    format_str.push_str("/{}");
                }
                let name = format_ident!("{}", sanitize_identifier(name));
                format_args.push(quote! { #name });
            }
            PathSegment::OptionalParam(name) => {
                // Optional params produce "/{val}" when Some, or "" when None.
                // If placed mid-path, None causes the segment to drop from the URL
                // (e.g., "/:year?/posts" with year=None → "/posts").
                format_str.push_str("{}");
                let name = format_ident!("{}", sanitize_identifier(name));
                format_args.push(quote! {
                    if let Some(val) = #name {
                        format!("/{}", val)
                    } else {
                        String::new()
                    }
                });
            }
        }
    }
}

/// Generates `layout()`, `page()`, and `index()` methods on a route struct.
///
/// Each method uses a `use super::…::*;` import chain whose depth matches
/// the module nesting, so that user-defined component functions from the
/// scope enclosing the `#[routes]` module are reachable.
fn generate_view_methods(route_def: &RouteDef) -> proc_macro2::TokenStream {
    // Compute the super chain to reach the routes module's parent scope, where
    // user-defined components are accessible. The depth is the number of module
    // segments from the routes root to the struct's module, plus 1 for the root itself.
    let depth = route_def.found_in_module_path.intermediate_segments().len() + 1;
    let supers = std::iter::repeat_n(quote! { super }, depth);
    let super_import = quote! {
        #[allow(clippy::wildcard_imports)]
        use #(#supers)::*::*;
    };

    let mut methods = quote! {};

    if let Some(expr) = &route_def.layout {
        methods.extend(quote! {
            pub fn layout(&self) -> impl ::leptos_router::ChooseView {
                #super_import
                #expr
            }
        });
    }

    if let Some(expr) = &route_def.page {
        methods.extend(quote! {
            pub fn page(&self) -> impl ::leptos_router::ChooseView {
                #super_import
                #expr
            }
        });
    }

    if let Some(expr) = &route_def.index {
        methods.extend(quote! {
            pub fn index(&self) -> impl ::leptos_router::ChooseView {
                #super_import
                #expr
            }
        });
    }

    methods
}

/// Collects everything needed to generate the `materialize()` method signature
/// and format string. Encapsulates the shared logic between root and child routes.
struct MaterializeContext {
    generics: proc_macro2::TokenStream,
    params: Vec<proc_macro2::TokenStream>,
    format_str: String,
    format_args: Vec<proc_macro2::TokenStream>,
    segment_vars: Vec<syn::Ident>,
}

impl MaterializeContext {
    /// Build context for a child route (with parent).
    ///
    /// Uses the pre-computed `all_params` from the full hierarchy. Prepends a `{}`
    /// placeholder in the format string for the parent's materialized path.
    fn for_child_route(route_def: &RouteDef, path_segments: &PathSegments) -> Self {
        let (parent_path, _) = route_def
            .parent_struct
            .as_ref()
            .expect("for_child_route requires parent_struct");

        let all_params = &route_def.all_params;

        let generic_params: Vec<_> = all_params
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let ty = format_ident!("T{}", i);
                quote! { #ty: ::std::fmt::Display }
            })
            .collect();

        let params: Vec<_> = all_params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let name = format_ident!("{}", sanitize_identifier(&p.name));
                let ty = format_ident!("T{}", i);
                if p.is_optional {
                    quote! { #name: Option<#ty> }
                } else {
                    quote! { #name: #ty }
                }
            })
            .collect();

        let generics = if generic_params.is_empty() {
            quote! {}
        } else {
            quote! { <#(#generic_params),*> }
        };

        let mut format_str = String::from("{}"); // parent path placeholder
        let mut format_args = Vec::new();
        create_format(
            path_segments,
            &mut format_str,
            &mut format_args,
            parent_path.is_empty() || parent_path == "/",
        );

        let segment_vars = (0..path_segments.segments.len())
            .map(|i| format_ident!("segment_{}", i))
            .collect();

        Self {
            generics,
            params,
            format_str,
            format_args,
            segment_vars,
        }
    }

    /// Build context for a root route (no parent).
    ///
    /// Computes params from local path segments only.
    fn for_root_route(path_segments: &PathSegments) -> Self {
        let mut generic_idx = 0usize;
        let params_with_generics: Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)> =
            path_segments
                .segments
                .iter()
                .filter_map(|seg| match seg {
                    PathSegment::Param(name) | PathSegment::Wildcard(name) => {
                        let name = format_ident!("{}", sanitize_identifier(name));
                        let ty = format_ident!("T{}", generic_idx);
                        generic_idx += 1;
                        Some((quote! { #ty: ::std::fmt::Display }, quote! { #name: #ty }))
                    }
                    PathSegment::OptionalParam(name) => {
                        let name = format_ident!("{}", sanitize_identifier(name));
                        let ty = format_ident!("T{}", generic_idx);
                        generic_idx += 1;
                        Some((
                            quote! { #ty: ::std::fmt::Display },
                            quote! { #name: Option<#ty> },
                        ))
                    }
                    PathSegment::Static(_) => None,
                })
                .collect();

        let generic_params: Vec<_> = params_with_generics
            .iter()
            .map(|(g, _)| g.clone())
            .collect();
        let params: Vec<_> = params_with_generics
            .iter()
            .map(|(_, p)| p.clone())
            .collect();

        let generics = if generic_params.is_empty() {
            quote! {}
        } else {
            quote! { <#(#generic_params),*> }
        };

        let mut format_str = String::new();
        let mut format_args = Vec::new();
        create_format(path_segments, &mut format_str, &mut format_args, false);

        let segment_vars = (0..path_segments.segments.len())
            .map(|i| format_ident!("segment_{}", i))
            .collect();

        Self {
            generics,
            params,
            format_str,
            format_args,
            segment_vars,
        }
    }
}

/// Generates the struct definition and its `impl` block for a single route.
///
/// Returns `(struct_def, struct_impl)` as separate token streams so the caller
/// can insert them into the correct source module.
///
/// Uses [`MaterializeContext`] to encapsulate the shared generic params, format
/// string, and segment variable logic. The only thing that differs between root
/// and child routes is the `materialize()` body.
pub fn generate_route_struct(
    route_def: &RouteDef,
    with_views: bool,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let struct_name = &route_def.name;
    let path = &route_def.path;
    let vis = &route_def.vis;
    let path_segments = &route_def.path_segments;
    let path_type = path_segments.generate_path_type();
    let path_pattern_str = &route_def.path_pattern;

    let view_methods = if with_views {
        generate_view_methods(route_def)
    } else {
        quote! {}
    };

    let struct_def = quote! {
        #[doc = #path]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis struct #struct_name;
    };

    let ctx = if route_def.parent_struct.is_some() {
        MaterializeContext::for_child_route(route_def, path_segments)
    } else {
        MaterializeContext::for_root_route(path_segments)
    };

    let MaterializeContext {
        generics,
        params,
        format_str,
        format_args,
        segment_vars,
    } = &ctx;

    let materialize_body = if let Some((_, parent)) = &route_def.parent_struct {
        let parent_params = route_def
            .all_params
            .iter()
            .take_while(|p| {
                !path_segments.segments.iter().any(|seg| {
                    matches!(seg,
                        PathSegment::Param(name) |
                        PathSegment::OptionalParam(name) |
                        PathSegment::Wildcard(name) if name == &p.name
                    )
                })
            })
            .map(|p| format_ident!("{}", sanitize_identifier(&p.name)));

        // When the struct is placed at the root module level (intermediate_segments() is empty)
        // and it has a parent, both are in the same module — reference the parent directly.
        // Otherwise, the parent is one module level up, so use `super::`.
        let parent_ref = if route_def
            .found_in_module_path
            .intermediate_segments()
            .is_empty()
        {
            quote! { #parent }
        } else {
            quote! { super::#parent }
        };

        quote! {
            let parent = #parent_ref;
            let parent_path = parent.materialize(#(#parent_params),*);
            let (#(#segment_vars,)*) = self.path();
            format!(#format_str, parent_path, #(#format_args),*)
        }
    } else {
        quote! {
            let (#(#segment_vars,)*) = self.path();
            format!(#format_str, #(#format_args),*)
        }
    };

    let struct_impl = quote! {
        impl #struct_name {
            pub fn path(&self) -> #path_type {
                ::leptos_router::path!(#path)
            }

            pub fn path_pattern(&self) -> &'static str {
                #path_pattern_str
            }

            pub fn materialize #generics(&self, #(#params),*) -> String {
                #materialize_body
            }

            #view_methods
        }
    };

    (struct_def, struct_impl)
}
