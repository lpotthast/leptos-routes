//! Central intermediate representation for parsed routes and the recursive
//! collection logic that builds the route tree from the module hierarchy.
//!
//! [`RouteDef`] is the primary data structure: one instance per `#[route]`-annotated
//! module. [`collect_route_definitions`] recursively walks the module tree to
//! populate a `Vec<RouteDef>`. The helper function [`flatten`] operates on the
//! resulting tree.

use crate::ModulePath;
use crate::path::{ParamInfo, PathSegment, PathSegments};
use crate::route_macro_args::RouteMacroArgs;
use crate::util::to_pascal_case;
use proc_macro_error2::abort;
use proc_macro2::Span;
use std::iter::from_fn;
use syn::spanned::Spanned;
use syn::{Expr, Item, ItemMod, PathArguments, Visibility};
/// Intermediate representation of a single parsed route.
///
/// One `RouteDef` is created for each `#[route("/...")]`-annotated module.
/// The tree structure mirrors the module nesting via the [`children`](Self::children) field.
#[derive(Debug)]
pub struct RouteDef {
    /// The original `snake_case` identifier of the module (e.g., `users`).
    #[expect(unused)]
    pub module_name: syn::Ident,
    /// Span of the module declaration, retained for error reporting.
    #[expect(unused)]
    pub module_span: Span,
    /// Span of the `#[route(...)]` attribute, used in diagnostics.
    pub route_ident_span: Span,

    /// The raw path string from `#[route("/...")]`, e.g., `"/"` or `"/users"`.
    pub path: String,
    /// Parsed representation of [`path`](Self::path) as typed segments.
    pub path_segments: PathSegments,

    /// Component expression from `layout!()`, if present.
    pub layout: Option<Expr>,
    /// Span of the `layout!()` invocation, retained for diagnostics.
    #[expect(unused)]
    pub layout_span: Option<Span>,

    /// Component expression from `index!()`, if present.
    pub index: Option<Expr>,
    /// Span of the `index!()` invocation, retained for diagnostics.
    #[expect(unused)]
    pub index_span: Option<Span>,

    /// Component expression from `page!()`, if present.
    pub page: Option<Expr>,
    /// Span of the `page!()` invocation, used in diagnostics.
    pub page_span: Option<Span>,

    /// Pascal-cased name of the module that had this route annotation.
    /// For a `mod users {}`, this will be `Users`.
    pub name: syn::Ident,

    /// Parent route info: `(parent_path_string, parent_PascalCase_ident)`.
    /// `None` for root-level routes.
    pub parent_struct: Option<(String, syn::Ident)>,
    /// Visibility of the generated struct (always rewritten to `pub` by the macro).
    pub vis: Visibility,
    /// Full module nesting path from the root `#[routes]` module, used to determine
    /// where the generated struct should be inserted.
    pub found_in_module_path: ModulePath,
    /// Child route definitions, forming the tree structure.
    pub children: Vec<RouteDef>,

    /// Pre-computed path pattern from root to this route
    /// (e.g., `"/users/:id/details"`). Computed during collection.
    pub path_pattern: String,
    /// Pre-computed list of all dynamic parameters from root to this route,
    /// ordered root-first. Computed during collection.
    pub all_params: Vec<ParamInfo>,
}

impl RouteDef {
    /// Builds a [`syn::Path`] for referencing this route's generated struct from
    /// the root module. For example, a struct `Details` nested under `users::user`
    /// produces the path `users::user::Details`.
    pub fn full_module_path_to_struct_def(&self) -> syn::Path {
        let struct_name = &self.name;
        let intermediate = self.found_in_module_path.intermediate_segments();

        let mut segments: syn::punctuated::Punctuated<syn::PathSegment, syn::Token![::]> =
            intermediate
                .iter()
                .map(|ident| syn::PathSegment {
                    ident: ident.clone(),
                    arguments: PathArguments::None,
                })
                .collect();

        segments.push(syn::PathSegment {
            ident: struct_name.clone(),
            arguments: PathArguments::None,
        });

        syn::Path {
            leading_colon: None,
            segments,
        }
    }
}

/// Returns true if `path` resolves to a macro named `name`
/// (handles both `name` and `some_crate::name`).
///
/// Intentionally matches only the last segment so that qualified paths
/// like `leptos_routes::layout` resolve correctly. A false positive from
/// an unrelated crate is unlikely in practice and would only cause the
/// tokens to be parsed as an `Expr`.
fn is_macro_named(path: &syn::Path, name: &str) -> bool {
    path.segments.last().is_some_and(|seg| seg.ident == name)
}

/// Identifies the parent route during recursive collection.
/// Bundles the parent's path string and generated struct name so they
/// cannot get out of sync.
pub(crate) struct ParentContext<'a> {
    /// The parent route's raw path string (e.g., `"/users"`).
    pub path: &'a str,
    /// The parent route's `PascalCase` struct identifier (e.g., `Users`).
    pub struct_name: &'a syn::Ident,
}

/// Result of scanning a route module's items for `layout!()`, `index!()`, and
/// `page!()` macro invocations.
#[derive(Default)]
pub(crate) struct BodyMacros {
    pub layout: Option<Expr>,
    pub layout_span: Option<Span>,
    pub index: Option<Expr>,
    pub index_span: Option<Span>,
    pub page: Option<Expr>,
    pub page_span: Option<Span>,
}

/// Scan a module's items for `layout!(...)`, `index!(...)`, and `page!(...)` macro invocations
/// and extract their expressions. The invocations are preserved in the item list
/// so that the compiler and rust-analyzer can still expand them for IDE support.
///
/// When `at_root` is `true`, `fallback!()` invocations are silently skipped (they are
/// handled separately by [`extract_root_fallback`]). When `false`, encountering
/// `fallback!()` is an error.
pub(crate) fn extract_body_macros(items: &[Item], at_root: bool) -> BodyMacros {
    let mut layout: Option<Expr> = None;
    let mut layout_span: Option<Span> = None;
    let mut index: Option<Expr> = None;
    let mut index_span: Option<Span> = None;
    let mut page: Option<Expr> = None;
    let mut page_span: Option<Span> = None;

    for item in items {
        if let Item::Macro(item_macro) = item {
            if is_macro_named(&item_macro.mac.path, "layout") {
                if layout.is_some() {
                    abort!(
                        item_macro.mac.path.span(),
                        "Duplicate layout!() invocation. Only one layout!() per module is allowed."
                    );
                }
                match syn::parse2::<Expr>(item_macro.mac.tokens.clone()) {
                    Ok(expr) => {
                        layout_span = Some(item_macro.mac.path.span());
                        layout = Some(expr);
                    }
                    Err(e) => abort!(
                        item_macro.mac.path.span(),
                        "Failed to parse layout!() argument: {}",
                        e
                    ),
                }
            } else if is_macro_named(&item_macro.mac.path, "index") {
                if index.is_some() {
                    abort!(
                        item_macro.mac.path.span(),
                        "Duplicate index!() invocation. Only one index!() per module is allowed."
                    );
                }
                match syn::parse2::<Expr>(item_macro.mac.tokens.clone()) {
                    Ok(expr) => {
                        index_span = Some(item_macro.mac.path.span());
                        index = Some(expr);
                    }
                    Err(e) => abort!(
                        item_macro.mac.path.span(),
                        "Failed to parse index!() argument: {}",
                        e
                    ),
                }
            } else if is_macro_named(&item_macro.mac.path, "fallback") {
                if !at_root {
                    abort!(
                        item_macro.mac.path.span(),
                        "fallback!() can only be used in the top-level #[routes] module, not inside #[route] modules."
                    );
                }
                // At root level, fallback is handled by extract_root_fallback(); skip it here.
            } else if is_macro_named(&item_macro.mac.path, "page") {
                if page.is_some() {
                    abort!(
                        item_macro.mac.path.span(),
                        "Duplicate page!() invocation. Only one page!() per module is allowed."
                    );
                }
                match syn::parse2::<Expr>(item_macro.mac.tokens.clone()) {
                    Ok(expr) => {
                        page_span = Some(item_macro.mac.path.span());
                        page = Some(expr);
                    }
                    Err(e) => abort!(
                        item_macro.mac.path.span(),
                        "Failed to parse page!() argument: {}",
                        e
                    ),
                }
            }
        }
    }

    BodyMacros {
        layout,
        layout_span,
        index,
        index_span,
        page,
        page_span,
    }
}

/// Scan the root module's items for a `fallback!(...)` macro invocation
/// and extract the fallback expression.
pub(crate) fn extract_root_fallback(items: &[Item]) -> Option<Expr> {
    let mut fallback: Option<Expr> = None;

    for item in items {
        if let Item::Macro(item_macro) = item
            && is_macro_named(&item_macro.mac.path, "fallback")
        {
            if fallback.is_some() {
                abort!(
                    item_macro.mac.path.span(),
                    "Duplicate fallback!() invocation. Only one fallback!() per #[routes] module is allowed."
                );
            }
            match syn::parse2::<Expr>(item_macro.mac.tokens.clone()) {
                Ok(expr) => {
                    fallback = Some(expr);
                }
                Err(e) => abort!(
                    item_macro.mac.path.span(),
                    "Failed to parse fallback!() argument: {}",
                    e
                ),
            }
        }
    }

    fallback
}

/// Validates that wildcard segments are at the end and that no dynamic parameter
/// name collides with an ancestor route's parameters.
fn validate_path_segments(
    path_segments: &PathSegments,
    route_path: &str,
    route_ident_span: Span,
    module_name: &syn::Ident,
    accumulated_params: &[ParamInfo],
) {
    // Wildcards must be the last segment.
    if let Some(pos) = path_segments
        .segments
        .iter()
        .position(|seg| matches!(seg, PathSegment::Wildcard(_)))
        && pos != path_segments.segments.len() - 1
    {
        abort!(
            route_ident_span,
            "Wildcard segments (e.g., `*name`) must be the last segment in a path. \
             Move the wildcard to the end of \"{}\".",
            route_path,
        );
    }

    // Check for parameter name collisions with ancestors.
    for seg in &path_segments.segments {
        let (PathSegment::Param(local_name)
        | PathSegment::OptionalParam(local_name)
        | PathSegment::Wildcard(local_name)) = seg
        else {
            continue;
        };
        if let Some(ancestor) = accumulated_params.iter().find(|p| p.name == *local_name) {
            let suggestion = format!("{module_name}_{local_name}");
            abort!(
                route_ident_span,
                "Parameter \"{}\" in route \"{}\" shadows the same parameter from ancestor route \"{}\". \
                 Each dynamic segment name must be unique across the route hierarchy. \
                 Maybe use \"{}\" instead.",
                local_name,
                route_path,
                ancestor.origin_path,
                suggestion,
            );
        }
    }
}

/// Computes the full path pattern from root to this route and collects all
/// dynamic parameters through the hierarchy.
pub(crate) fn compute_accumulated_context(
    route_path: &str,
    path_segments: &PathSegments,
    accumulated_path_prefix: &str,
    accumulated_params: &[ParamInfo],
) -> (String, Vec<ParamInfo>) {
    let path_pattern = if route_path == "/" {
        if accumulated_path_prefix.is_empty() {
            "/".to_string()
        } else {
            accumulated_path_prefix.to_string()
        }
    } else if accumulated_path_prefix.is_empty() || accumulated_path_prefix == "/" {
        route_path.to_string()
    } else {
        format!("{accumulated_path_prefix}{route_path}")
    };

    let mut all_params = accumulated_params.to_vec();
    for seg in &path_segments.segments {
        match seg {
            PathSegment::Param(name) | PathSegment::Wildcard(name) => {
                all_params.push(ParamInfo {
                    name: name.clone(),
                    is_optional: false,
                    origin_path: route_path.to_string(),
                });
            }
            PathSegment::OptionalParam(name) => all_params.push(ParamInfo {
                name: name.clone(),
                is_optional: true,
                origin_path: route_path.to_string(),
            }),
            PathSegment::Static(_) => {}
        }
    }

    (path_pattern, all_params)
}

/// Validates that body macro combinations are mutually exclusive.
pub(crate) fn validate_body_macro_conflicts(body: &BodyMacros) {
    if body.layout.is_some() && body.page.is_some() {
        abort!(
            body.page_span
                .expect("page_span must be Some when page is Some"),
            "Cannot use both layout!() and page!() in the same module. Use layout!() for parent routes (with children) or page!() for leaf routes."
        );
    }

    if body.index.is_some() && body.page.is_some() {
        abort!(
            body.page_span
                .expect("page_span must be Some when page is Some"),
            "Cannot use both index!() and page!() in the same module. \
             index!() is for parent routes (alongside layout!()), \
             page!() is for leaf routes (without children)."
        );
    }
}

/// Injects `use super::*;` into a module to chain imports downward.
fn inject_use_super(module: &mut ItemMod) {
    if let Some((_, items)) = &mut module.content {
        let use_super: Item = syn::parse_quote! {
            #[allow(unused_imports, clippy::wildcard_imports)]
            use super::*;
        };
        items.insert(0, use_super);
    }
}

/// Validates constraints that can only be checked after child routes are collected.
fn validate_post_recursion(route_def: &RouteDef) {
    // index!() conflicts with a #[route("/")] child.
    if route_def.index.is_some()
        && let Some(child) = route_def.children.iter().find(|c| c.path == "/")
    {
        abort!(
            child.route_ident_span,
            "Conflicting index route: this module already has `index!()`. Use either `index!()` or a `#[route(\"/\")]` child, not both."
        );
    }

    // page!() on a parent route (with children) is always invalid.
    if route_def.page.is_some() && !route_def.children.is_empty() {
        abort!(
            route_def.page_span.expect("page_span must be Some when page is Some"),
            "page!() must only be used on leaf routes (without children).";
            help = "Replace `page!()` with `layout!()` and/or `index!()`, or remove child routes to make this a leaf route."
        );
    }
}

/// Recursively walks the module tree to build [`RouteDef`] instances.
///
/// For each module with a `#[route("/...")]` attribute, this function:
/// 1. Parses the route path via [`RouteMacroArgs`].
/// 2. Enforces that the module has no explicit visibility (rewrites to `pub`).
/// 3. Injects `use super::*;` for import chaining.
/// 4. Extracts `layout!()`, `index!()`, and `page!()` expressions.
/// 5. Validates mutual exclusivity (no `layout` + `page`, no `index` + `page`).
/// 6. Validates wildcard placement and parameter name collisions via [`validate_path_segments`].
/// 7. Pre-computes `path_pattern` and `all_params` via [`compute_accumulated_context`].
/// 8. Recurses into child modules.
/// 9. Validates page-on-parent and index+route("/") conflicts.
/// 10. Pushes the resulting [`RouteDef`] into `route_defs`.
pub fn collect_route_definitions(
    module: &mut ItemMod,
    parent: Option<&ParentContext<'_>>,
    route_defs: &mut Vec<RouteDef>,
    module_path: &ModulePath,
    accumulated_path_prefix: &str,
    accumulated_params: &[ParamInfo],
) {
    let module_name = module.ident.clone();

    // Create current module path
    let mut current_module_path = module_path.clone();
    current_module_path.push(module_name.clone());

    let Some(args) = RouteMacroArgs::parse(&module.attrs) else {
        // This module was not annotated with `#[route]`. Skip it and all potential submodules.
        return;
    };

    // Route modules must not carry an explicit visibility modifier.
    // The macro automatically rewrites them to `pub`.
    if !matches!(module.vis, Visibility::Inherited) {
        abort!(
            module.vis,
            "Route modules must not have a visibility modifier. \
             The `#[route]` macro automatically makes them public. \
             Use `mod {}` instead.",
            module_name,
        );
    }
    module.vis = Visibility::Public(syn::token::Pub::default());
    let vis = module.vis.clone();

    // Keep #[route] attributes in place so the compiler expands them (they are
    // pass-through) and rust-analyzer can provide semantic highlighting / go-to-definition.

    // Inject `use super::*;` so that macro imports and component functions from the
    // parent chain downward through the module tree.
    inject_use_super(module);

    // Extract and validate layout!/index!/page! macro invocations from the module body.
    let body = module
        .content
        .as_mut()
        .map(|(_, items)| extract_body_macros(items, false))
        .unwrap_or_default();
    validate_body_macro_conflicts(&body);

    let path_segments = PathSegments::parse(&args.route_path_segments);

    validate_path_segments(
        &path_segments,
        &args.route_path_segments,
        args.route_ident_span,
        &module_name,
        accumulated_params,
    );

    let (path_pattern, all_params) = compute_accumulated_context(
        &args.route_path_segments,
        &path_segments,
        accumulated_path_prefix,
        accumulated_params,
    );

    let mut route_def = RouteDef {
        module_name: module.ident.clone(),
        module_span: module.span(),
        route_ident_span: args.route_ident_span,
        path: args.route_path_segments.clone(),
        path_segments,
        layout: body.layout,
        layout_span: body.layout_span,
        index: body.index,
        index_span: body.index_span,
        page: body.page,
        page_span: body.page_span,
        name: syn::Ident::new(
            &to_pascal_case(&module_name.to_string()),
            module_name.span(),
        ),
        parent_struct: parent.map(|p| (p.path.to_owned(), p.struct_name.clone())),
        vis,
        found_in_module_path: current_module_path.clone(),
        children: Vec::new(),
        path_pattern: path_pattern.clone(),
        all_params: all_params.clone(),
    };

    let child_path_prefix = if path_pattern == "/" {
        String::new()
    } else {
        path_pattern
    };

    if let Some((_, items)) = &mut module.content {
        let parent_ctx = ParentContext {
            path: &args.route_path_segments,
            struct_name: &route_def.name,
        };
        for item in items.iter_mut() {
            if let Item::Mod(child_module) = item {
                collect_route_definitions(
                    child_module,
                    Some(&parent_ctx),
                    &mut route_def.children,
                    &current_module_path,
                    &child_path_prefix,
                    &all_params,
                );
            }
        }
    }

    validate_post_recursion(&route_def);
    route_defs.push(route_def);
}

/// Returns a depth-first iterator over all [`RouteDef`] nodes in the route tree.
pub fn flatten(root_route_defs: &[RouteDef]) -> impl Iterator<Item = &RouteDef> {
    let mut stack = Vec::new();
    // Push in reverse so that pop() yields routes in declaration order.
    stack.extend(root_route_defs.iter().rev());
    from_fn(move || {
        if let Some(node) = stack.pop() {
            stack.extend(node.children.iter().rev());
            return Some(node);
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_path::ModulePath;
    use assertr::prelude::*;

    /// Helper to create a `RouteDef` for testing tree-traversal functions.
    fn make_route(name: &str, path: &str, children: Vec<RouteDef>) -> RouteDef {
        let ident = syn::Ident::new(name, Span::call_site());
        let pascal = to_pascal_case(name);
        RouteDef {
            module_name: ident.clone(),
            module_span: Span::call_site(),
            route_ident_span: Span::call_site(),
            path: path.to_string(),
            path_segments: PathSegments::parse(path),
            layout: None,
            layout_span: None,
            index: None,
            index_span: None,
            page: None,
            page_span: None,
            name: syn::Ident::new(&pascal, Span::call_site()),
            parent_struct: None,
            vis: Visibility::Inherited,
            found_in_module_path: ModulePath::root(ident),
            children,
            path_pattern: String::new(),
            all_params: Vec::new(),
        }
    }

    mod is_macro_named_tests {
        use super::*;

        fn make_path(name: &str) -> syn::Path {
            syn::parse_str(name).unwrap()
        }

        #[test]
        fn simple_name_matches() {
            assert_that!(is_macro_named(&make_path("layout"), "layout")).is_true();
        }

        #[test]
        fn simple_name_does_not_match() {
            assert_that!(is_macro_named(&make_path("layout"), "page")).is_false();
        }

        #[test]
        fn qualified_path_matches_last_segment() {
            assert_that!(is_macro_named(&make_path("some_crate::layout"), "layout")).is_true();
        }

        #[test]
        fn qualified_path_does_not_match_non_last_segment() {
            assert_that!(is_macro_named(
                &make_path("some_crate::layout"),
                "some_crate",
            ))
            .is_false();
        }
    }

    mod flatten_tests {
        use super::*;

        #[test]
        fn empty_input() {
            let routes: Vec<RouteDef> = vec![];
            let result: Vec<_> = flatten(&routes).collect();
            assert_that!(result).is_empty();
        }

        #[test]
        fn single_root() {
            let routes = vec![make_route("root", "/", vec![])];
            let result: Vec<_> = flatten(&routes).collect();
            assert_that!(result.as_slice()).has_length(1);
            assert_that!(result[0].path.as_str()).is_equal_to("/");
        }

        #[test]
        fn parent_with_children() {
            let child_a = make_route("a", "/a", vec![]);
            let child_b = make_route("b", "/b", vec![]);
            let root = make_route("root", "/", vec![child_a, child_b]);
            let routes = vec![root];
            let result: Vec<_> = flatten(&routes).map(|r| r.path.as_str()).collect();
            assert_that!(result.as_slice()).has_length(3);
            // Preserves declaration order: root, then children in order.
            assert_that!(result[0]).is_equal_to("/");
            assert_that!(result[1]).is_equal_to("/a");
            assert_that!(result[2]).is_equal_to("/b");
        }

        #[test]
        fn deeply_nested() {
            let grandchild = make_route("details", "/details", vec![]);
            let child = make_route("user", "/:id", vec![grandchild]);
            let root = make_route("users", "/users", vec![child]);
            let routes = vec![root];
            let result: Vec<_> = flatten(&routes).map(|r| r.path.as_str()).collect();
            assert_that!(result.as_slice()).has_length(3);
            assert_that!(result[0]).is_equal_to("/users");
        }

        #[test]
        fn multiple_roots() {
            let root_a = make_route("users", "/users", vec![]);
            let root_b = make_route("posts", "/posts", vec![]);
            let routes = vec![root_a, root_b];
            let result: Vec<_> = flatten(&routes).map(|r| r.path.as_str()).collect();
            assert_that!(result.as_slice()).has_length(2);
            assert_that!(result[0]).is_equal_to("/users");
            assert_that!(result[1]).is_equal_to("/posts");
        }

        #[test]
        fn wide_tree() {
            let children: Vec<_> = (0..5)
                .map(|i| {
                    let name = format!("child{i}");
                    let path = format!("/child{i}");
                    make_route(&name, &path, vec![])
                })
                .collect();
            let root = make_route("root", "/", children);
            let routes = vec![root];
            let result: Vec<_> = flatten(&routes).collect();
            assert_that!(result.as_slice()).has_length(6); // root + 5 children
        }
    }

    mod extract_body_macros_tests {
        use super::*;

        fn parse_items(code: &str) -> Vec<Item> {
            let file: syn::File = syn::parse_str(code).unwrap();
            file.items
        }

        #[test]
        fn empty_items_returns_all_none() {
            let body = extract_body_macros(&[], false);
            assert_that!(body.layout).is_none();
            assert_that!(body.index).is_none();
            assert_that!(body.page).is_none();
        }

        #[test]
        fn layout_only() {
            let items = parse_items("layout!(SomeLayout);");
            let body = extract_body_macros(&items, false);
            assert_that!(body.layout).is_some();
            assert_that!(body.index).is_none();
            assert_that!(body.page).is_none();
        }

        #[test]
        fn page_only() {
            let items = parse_items("page!(SomePage);");
            let body = extract_body_macros(&items, false);
            assert_that!(body.layout).is_none();
            assert_that!(body.index).is_none();
            assert_that!(body.page).is_some();
        }

        #[test]
        fn index_only() {
            let items = parse_items("index!(SomeIndex);");
            let body = extract_body_macros(&items, false);
            assert_that!(body.layout).is_none();
            assert_that!(body.index).is_some();
            assert_that!(body.page).is_none();
        }

        #[test]
        fn layout_and_index() {
            let items = parse_items("layout!(L); index!(I);");
            let body = extract_body_macros(&items, false);
            assert_that!(body.layout).is_some();
            assert_that!(body.index).is_some();
            assert_that!(body.page).is_none();
        }

        #[test]
        fn non_macro_items_are_ignored() {
            let items = parse_items("fn foo() {} struct Bar;");
            let body = extract_body_macros(&items, false);
            assert_that!(body.layout).is_none();
            assert_that!(body.index).is_none();
            assert_that!(body.page).is_none();
        }

        #[test]
        fn fallback_at_root_is_skipped() {
            let items = parse_items("fallback!(F);");
            let body = extract_body_macros(&items, true);
            // fallback is silently skipped by extract_body_macros when at_root.
            assert_that!(body.layout).is_none();
            assert_that!(body.index).is_none();
            assert_that!(body.page).is_none();
        }
    }

    mod extract_root_fallback_tests {
        use super::*;

        fn parse_items(code: &str) -> Vec<Item> {
            let file: syn::File = syn::parse_str(code).unwrap();
            file.items
        }

        #[test]
        fn no_fallback_returns_none() {
            let items = parse_items("fn foo() {}");
            assert_that!(extract_root_fallback(&items)).is_none();
        }

        #[test]
        fn fallback_present_returns_some() {
            let items = parse_items("fallback!(MyFallback);");
            assert_that!(extract_root_fallback(&items)).is_some();
        }

        #[test]
        fn empty_items_returns_none() {
            assert_that!(extract_root_fallback(&[])).is_none();
        }
    }

    mod compute_accumulated_context_tests {
        use super::*;

        #[test]
        fn root_path_with_empty_prefix() {
            let segments = PathSegments::parse("/");
            let (pattern, params) = compute_accumulated_context("/", &segments, "", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/");
            assert_that!(params).is_empty();
        }

        #[test]
        fn static_path_with_empty_prefix() {
            let segments = PathSegments::parse("/users");
            let (pattern, params) = compute_accumulated_context("/users", &segments, "", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/users");
            assert_that!(params).is_empty();
        }

        #[test]
        fn static_path_with_root_prefix() {
            let segments = PathSegments::parse("/users");
            let (pattern, params) = compute_accumulated_context("/users", &segments, "/", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/users");
            assert_that!(params).is_empty();
        }

        #[test]
        fn static_path_with_non_root_prefix() {
            let segments = PathSegments::parse("/details");
            let (pattern, params) =
                compute_accumulated_context("/details", &segments, "/users", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/users/details");
            assert_that!(params).is_empty();
        }

        #[test]
        fn param_path_accumulates_params() {
            let segments = PathSegments::parse("/:id");
            let (pattern, params) = compute_accumulated_context("/:id", &segments, "/users", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/users/:id");
            assert_that!(params.as_slice()).has_length(1);
            assert_that!(params[0].name.as_str()).is_equal_to("id");
            assert_that!(params[0].is_optional).is_false();
        }

        #[test]
        fn optional_param_marked_correctly() {
            let segments = PathSegments::parse("/:page?");
            let (pattern, params) =
                compute_accumulated_context("/:page?", &segments, "/posts", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/posts/:page?");
            assert_that!(params.as_slice()).has_length(1);
            assert_that!(params[0].name.as_str()).is_equal_to("page");
            assert_that!(params[0].is_optional).is_true();
        }

        #[test]
        fn wildcard_param_is_not_optional() {
            let segments = PathSegments::parse("/*rest");
            let (pattern, params) = compute_accumulated_context("/*rest", &segments, "/files", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/files/*rest");
            assert_that!(params.as_slice()).has_length(1);
            assert_that!(params[0].name.as_str()).is_equal_to("rest");
            assert_that!(params[0].is_optional).is_false();
        }

        #[test]
        fn inherits_parent_params() {
            let parent_params = vec![ParamInfo {
                name: "org".to_string(),
                is_optional: false,
                origin_path: "/orgs/:org".to_string(),
            }];
            let segments = PathSegments::parse("/:repo");
            let (pattern, params) =
                compute_accumulated_context("/:repo", &segments, "/orgs/:org", &parent_params);
            assert_that!(pattern.as_str()).is_equal_to("/orgs/:org/:repo");
            assert_that!(params.as_slice()).has_length(2);
            assert_that!(params[0].name.as_str()).is_equal_to("org");
            assert_that!(params[1].name.as_str()).is_equal_to("repo");
        }

        #[test]
        fn root_path_with_non_empty_prefix_uses_prefix() {
            let segments = PathSegments::parse("/");
            let (pattern, params) = compute_accumulated_context("/", &segments, "/api", &[]);
            assert_that!(pattern.as_str()).is_equal_to("/api");
            assert_that!(params).is_empty();
        }
    }
}
