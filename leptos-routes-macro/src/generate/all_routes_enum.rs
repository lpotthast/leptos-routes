//! Generates the `Route` enum with one variant per route in the module tree.
//!
//! The enum provides `path_pattern()` (full path string), `all()` (static slice
//! of all variants), and a `Display` impl.

use crate::route_def::{RouteDef, flatten};
use crate::util::to_pascal_case;
use quote::quote;

/// Generates the `Route` enum, its `impl` block, and its `Display` impl.
///
/// Works in three phases:
/// 1. **Variants**: For each route, builds a variant name by concatenating the
///    `PascalCase`d intermediate module segments with the struct name
///    (e.g., module path `[users, user]` + struct `Details` → `UsersUserDetails`).
/// 2. **Match arms**: Maps each variant to its `path_pattern()` for `path_pattern()`.
/// 3. **Instances**: Builds the `all()` array with one instance per variant.
///
/// **Note:** Variant names are built by concatenating `PascalCase`'d intermediate
/// module names. In rare cases, different module hierarchies can produce the same
/// variant name (e.g., `mod ab { mod cd {} }` and `mod a_b { mod cd {} }` both
/// `PascalCase` to `AbCd`). The compiler will report a duplicate variant error in
/// this case. Rename one of the conflicting modules to resolve it.
pub fn generate_route_enum(route_defs: &[RouteDef]) -> Vec<proc_macro2::TokenStream> {
    let mut all_routes_variants = Vec::new();
    let mut path_pattern_arms = Vec::new();
    let mut all_instances = Vec::new();

    for route_def in flatten(route_defs) {
        let struct_name = &route_def.name;

        let paths = &route_def.found_in_module_path.intermediate_segments();

        let variant_name = {
            let prefix = paths.iter().fold(String::new(), |mut acc, seg| {
                acc.push_str(&to_pascal_case(&seg.to_string()));
                acc
            });
            if prefix.is_empty() {
                struct_name.clone()
            } else {
                syn::Ident::new(&format!("{prefix}{struct_name}"), struct_name.span())
            }
        };
        let path = quote! { #(#paths::)*#struct_name };

        all_routes_variants.push(quote! {
            #variant_name(#path),
        });

        path_pattern_arms.push(quote! {
            Route::#variant_name(inner) => inner.path_pattern(),
        });

        all_instances.push(quote! {
            Route::#variant_name(#path)
        });
    }

    let all_routes_enum = quote! {
        /// An enum with a variant for each route defined in this module.
        ///
        /// Each variant wraps the corresponding route struct. Note that `materialize()`
        /// cannot be called generically through this enum, as each route may require
        /// different parameters. Use the individual route structs directly for materialization.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Route {
            #(#all_routes_variants)*
        }
    };

    let route_impl = quote! {
        impl Route {
            /// Returns the full path pattern of this route, including all parent segments.
            pub fn path_pattern(&self) -> &'static str {
                match self {
                    #(#path_pattern_arms)*
                }
            }

            /// Returns a static slice containing one instance of every route variant.
            pub fn all() -> &'static [Route] {
                &[#(#all_instances),*]
            }
        }
    };

    let display_impl = quote! {
        impl ::std::fmt::Display for Route {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.path_pattern())
            }
        }
    };

    vec![all_routes_enum, route_impl, display_impl]
}
